use crate::commands::Project;
use crate::context::build_context_with;
use crate::errors::CtxResult;
use crate::output::{Default, Term, emit_json};

pub fn cmd_context(
    project: &Project,
    task: &str,
    include_bodies: bool,
    max_tokens: Option<usize>,
    no_git: bool,
    t: &Term,
) -> CtxResult<()> {
    project.require_initialized()?;

    // Consider working-tree changes when ranking (cheap: one git call). The
    // flag reports whether the git signal was *consulted*, independent of
    // whether any changes were found.
    let git_changes: Option<Vec<String>> = if !no_git && project.git.is_some() {
        project
            .git
            .as_ref()
            .and_then(|git| crate::git::changed::changed_files(git, None).ok())
            .map(|files| files.into_iter().map(|c| c.path).collect())
    } else {
        None
    };

    let package = build_context_with(
        &project.db,
        &project.root,
        task,
        &project.config,
        include_bodies,
        max_tokens,
        git_changes.as_deref(),
    )?;

    if t.is_json() {
        emit_json(&serde_json::to_value(&package)?);
        return Ok(());
    }

    println!("PROJECT CONTEXT");
    println!();
    println!("Task:");
    println!("  {task}");
    println!();
    if !package.keywords.is_empty() {
        println!("Keywords:");
        println!("  {}", package.keywords.join(", "));
        println!();
    }

    if !package.architecture.is_empty() {
        println!("Relevant architecture:");
        for line in &package.architecture {
            println!("  {line}");
        }
        println!();
    }

    if !package.relevant_symbols.is_empty() {
        println!("Relevant symbols:");
        for s in &package.relevant_symbols {
            println!(
                "  {}  {}:{}  ({})",
                t.style(Default::BOLD, &s.name),
                s.path,
                s.line,
                s.kind
            );
        }
        println!();
    }

    if !package.relevant_dependencies.is_empty() {
        println!("Relevant dependencies:");
        for d in package.relevant_dependencies.iter().take(12) {
            println!("  {d}");
        }
        println!();
    }

    if !package.files.is_empty() {
        println!("Suggested files:");
        for f in &package.files {
            println!("  {}  (score {:.2}, ~{} tokens)", f.path, f.score, f.tokens);
            for r in &f.reasons {
                println!("      + {r}");
            }
        }
        println!();
        println!(
            "Context budget: {} / {} tokens (estimate)",
            package.total_tokens, package.budget
        );
        if package.omitted_files > 0 {
            println!("Omitted: {} lower-relevance files", package.omitted_files);
        }
        if package.budget_exceeded {
            println!(
                "{} budget exceeded — increase --max-tokens or reduce include_bodies",
                t.style(Default::YELLOW, "note:")
            );
        }
        if package.git_changes_considered {
            println!("Git changes considered: yes");
        }
        println!();
        println!("Suggested context:");
        println!("{}", package.suggested_context);
    } else {
        println!("No relevant context found for this task.");
    }
    let _ = t;
    Ok(())
}
