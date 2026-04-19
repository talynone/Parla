// Service d'application des remplacements de mots sur le texte transcrit.
//
// Reference VoiceInk : Transcription/Processing/WordReplacementService.swift
// - CSV split des variantes originales
// - Regex \\bORIGINAL\\b case-insensitive
// - Fallback substring (case-insensitive) si l'original contient un scalaire
//   dans les plages Hiragana/Katakana/CJK/Hangul/Thai (L58-64)

use fancy_regex::Regex as FancyRegex;

use crate::db::word_replacement::WordReplacement;

/// Applique en cascade toutes les regles enabled sur `text`.
/// L'ordre est celui retourne par la DB (date desc cote list_enabled).
pub fn apply(text: &str, rules: &[WordReplacement]) -> String {
    let mut current = text.to_string();

    for rule in rules {
        if !rule.is_enabled {
            continue;
        }
        let variants: Vec<&str> = rule
            .original_text
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for original in variants {
            if original.is_empty() {
                continue;
            }
            if uses_word_boundaries(original) {
                current = replace_with_boundaries(&current, original, &rule.replacement_text);
            } else {
                current = replace_substring_ci(&current, original, &rule.replacement_text);
            }
        }
    }

    current
}

fn replace_with_boundaries(haystack: &str, needle: &str, replacement: &str) -> String {
    // Echappement + \\b...\\b case-insensitive. fancy-regex necessaire sinon OK
    // avec le crate regex standard. On utilise fancy pour garder la coherence
    // avec le filter (et pour supporter Unicode word boundaries si necessaire).
    let escaped = fancy_regex::escape(needle);
    let pattern = format!(r"(?i)\b{escaped}\b");
    match FancyRegex::new(&pattern) {
        Ok(re) => re.replace_all(haystack, replacement).into_owned(),
        Err(_) => replace_substring_ci(haystack, needle, replacement),
    }
}

/// Remplace toutes les occurrences case-insensitive de `needle` par
/// `replacement`. Implementation manuelle pour eviter d'instancier un regex.
fn replace_substring_ci(haystack: &str, needle: &str, replacement: &str) -> String {
    if needle.is_empty() {
        return haystack.to_string();
    }
    let hay_lower = haystack.to_lowercase();
    let needle_lower = needle.to_lowercase();
    let mut out = String::with_capacity(haystack.len());
    let mut cursor = 0usize;
    while let Some(rel) = hay_lower[cursor..].find(&needle_lower) {
        let start = cursor + rel;
        // Attention aux boundaries UTF-8 : `to_lowercase` peut modifier les
        // longueurs de caracteres. On verifie que start et start+needle.len()
        // sont bien des frontieres char_boundary dans l'original.
        let end = start + needle.len();
        if !haystack.is_char_boundary(start) || !haystack.is_char_boundary(end.min(haystack.len())) {
            // Fallback prudent : skip.
            cursor = start + needle.len().max(1);
            continue;
        }
        out.push_str(&haystack[cursor..start]);
        out.push_str(replacement);
        cursor = end;
    }
    out.push_str(&haystack[cursor..]);
    out
}

/// Reproduit VoiceInk WordReplacementService.usesWordBoundaries (L56-75).
/// Si un scalaire de `original` tombe dans une plage non-spacee, retourne
/// false pour declencher le fallback substring.
pub fn uses_word_boundaries(original: &str) -> bool {
    for ch in original.chars() {
        let code = ch as u32;
        let in_non_spaced = (0x3040..=0x309F).contains(&code) // Hiragana
            || (0x30A0..=0x30FF).contains(&code)              // Katakana
            || (0x4E00..=0x9FFF).contains(&code)              // CJK Unified
            || (0xAC00..=0xD7AF).contains(&code)              // Hangul Syllables
            || (0x0E00..=0x0E7F).contains(&code); // Thai
        if in_non_spaced {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn rule(id: &str, original: &str, replacement: &str) -> WordReplacement {
        WordReplacement {
            id: id.into(),
            original_text: original.into(),
            replacement_text: replacement.into(),
            date_added: Utc::now(),
            is_enabled: true,
        }
    }

    #[test]
    fn boundary_replacement_ci() {
        let rules = vec![rule("1", "docker", "Docker")];
        let out = apply("I use docker and Docker daily, docked too.", &rules);
        assert_eq!(out, "I use Docker and Docker daily, docked too.");
    }

    #[test]
    fn csv_variants() {
        let rules = vec![rule("1", "nyc, NY, new york", "New York City")];
        let out = apply("Living in NYC and NY, also new york.", &rules);
        assert!(out.contains("New York City"));
        assert!(!out.contains("NYC"));
    }

    #[test]
    fn cjk_falls_back_to_substring() {
        // Japonais : pas de word boundaries naturelles.
        let rules = vec![rule("1", "東京", "Tokyo")];
        let out = apply("私は東京が好き。東京タワー。", &rules);
        assert!(out.contains("Tokyo"));
        assert!(!out.contains("東京"));
    }

    #[test]
    fn disabled_rule_ignored() {
        let mut r = rule("1", "hello", "hi");
        r.is_enabled = false;
        let out = apply("hello world", &[r]);
        assert_eq!(out, "hello world");
    }

    #[test]
    fn uses_word_boundaries_detects_cjk() {
        assert!(!uses_word_boundaries("東京"));
        assert!(!uses_word_boundaries("안녕하세요"));
        assert!(!uses_word_boundaries("สวัสดี"));
        assert!(uses_word_boundaries("hello"));
        assert!(uses_word_boundaries("docker, k8s"));
    }
}
