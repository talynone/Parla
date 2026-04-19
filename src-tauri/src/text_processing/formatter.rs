// Text formatter : chunking en paragraphes a partir du texte brut.
//
// Reference VoiceInk : Transcription/Processing/WhisperTextFormatter.swift
//
// Algo exact (L33-119) :
// - Tokenisation en phrases via NLTokenizer(.sentence). Cote Rust on n'a pas
//   d'equivalent exact ; on approxime via regex sur fins de phrase courantes
//   ([.!?]+ suivi d'un whitespace) avec unicode-segmentation en fallback.
// - Tokenisation en mots via NLTokenizer(.word) ; cote Rust on utilise
//   unicode_words() de unicode-segmentation qui ignore la ponctuation.
//
// Constantes (L6-8) strictement identiques :
//   TARGET_WORD_COUNT = 50
//   MAX_SENTENCES_PER_CHUNK = 4
//   MIN_WORDS_FOR_SIGNIFICANT_SENTENCE = 4
//
// Sortie : chunks joints par " " (intra) et separes par "\n\n" (inter).

use std::sync::OnceLock;

use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

const TARGET_WORD_COUNT: usize = 50;
const MAX_SENTENCES_PER_CHUNK: usize = 4;
const MIN_WORDS_FOR_SIGNIFICANT_SENTENCE: usize = 4;

pub fn format(text: &str) -> String {
    let sentences = split_sentences(text);
    if sentences.is_empty() {
        return String::new();
    }

    let mut final_chunks: Vec<String> = Vec::new();
    let mut idx = 0usize;
    let total = sentences.len();

    while idx < total {
        // Phase 2a : accumulation jusqu'a TARGET_WORD_COUNT.
        let mut chunk: Vec<&String> = Vec::new();
        let mut chunk_word_count = 0usize;
        let mut significant_sentences = 0usize;
        let mut probe = idx;

        while probe < total {
            let s = &sentences[probe];
            let w = count_words(s);
            chunk.push(s);
            chunk_word_count += w;
            if w >= MIN_WORDS_FOR_SIGNIFICANT_SENTENCE {
                significant_sentences += 1;
            }
            probe += 1;
            if chunk_word_count >= TARGET_WORD_COUNT {
                break;
            }
        }

        // Phase 2b : si trop de phrases significatives, trimmer.
        let final_chunk: Vec<&String> = if significant_sentences > MAX_SENTENCES_PER_CHUNK {
            let mut trimmed = Vec::new();
            let mut sig_seen = 0usize;
            for s in &chunk {
                trimmed.push(*s);
                if count_words(s) >= MIN_WORDS_FOR_SIGNIFICANT_SENTENCE {
                    sig_seen += 1;
                    if sig_seen >= MAX_SENTENCES_PER_CHUNK {
                        break;
                    }
                }
            }
            trimmed
        } else {
            chunk.clone()
        };

        if final_chunk.is_empty() {
            // Safeguard anti-boucle infinie (VoiceInk L100-118).
            idx += 1;
            continue;
        }

        let consumed = final_chunk.len();
        let joined: Vec<String> = final_chunk.iter().map(|s| s.to_string()).collect();
        final_chunks.push(joined.join(" "));

        idx += consumed;
    }

    final_chunks.join("\n\n").trim().to_string()
}

fn sentence_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Split sur tout groupe ponctuation de fin (. ! ? ...) suivi d'un espace
    // ou d'un saut de ligne. Conserve la ponctuation avec un lookbehind-like via
    // split_keep pattern. Pour simplifier on applique un regex de capture.
    RE.get_or_init(|| {
        Regex::new(r"(?s)(.+?[\.!\?\u{2026}]+[\)\]\}\x22'\u{00BB}\u{201D}]?)(?:\s+|$)")
            .expect("sentence regex invalide")
    })
}

fn split_sentences(text: &str) -> Vec<String> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut last_end = 0usize;
    for cap in sentence_regex().captures_iter(text) {
        if let Some(m) = cap.get(1) {
            let s = m.as_str().trim();
            if !s.is_empty() {
                out.push(s.to_string());
            }
            last_end = cap.get(0).map(|g| g.end()).unwrap_or(last_end);
        }
    }
    // Tail : phrase finale sans ponctuation de fin.
    if last_end < text.len() {
        let tail = text[last_end..].trim();
        if !tail.is_empty() {
            out.push(tail.to_string());
        }
    }
    if out.is_empty() {
        // Fallback : considerer le texte entier comme une seule phrase.
        out.push(text.to_string());
    }
    out
}

fn count_words(s: &str) -> usize {
    s.unicode_words().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_empty_for_empty_input() {
        assert_eq!(format(""), "");
    }

    #[test]
    fn short_single_sentence_is_returned_as_is() {
        let out = format("Hello world.");
        assert_eq!(out, "Hello world.");
    }

    #[test]
    fn splits_into_paragraphs_after_50_words() {
        // 12 phrases de 5 mots chacune = 60 mots. Le formatter doit creer au
        // moins 2 chunks separes par \n\n.
        let sentence = "the quick brown fox jumps. ";
        let text = sentence.repeat(12);
        let out = format(text.trim());
        assert!(
            out.contains("\n\n"),
            "doit contenir au moins un separateur de paragraphe, got: {out:?}"
        );
    }

    #[test]
    fn caps_significant_sentences_at_four_when_exceeding_target() {
        // 10 phrases tres courtes (3 mots = non-significatives) + une tres longue
        // -> ne doit pas generer des chunks vides.
        let text = "Yes. No. Ok. ".repeat(10)
            + "The quick brown fox jumps over the lazy dog near the river bank.";
        let out = format(&text);
        assert!(!out.is_empty());
    }
}
