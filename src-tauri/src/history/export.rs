// Export CSV des transcriptions.
//
// Reference VoiceInk : Services/VoiceInkCSVExportService.swift. Colonnes
// dans l'ordre: Original Transcript, Enhanced Transcript, Enhancement
// Model, Prompt Name, Transcription Model, Power Mode, Enhancement Time,
// Transcription Time, Timestamp, Duration. Power Mode = "<emoji> <name>".

use crate::db::transcription::TranscriptionRecord;

pub fn to_csv(records: &[TranscriptionRecord]) -> String {
    let mut out = String::new();
    out.push_str("Original Transcript,Enhanced Transcript,Enhancement Model,Prompt Name,Transcription Model,Power Mode,Enhancement Time,Transcription Time,Timestamp,Duration\n");
    for r in records {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&r.text),
            csv_escape(r.enhanced_text.as_deref().unwrap_or("")),
            csv_escape(r.ai_enhancement_model_name.as_deref().unwrap_or("")),
            csv_escape(r.prompt_name.as_deref().unwrap_or("")),
            csv_escape(r.transcription_model_name.as_deref().unwrap_or("")),
            csv_escape(&power_mode_label(r)),
            r.enhancement_duration_sec.unwrap_or(0.0),
            r.transcription_duration_sec.unwrap_or(0.0),
            r.timestamp.to_rfc3339(),
            r.duration_sec.unwrap_or(0.0),
        ));
    }
    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

fn power_mode_label(r: &TranscriptionRecord) -> String {
    match (&r.power_mode_emoji, &r.power_mode_name) {
        (Some(e), Some(n)) => format!("{e} {n}"),
        (_, Some(n)) => n.clone(),
        _ => String::new(),
    }
}
