// Filtre post-transcription : supprime les blocs <TAG>, les brackets [()], les
// filler words (si actif) et normalise les espaces.
//
// Reference VoiceInk : Transcription/Processing/TranscriptionOutputFilter.swift
// Ordre exact (L14-44) :
//  1. Blocs TAG : `<([A-Za-z][A-Za-z0-9:_-]*)[^>]*>[\s\S]*?</\1>`
//  2. Brackets : `\[.*?\]`, `\(.*?\)`, `\{.*?\}`
//  3. Filler words (conditionnel) : `\bWORD\b[,.]?` case-insensitive
//  4. Collapse `\s{2,}` vers un espace unique
//  5. Trim final

use std::sync::OnceLock;

use regex::Regex;

/// Regex des blocs de type XML/HTML avec backreference sur la balise.
/// `regex` standard de Rust ne supporte pas les backreferences, on utilise
/// `fancy-regex` uniquement pour ce pattern.
fn tag_regex() -> &'static fancy_regex::Regex {
    static RE: OnceLock<fancy_regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        fancy_regex::Regex::new(r"(?s)<([A-Za-z][A-Za-z0-9:_-]*)[^>]*>.*?</\1>")
            .expect("tag regex invalide")
    })
}

/// Regex des contenus entre brackets (comme VoiceInk L7-11).
fn bracket_regexes() -> &'static [Regex; 3] {
    static RE: OnceLock<[Regex; 3]> = OnceLock::new();
    RE.get_or_init(|| {
        [
            Regex::new(r"\[.*?\]").unwrap(),
            Regex::new(r"\(.*?\)").unwrap(),
            Regex::new(r"\{.*?\}").unwrap(),
        ]
    })
}

fn whitespace_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\s{2,}").unwrap())
}

/// Cache les regex filler pour ne pas les recompiler a chaque transcription.
/// Invalidation simple : on recompile si la liste change.
static FILLER_CACHE: OnceLock<parking_lot::Mutex<Option<FillerCache>>> = OnceLock::new();

struct FillerCache {
    words: Vec<String>,
    regexes: Vec<Regex>,
}

/// Applique le filtre complet. `filler_words` vide = skip cette etape (equivalent
/// a RemoveFillerWords=false cote VoiceInk).
pub fn filter(text: &str, filler_words: &[String]) -> String {
    let mut out = text.to_string();

    // 1. Blocs TAG complets (fancy-regex pour le backreference \1).
    out = tag_regex().replace_all(&out, "").into_owned();
    // fancy-regex peut laisser des blocs imbriques ou attributs, une seconde passe
    // retire les balises orphelines eventuelles.
    let orphan_tags = Regex::new(r"</?[A-Za-z][A-Za-z0-9:_-]*[^>]*>").unwrap();
    out = orphan_tags.replace_all(&out, "").into_owned();

    // 2. Brackets
    for re in bracket_regexes() {
        out = re.replace_all(&out, "").into_owned();
    }

    // 3. Filler words (conditionnel)
    if !filler_words.is_empty() {
        let regexes = filler_regexes(filler_words);
        for re in &regexes {
            out = re.replace_all(&out, "").into_owned();
        }
    }

    // 4. Collapse whitespace
    out = whitespace_regex().replace_all(&out, " ").into_owned();

    // 5. Trim
    out.trim().to_string()
}

fn filler_regexes(words: &[String]) -> Vec<Regex> {
    let slot = FILLER_CACHE.get_or_init(|| parking_lot::Mutex::new(None));
    let mut guard = slot.lock();
    if let Some(cache) = guard.as_ref() {
        if cache.words == words {
            return cache.regexes.clone();
        }
    }
    let regexes: Vec<Regex> = words
        .iter()
        .filter_map(|w| {
            let escaped = regex::escape(w);
            Regex::new(&format!(r"(?i)\b{escaped}\b[,.]?")).ok()
        })
        .collect();
    *guard = Some(FillerCache {
        words: words.to_vec(),
        regexes: regexes.clone(),
    });
    regexes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_tag_blocks() {
        let out = filter("hello <music>lalala</music> world", &[]);
        assert_eq!(out, "hello world");
    }

    #[test]
    fn strips_brackets() {
        let out = filter("hello [noise] (laughter) {silence} world", &[]);
        assert_eq!(out, "hello world");
    }

    #[test]
    fn removes_filler_words_case_insensitive() {
        let fillers = vec!["uh".to_string(), "um".to_string()];
        let out = filter("Uh, hello. Um world.", &fillers);
        // Les commas/points adjacents sont aussi consommes par le regex [,.]?
        assert_eq!(out, "hello. world.");
    }

    #[test]
    fn collapses_whitespace() {
        let out = filter("hello    world\t\n  ", &[]);
        assert_eq!(out, "hello world");
    }

    #[test]
    fn empty_input() {
        assert_eq!(filter("", &[]), "");
    }
}
