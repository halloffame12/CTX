//! Embedded SQLite code graph. Versioned schema, WAL mode, batched writes.

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::errors::{CtxError, CtxResult};
use crate::lang::LanguageId;

pub const SCHEMA_VERSION: i64 = 1;

pub const DB_PATH: &str = ".ctx/index.db";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileRecord {
    pub id: i64,
    pub path: String,
    pub hash: String,
    pub mtime: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub size: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SymbolRow {
    pub id: i64,
    pub file_id: i64,
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    pub exported: bool,
    pub start_line: u32,
    pub end_line: u32,
    pub start_byte: i64,
    pub end_byte: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DepRow {
    pub id: i64,
    pub source_file_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_file_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imported_symbol: Option<String>,
    pub dependency_type: String,
    pub source_raw: String,
}

#[derive(Debug, Clone)]
pub struct FileRef {
    pub path: String,
}

impl FileRef {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

pub struct Database {
    conn: Connection,
    path: std::path::PathBuf,
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Database({})", self.path.display())
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        // SQLite is file-backed; a fresh connection over the same WAL db is
        // an equivalent handle (open() also runs migrations, which are no-ops).
        let root = self
            .path
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or_else(|| std::path::Path::new("."));
        Database::open(root).expect("reopen ctx index db")
    }
}

impl Database {
    pub fn open(root: &Path) -> CtxResult<Database> {
        let dir = root.join(".ctx");
        std::fs::create_dir_all(&dir)
            .map_err(|e| CtxError::Io(format!("creating {}: {e}", dir.display())))?;
        let path = dir.join("index.db");
        let conn = Connection::open(&path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let mut db = Database { conn, path };
        db.migrate()?;
        Ok(db)
    }

    pub fn exists(root: &Path) -> bool {
        root.join(DB_PATH).is_file()
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    pub fn begin(&mut self) -> CtxResult<Transaction<'_>> {
        Ok(self.conn.transaction()?)
    }

    fn migrate(&mut self) -> CtxResult<()> {
        let v: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap_or(0);
        if v >= SCHEMA_VERSION {
            return Ok(());
        }
        let tx = self.conn.transaction()?;
        tx.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                hash TEXT NOT NULL,
                mtime INTEGER NOT NULL,
                language TEXT,
                size INTEGER
            );
            CREATE TABLE IF NOT EXISTS symbols (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                signature TEXT,
                parent TEXT,
                visibility TEXT,
                exported INTEGER NOT NULL DEFAULT 0,
                start_line INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                start_byte INTEGER,
                end_byte INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
            CREATE INDEX IF NOT EXISTS idx_symbols_name_kind ON symbols(name, kind);
            CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file_id);
            CREATE TABLE IF NOT EXISTS dependencies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
                target_file_id INTEGER REFERENCES files(id) ON DELETE CASCADE,
                imported_symbol TEXT,
                dependency_type TEXT,
                source_raw TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_deps_target ON dependencies(target_file_id);
            CREATE INDEX IF NOT EXISTS idx_deps_source ON dependencies(source_file_id);
            "#,
        )?;
        tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        tx.commit()?;
        Ok(())
    }

    // ---- writes ------------------------------------------------------------

    pub fn wipe(&mut self) -> CtxResult<()> {
        let tx = self.conn.transaction()?;
        tx.execute_batch(
            "DELETE FROM dependencies;
             DELETE FROM symbols;
             DELETE FROM files;",
        )?;
        tx.commit()?;
        Ok(())
    }

    // ---- reads -------------------------------------------------------------

    pub fn file_by_path(&self, path: &str) -> CtxResult<Option<FileRecord>> {
        Ok(self
            .conn
            .query_row(
                "SELECT id, path, hash, mtime, language, size FROM files WHERE path = ?1",
                params![path],
                row_file,
            )
            .optional()?)
    }

    pub fn file_by_id(&self, id: i64) -> CtxResult<Option<FileRecord>> {
        Ok(self
            .conn
            .query_row(
                "SELECT id, path, hash, mtime, language, size FROM files WHERE id = ?1",
                params![id],
                row_file,
            )
            .optional()?)
    }

    pub fn all_files(&self) -> CtxResult<Vec<FileRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, path, hash, mtime, language, size FROM files")?;
        let rows = stmt
            .query_map([], row_file)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn files_like(&self, needle: &str, limit: usize) -> CtxResult<Vec<FileRecord>> {
        let pattern = format!("%{}%", escape_like(needle));
        let mut stmt = self.conn.prepare(
            "SELECT id, path, hash, mtime, language, size FROM files WHERE path LIKE ?1 ESCAPE '\\' ORDER BY path LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![pattern, limit as i64], row_file)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn stats(&self) -> CtxResult<(i64, i64, i64)> {
        let files: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |r| r.get(0))?;
        let symbols: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get(0))?;
        let deps: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM dependencies", [], |r| r.get(0))?;
        Ok((files, symbols, deps))
    }

    pub fn schema_sql(&self) -> CtxResult<String> {
        let mut stmt = self.conn.prepare(
            "SELECT type, name, sql FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let lines: Vec<String> = rows
            .into_iter()
            .map(|(ty, name, sql)| match sql {
                Some(s) => format!("[{ty}] {name}\n{s}"),
                None => format!("[{ty}] {name}"),
            })
            .collect();
        Ok(lines.join("\n;\n"))
    }

    pub fn symbols_for_file(&self, file_id: i64) -> CtxResult<Vec<SymbolRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, name, kind, signature, parent, visibility, exported,
                    start_line, end_line, start_byte, end_byte
             FROM symbols WHERE file_id = ?1 ORDER BY start_line",
        )?;
        let rows = stmt
            .query_map(params![file_id], row_symbol)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn symbols_by_name(&self, name: &str, limit: usize) -> CtxResult<Vec<SymbolRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.file_id, s.name, s.kind, s.signature, s.parent, s.visibility, s.exported,
                    s.start_line, s.end_line, s.start_byte, s.end_byte
             FROM symbols s WHERE s.name = ?1 ORDER BY s.start_line LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![name, limit as i64], row_symbol)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Look up a symbol whose parent (containing class/struct) matches `parent`
    /// and whose own name matches `name`. Used for qualified lookups such as
    /// `UserService.updateUser`.
    pub fn symbols_by_parent_and_name(
        &self,
        parent: &str,
        name: &str,
        limit: usize,
    ) -> CtxResult<Vec<SymbolRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.file_id, s.name, s.kind, s.signature, s.parent, s.visibility, s.exported,
                    s.start_line, s.end_line, s.start_byte, s.end_byte
             FROM symbols s WHERE s.parent = ?1 AND s.name = ?2
             ORDER BY s.start_line LIMIT ?3",
        )?;
        let rows = stmt
            .query_map(params![parent, name, limit as i64], row_symbol)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Search across symbol names and file paths. Uses substring + prefix
    /// heuristics; deterministic ordering.
    pub fn search(
        &self,
        term: &str,
        kind: Option<&str>,
        limit: usize,
    ) -> CtxResult<Vec<SymbolRow>> {
        let pattern = format!("%{}%", escape_like(term));
        let exact = term.to_string();
        let base = "SELECT s.id, s.file_id, s.name, s.kind, s.signature, s.parent, s.visibility, s.exported,
                    s.start_line, s.end_line, s.start_byte, s.end_byte
             FROM symbols s";
        let (sql, bound): (String, Vec<Box<dyn rusqlite::ToSql>>) = match kind {
            Some(k) => (
                format!(
                    "{base} WHERE s.name LIKE ?1 ESCAPE '\\' AND s.kind = ?2
                     ORDER BY CASE WHEN s.name = ?3 THEN 0 ELSE 1 END, s.start_line LIMIT ?4"
                ),
                vec![
                    Box::new(pattern),
                    Box::new(k.to_string()),
                    Box::new(exact),
                    Box::new(limit as i64),
                ],
            ),
            None => (
                format!(
                    "{base} WHERE s.name LIKE ?1 ESCAPE '\\'
                     ORDER BY CASE WHEN s.name = ?2 THEN 0 ELSE 1 END, s.start_line LIMIT ?3"
                ),
                vec![Box::new(pattern), Box::new(exact), Box::new(limit as i64)],
            ),
        };
        let mut stmt = self.conn.prepare_cached(&sql)?;
        let rows = stmt
            .query_map(
                rusqlite::params_from_iter(bound.iter().map(|b| b.as_ref())),
                row_symbol,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn dependencies_of(&self, file_id: i64) -> CtxResult<Vec<DepRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source_file_id, target_file_id, imported_symbol, dependency_type, source_raw
             FROM dependencies WHERE source_file_id = ?1 ORDER BY id",
        )?;
        let rows = stmt
            .query_map(params![file_id], row_dep)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Files that depend on `file_id` (incoming edges), with the symbol they
    /// imported if any.
    pub fn dependents_of(&self, file_id: i64) -> CtxResult<Vec<(String, Option<String>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT f.path, d.imported_symbol
             FROM dependencies d JOIN files f ON f.id = d.source_file_id
             WHERE d.target_file_id = ?1 ORDER BY f.path",
        )?;
        let rows = stmt
            .query_map(params![file_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn internal_dependencies_of(
        &self,
        file_id: i64,
    ) -> CtxResult<Vec<(String, Option<String>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT f.path, d.imported_symbol
             FROM dependencies d JOIN files f ON f.id = d.target_file_id
             WHERE d.source_file_id = ?1 AND d.target_file_id IS NOT NULL ORDER BY f.path",
        )?;
        let rows = stmt
            .query_map(params![file_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn file_symbol_count(&self, file_id: i64) -> CtxResult<i64> {
        Ok(self.conn.query_row(
            "SELECT COUNT(*) FROM symbols WHERE file_id = ?1",
            params![file_id],
            |r| r.get(0),
        )?)
    }

    pub fn file_dependents_count(&self, file_id: i64) -> CtxResult<i64> {
        Ok(self.conn.query_row(
            "SELECT COUNT(*) FROM dependencies WHERE target_file_id = ?1",
            params![file_id],
            |r| r.get(0),
        )?)
    }

    // ---- writes ------------------------------------------------------------

    pub fn upsert_file(
        tx: &Transaction,
        path: &str,
        hash: &str,
        mtime: i64,
        language: &str,
        size: i64,
    ) -> CtxResult<i64> {
        tx.execute(
            "INSERT INTO files (path, hash, mtime, language, size) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(path) DO UPDATE SET hash=?2, mtime=?3, language=?4, size=?5",
            params![path, hash, mtime, language, size],
        )?;
        let id: i64 = tx.query_row("SELECT id FROM files WHERE path = ?1", params![path], |r| {
            r.get(0)
        })?;
        Ok(id)
    }

    pub fn delete_file(tx: &Transaction, path: &str) -> CtxResult<()> {
        tx.execute("DELETE FROM files WHERE path = ?1", params![path])?;
        Ok(())
    }

    /// Replace symbols + dependencies for a file (used on reindex).
    pub fn replace_symbols(
        tx: &Transaction,
        file_id: i64,
        symbols: &[crate::parser::Symbol],
    ) -> CtxResult<()> {
        tx.execute("DELETE FROM symbols WHERE file_id = ?1", params![file_id])?;
        let mut stmt = tx.prepare(
            "INSERT INTO symbols (file_id, name, kind, signature, parent, visibility, exported,
                                 start_line, end_line, start_byte, end_byte)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        )?;
        for s in symbols {
            stmt.execute(params![
                file_id,
                s.name,
                s.kind.as_str(),
                s.signature,
                s.parent,
                s.visibility,
                s.exported as i64,
                s.span.start_line as i64,
                s.span.end_line as i64,
                s.span.start_byte as i64,
                s.span.end_byte as i64,
            ])?;
        }
        Ok(())
    }

    pub fn replace_dependencies(
        tx: &Transaction,
        file_id: i64,
        deps: &[crate::parser::Dependency],
        file_ids_by_path: &std::collections::HashMap<String, i64>,
    ) -> CtxResult<()> {
        tx.execute(
            "DELETE FROM dependencies WHERE source_file_id = ?1",
            params![file_id],
        )?;
        let mut stmt = tx.prepare(
            "INSERT INTO dependencies (source_file_id, target_file_id, imported_symbol, dependency_type, source_raw)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for d in deps {
            let target = match &d.resolved {
                crate::parser::ResolvedDependency::Internal(rel) => {
                    file_ids_by_path.get(rel).copied()
                }
                _ => None,
            };
            stmt.execute(params![
                file_id,
                target,
                d.imported_symbol,
                d.dependency_type.as_str(),
                d.source_raw,
            ])?;
        }
        Ok(())
    }

    /// Map project-relative file paths to their file ids.
    pub fn path_id_map(&self) -> CtxResult<std::collections::HashMap<String, i64>> {
        let mut map = std::collections::HashMap::new();
        let mut stmt = self.conn.prepare("SELECT id, path FROM files")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
        for row in rows {
            let (id, path) = row?;
            map.insert(path, id);
        }
        Ok(map)
    }

    /// Bulk load for the context engine: all files, all symbols, and an
    /// incoming-dependency count per file.
    pub fn context_load(&self) -> CtxResult<ContextData> {
        let mut files = std::collections::HashMap::new();
        for f in self.all_files()? {
            files.insert(f.id, f);
        }
        let symbols = {
            let mut stmt = self.conn.prepare(
                "SELECT id, file_id, name, kind, signature, parent, visibility, exported,
                        start_line, end_line, start_byte, end_byte FROM symbols",
            )?;

            stmt.query_map([], row_symbol)?
                .collect::<Result<Vec<_>, _>>()?
        };
        let mut dep_counts = std::collections::HashMap::new();
        {
            let mut stmt = self
                .conn
                .prepare("SELECT target_file_id, COUNT(*) FROM dependencies WHERE target_file_id IS NOT NULL GROUP BY target_file_id")?;
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?
                .collect::<Result<Vec<_>, _>>()?;
            for (id, n) in rows {
                dep_counts.insert(id, n);
            }
        }
        Ok(ContextData {
            files,
            symbols,
            dep_counts,
        })
    }
}

#[derive(Debug)]
pub struct ContextData {
    pub files: std::collections::HashMap<i64, FileRecord>,
    pub symbols: Vec<SymbolRow>,
    pub dep_counts: std::collections::HashMap<i64, i64>,
}

fn row_file(r: &rusqlite::Row) -> rusqlite::Result<FileRecord> {
    Ok(FileRecord {
        id: r.get(0)?,
        path: r.get(1)?,
        hash: r.get(2)?,
        mtime: r.get(3)?,
        language: r.get(4)?,
        size: r.get(5)?,
    })
}

fn row_symbol(r: &rusqlite::Row) -> rusqlite::Result<SymbolRow> {
    Ok(SymbolRow {
        id: r.get(0)?,
        file_id: r.get(1)?,
        name: r.get(2)?,
        kind: r.get(3)?,
        signature: r.get(4)?,
        parent: r.get(5)?,
        visibility: r.get(6)?,
        exported: r.get::<_, i64>(7)? != 0,
        start_line: r.get::<_, i64>(8)? as u32,
        end_line: r.get::<_, i64>(9)? as u32,
        start_byte: r.get(10)?,
        end_byte: r.get(11)?,
    })
}

fn row_dep(r: &rusqlite::Row) -> rusqlite::Result<DepRow> {
    Ok(DepRow {
        id: r.get(0)?,
        source_file_id: r.get(1)?,
        target_file_id: r.get(2)?,
        imported_symbol: r.get(3)?,
        dependency_type: r.get(4)?,
        source_raw: r.get(5)?,
    })
}

pub fn file_language_of(record: &FileRecord) -> Option<LanguageId> {
    record.language.as_deref().and_then(LanguageId::from_str)
}

pub fn path_language(path: &str) -> Option<LanguageId> {
    LanguageId::from_extension(
        Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or(""),
    )
}

/// Escape `%`, `_` and `\` so a user query is matched literally inside a
/// `LIKE ? ESCAPE '\'` pattern rather than acting as SQL wildcards.
pub fn escape_like(term: &str) -> String {
    let mut out = String::with_capacity(term.len());
    for c in term.chars() {
        match c {
            '%' | '_' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn like_escape_escapes_wildcards() {
        assert_eq!(escape_like("foo"), "foo");
        assert_eq!(escape_like("a_b"), "a\\_b");
        assert_eq!(escape_like("100%"), "100\\%");
        assert_eq!(escape_like("a\\b"), "a\\\\b");
        assert_eq!(escape_like("%_\\"), "\\%\\_\\\\");
    }

    #[test]
    fn files_like_matches_literal_underscore() {
        let root = tempfile_dir();
        let mut db = Database::open(&root).unwrap();
        let tx = db.begin().unwrap();
        Database::upsert_file(&tx, "src/user_service.ts", "h", 0, "typescript", 10).unwrap();
        Database::upsert_file(&tx, "src/userservice.ts", "h", 0, "typescript", 10).unwrap();
        Database::upsert_file(&tx, "src/userservice.py", "h", 0, "python", 10).unwrap();
        tx.commit().unwrap();

        // `_` must be literal: only the file with a real underscore matches.
        let hits = db.files_like("user_service", 10).unwrap();
        assert_eq!(hits.len(), 1, "literal underscore: {hits:?}");
        assert_eq!(hits[0].path, "src/user_service.ts");

        let hits2 = db.files_like("userservice", 10).unwrap();
        assert_eq!(hits2.len(), 2, "plain substring matches both: {hits2:?}");
        std::fs::remove_dir_all(&root).ok();
    }

    fn tempfile_dir() -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("ctx_db_like_{nanos}"))
    }
}
