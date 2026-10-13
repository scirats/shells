pub(crate) const SHELLS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS shells (
    name       TEXT PRIMARY KEY,
    profile    TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;
