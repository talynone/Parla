// Commandes Tauri pour le dictionnaire personnel (word replacements).

use tauri::State;

use crate::db::word_replacement::{
    self as repo, NewWordReplacement, UpdateWordReplacement, WordReplacement,
};
use crate::db::Database;

#[tauri::command]
pub fn list_word_replacements(db: State<'_, Database>) -> Result<Vec<WordReplacement>, String> {
    let conn = db.0.lock();
    repo::list_all(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_word_replacement(
    db: State<'_, Database>,
    payload: NewWordReplacement,
) -> Result<WordReplacement, String> {
    let conn = db.0.lock();
    repo::insert(&conn, payload).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_word_replacement(
    db: State<'_, Database>,
    payload: UpdateWordReplacement,
) -> Result<WordReplacement, String> {
    let conn = db.0.lock();
    repo::update(&conn, payload).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_word_replacement(db: State<'_, Database>, id: String) -> Result<(), String> {
    let conn = db.0.lock();
    repo::delete(&conn, &id).map_err(|e| e.to_string())
}
