//! `ctx doctor`: lightweight project diagnostics — focused on ctx, not a
//! general system-management tool.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

use crate::config::Config;
use crate::errors::CtxResult;
use crate::git::GitRepo;
use crate::graph::database::Database;
use crate::indexing::scanner::scan;
use crate::lang::LanguageId;
use crate::output::{Term, emit_json};

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub root: String,
    pub git: bool,
    pub git_root: Option<String>,
    pub languages: Vec<String>,
    pub framework: Option<String>,
    pub package_manager: Option<String>,
    pub index: IndexStatus,
    pub database: Option<DatabaseStatus>,
    pub parser_support: Vec<ParserSupport>,
    pub status: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexStatus {
    pub initialized: bool,
    pub fresh: bool,
    pub files: u64,
    pub symbols: u64,
    pub dependencies: u64,
    pub stale_files: usize,
    pub unindexed_files: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseStatus {
    pub healthy: bool,
    pub schema_version: i64,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParserSupport {
    pub language: String,
    pub supported: bool,
}

pub fn cmd_doctor(cwd: &Path, root_override: Option<&Path>, t: &Term) -> CtxResult<()> {
    let root = match root_override {
        Some(r) => r.to_path_buf(),
        None => cwd.to_path_buf(),
    };
    let report = doctor(&root)?;

    if t.is_json() {
        emit_json(&serde_json::to_value(&report)?);
        return Ok(());
    }

    println!("{}", t.style(crate::output::Default::BOLD, "ctx doctor"));
    println!();
    println!("Project:");
    println!("  {}", report.root);
    if report.git {
        println!("{} Git repository detected", t.ok(""));
        if let Some(gr) = &report.git_root {
            println!("  git root: {gr}");
        }
    } else {
        println!("  no git repository detected");
    }
    if !report.languages.is_empty() {
        println!("Language:");
        println!("  {}", report.languages.join(", "));
    }
    if let Some(fw) = &report.framework {
        println!("Framework:");
        println!("  {fw}");
    }
    if let Some(pm) = &report.package_manager {
        println!("Package manager:");
        println!("  {pm}");
    }

    println!("Index:");
    match &report.index.initialized {
        false => println!(
            "  {} .ctx index not found",
            t.style(crate::output::Default::RED, "✗")
        ),
        true => {
            println!(
                "  {} .ctx database {}",
                t.ok(""),
                if report.database.as_ref().map(|d| d.healthy).unwrap_or(false) {
                    "healthy"
                } else {
                    "unhealthy"
                }
            );
            println!("  {} {} files indexed", t.ok(""), report.index.files);
            println!("  {} {} symbols indexed", t.ok(""), report.index.symbols);
            println!(
                "  {} {} dependencies indexed",
                t.ok(""),
                report.index.dependencies
            );
            if report.index.fresh {
                println!("  {} Index is current", t.ok(""));
            } else {
                println!(
                    "  {} Index is stale ({} files changed, {} new files)",
                    t.style(crate::output::Default::YELLOW, "⚠"),
                    report.index.stale_files,
                    report.index.unindexed_files
                );
            }
        }
    }

    println!("Parser support:");
    for p in &report.parser_support {
        let mark = if p.supported {
            t.ok("")
        } else {
            t.style(crate::output::Default::RED, "✗")
        };
        println!("  {mark} {}", p.language);
    }

    for w in &report.warnings {
        println!("{} {w}", t.style(crate::output::Default::YELLOW, "⚠"));
    }

    println!();
    match report.status.as_str() {
        "READY" => println!(
            "{}",
            t.ok(&t.style(crate::output::Default::BOLD, "Status: READY"))
        ),
        "STALE" => println!(
            "{}",
            t.style(
                crate::output::Default::YELLOW,
                "Status: STALE — run `ctx init` to re-index"
            )
        ),
        _ => println!(
            "{}",
            t.style(
                crate::output::Default::RED,
                "Status: NOT INITIALIZED — run `ctx init` to index this project"
            )
        ),
    }
    Ok(())
}

fn doctor(root: &Path) -> CtxResult<DoctorReport> {
    let mut warnings: Vec<String> = Vec::new();

    let git = GitRepo::discover(root)?;
    let git_root = git.as_ref().map(|g| g.root.display().to_string());

    let config = Config::load(root)?;

    let initialized = Database::exists(root);
    let mut index = IndexStatus {
        initialized,
        fresh: !initialized,
        files: 0,
        symbols: 0,
        dependencies: 0,
        stale_files: 0,
        unindexed_files: 0,
    };
    let mut database = None;

    if initialized {
        let db = Database::open(root)?;
        let (files, symbols, deps) = db.stats()?;
        index.files = files as u64;
        index.symbols = symbols as u64;
        index.dependencies = deps as u64;

        let healthy = db
            .conn()
            .query_row("PRAGMA quick_check", [], |r| r.get::<_, String>(0))
            .map(|s| s == "ok")
            .unwrap_or(false);
        let schema_version = db
            .conn()
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap_or(0);
        database = Some(DatabaseStatus {
            healthy,
            schema_version,
            path: root
                .join(crate::graph::database::DB_PATH)
                .display()
                .to_string(),
        });
        if !healthy {
            warnings.push("SQLite quick_check reported an unhealthy database; run `ctx init --force` to rebuild it.".into());
        }

        // freshness: compare indexed metadata with disk, and discover new files
        let records = db.all_files()?;
        let mut indexed: BTreeMap<String, (i64, i64)> = BTreeMap::new();
        for f in records {
            indexed.insert(f.path.clone(), (f.mtime, f.size));
        }
        let discovered = scan(root, &config).unwrap_or_default();
        let mut stale = 0usize;
        let mut unindexed = 0usize;
        for d in &discovered {
            match indexed.get(&d.rel_path) {
                None => unindexed += 1,
                Some(&(mtime, size)) => {
                    if mtime != d.mtime || size != d.size {
                        stale += 1;
                    }
                }
            }
        }
        for path in indexed.keys() {
            if !root.join(path).exists() {
                stale += 1;
            }
        }
        index.stale_files = stale;
        index.unindexed_files = unindexed;
        index.fresh = stale == 0 && unindexed == 0;
    }

    let mut languages: Vec<String> = Vec::new();
    if initialized {
        if let Some(db) = &database {
            let _ = db;
        }
        if let Ok(db) = Database::open(root) {
            for f in db.all_files()? {
                if let Some(lang) = f.language
                    && !languages.contains(&lang)
                {
                    languages.push(lang);
                }
            }
        }
    } else {
        let discovered = scan(root, &config).unwrap_or_default();
        for d in discovered {
            let name = d.language.display_name().to_string();
            if !languages.contains(&name) {
                languages.push(name);
            }
        }
    }
    languages.sort();

    let framework = detect_framework(root);
    let package_manager = detect_package_manager(root);

    let parser_support: Vec<ParserSupport> = LanguageId::all()
        .iter()
        .map(|l| ParserSupport {
            language: l.display_name().to_string(),
            supported: true,
        })
        .collect();

    let status = if !initialized {
        "NOT_INITIALIZED".to_string()
    } else if index.fresh {
        "READY".to_string()
    } else {
        "STALE".to_string()
    };

    Ok(DoctorReport {
        root: root.display().to_string(),
        git: git.is_some(),
        git_root,
        languages,
        framework,
        package_manager,
        index,
        database,
        parser_support,
        status,
        warnings,
    })
}

fn detect_framework(root: &Path) -> Option<String> {
    let pkg = root.join("package.json");
    if pkg.is_file() {
        if let Ok(raw) = std::fs::read_to_string(&pkg)
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw)
        {
            let mut deps: Vec<String> = Vec::new();
            if let Some(d) = v.get("dependencies").and_then(|d| d.as_object()) {
                deps.extend(d.keys().map(|k| k.to_string()));
            }
            if let Some(d) = v.get("devDependencies").and_then(|d| d.as_object()) {
                deps.extend(d.keys().map(|k| k.to_string()));
            }
            for key in [
                "next",
                "nuxt",
                "remix",
                "gatsby",
                "svelte",
                "angular",
                "vue",
                "react",
                "@nestjs/core",
                "express",
                "fastify",
                "koa",
                "graphql",
                "prisma",
                "drizzle-orm",
                "tailwindcss",
                "vitest",
                "jest",
            ] {
                if deps.iter().any(|d| d == key) {
                    return Some(match key {
                        "next" => "Next.js".to_string(),
                        "nuxt" => "Nuxt".to_string(),
                        "remix" => "Remix".to_string(),
                        "svelte" => "SvelteKit".to_string(),
                        "angular" => "Angular".to_string(),
                        "vue" => "Vue".to_string(),
                        "react" => "React".to_string(),
                        "@nestjs/core" => "NestJS".to_string(),
                        "express" => "Express".to_string(),
                        "fastify" => "Fastify".to_string(),
                        "koa" => "Koa".to_string(),
                        "graphql" => "GraphQL".to_string(),
                        "prisma" => "Prisma".to_string(),
                        "drizzle-orm" => "Drizzle ORM".to_string(),
                        "tailwindcss" => "Tailwind CSS".to_string(),
                        "vitest" => "Vitest".to_string(),
                        "jest" => "Jest".to_string(),
                        _ => key.to_string(),
                    });
                }
            }
        }
        return None;
    }
    let cargo = root.join("Cargo.toml");
    if cargo.is_file() {
        let raw = std::fs::read_to_string(&cargo).unwrap_or_default();
        for key in ["actix-web", "axum", "rocket", "tokio", "serde", "clap"] {
            if raw.contains(&format!("{key} ")) || raw.contains(&format!("\n{key}=")) {
                return Some(match key {
                    "actix-web" => "Actix Web".to_string(),
                    "axum" => "Axum".to_string(),
                    "rocket" => "Rocket".to_string(),
                    "tokio" => "Tokio".to_string(),
                    "serde" => "Serde".to_string(),
                    "clap" => "Clap".to_string(),
                    _ => key.to_string(),
                });
            }
        }
        return None;
    }
    None
}

fn detect_package_manager(root: &Path) -> Option<String> {
    for (file, name) in [
        ("pnpm-lock.yaml", "pnpm"),
        ("yarn.lock", "yarn"),
        ("package-lock.json", "npm"),
        ("bun.lockb", "bun"),
        ("bun.lock", "bun"),
        ("uv.lock", "uv"),
        ("poetry.lock", "poetry"),
        ("Cargo.lock", "cargo"),
        ("go.sum", "go modules"),
    ] {
        if root.join(file).is_file() {
            return Some(name.to_string());
        }
    }
    None
}
