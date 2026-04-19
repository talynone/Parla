// CRUD des mots de remplacement (dictionnaire personnel).
//
// Reference VoiceInk : Models/WordReplacement.swift L4-18.
// Champs : id (UUID), originalText (CSV comma-separated), replacementText,
// dateAdded, isEnabled.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordReplacement {
    pub id: String,
    /// CSV de variantes a matcher (separees par virgule).
    pub original_text: String,
    pub replacement_text: String,
    pub date_added: DateTime<Utc>,
    pub is_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct NewWordReplacement {
    pub original_text: String,
    pub replacement_text: String,
    #[serde(default = "default_true")]
    pub is_enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateWordReplacement {
    pub id: String,
    #[serde(default)]
    pub original_text: Option<String>,
    #[serde(default)]
    pub replacement_text: Option<String>,
    #[serde(default)]
    pub is_enabled: Option<bool>,
}

pub fn list_all(conn: &Connection) -> Result<Vec<WordReplacement>> {
    let mut stmt = conn.prepare(
        "SELECT id, original_text, replacement_text, date_added, is_enabled
         FROM word_replacements ORDER BY date_added DESC",
    )?;
    let rows = stmt.query_map([], row_to_record)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn list_enabled(conn: &Connection) -> Result<Vec<WordReplacement>> {
    let mut stmt = conn.prepare(
        "SELECT id, original_text, replacement_text, date_added, is_enabled
         FROM word_replacements WHERE is_enabled = 1 ORDER BY date_added DESC",
    )?;
    let rows = stmt.query_map([], row_to_record)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn insert(conn: &Connection, new: NewWordReplacement) -> Result<WordReplacement> {
    let record = WordReplacement {
        id: Uuid::new_v4().to_string(),
        original_text: new.original_text,
        replacement_text: new.replacement_text,
        date_added: Utc::now(),
        is_enabled: new.is_enabled,
    };
    conn.execute(
        "INSERT INTO word_replacements (id, original_text, replacement_text, date_added, is_enabled)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            record.id,
            record.original_text,
            record.replacement_text,
            record.date_added.to_rfc3339(),
            record.is_enabled as i32,
        ],
    )
    .context("insert word_replacement")?;
    Ok(record)
}

pub fn update(conn: &Connection, upd: UpdateWordReplacement) -> Result<WordReplacement> {
    let existing: WordReplacement = conn.query_row(
        "SELECT id, original_text, replacement_text, date_added, is_enabled
         FROM word_replacements WHERE id = ?1",
        [&upd.id],
        row_to_record,
    )?;

    let new_original = upd.original_text.unwrap_or(existing.original_text);
    let new_replacement = upd.replacement_text.unwrap_or(existing.replacement_text);
    let new_enabled = upd.is_enabled.unwrap_or(existing.is_enabled);

    conn.execute(
        "UPDATE word_replacements
         SET original_text = ?1, replacement_text = ?2, is_enabled = ?3
         WHERE id = ?4",
        params![
            new_original,
            new_replacement,
            new_enabled as i32,
            existing.id,
        ],
    )?;

    Ok(WordReplacement {
        id: existing.id,
        original_text: new_original,
        replacement_text: new_replacement,
        date_added: existing.date_added,
        is_enabled: new_enabled,
    })
}

pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM word_replacements WHERE id = ?1", [id])?;
    Ok(())
}

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<WordReplacement> {
    let date_str: String = row.get(3)?;
    let date_added = DateTime::parse_from_rfc3339(&date_str)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    let enabled: i32 = row.get(4)?;
    Ok(WordReplacement {
        id: row.get(0)?,
        original_text: row.get(1)?,
        replacement_text: row.get(2)?,
        date_added,
        is_enabled: enabled != 0,
    })
}
