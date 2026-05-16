use nlprule::{Tokenizer, tokenizer_filename};
use phf::{phf_map, phf_set};
use poise::serenity_prelude as serenity;
use regex::Regex;
use std::sync::LazyLock;

static ALLOWED_ABBREVIATIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "perms" => "permissions",
    "msgs" => "messages",
    "imgs" => "images",
};

static PHRASE_BREAKERS: phf::Set<&'static str> = phf_set! {
    "i", "you", "he", "she", "it", "we", "they", "me", "him", "us", "them",
    "is", "are", "was", "were", "am", "be", "been", "being",
    "do", "does", "did", "have", "has", "had",
    "can", "could", "shall", "should", "will", "would", "may", "might", "must",
    "and", "but", "or", "so", "because", "if", "then", "than", "why", "how", "what", "where",
    "in", "on", "at", "to", "from", "with", "about", "for", "of", "by",
    "think", "know", "say", "said", "make", "made", "go", "went", "take", "took", "want", "got", "get",
    "love", "like", "hate", "see", "saw", "look", "looks", "need", "needs",
};

#[allow(clippy::unwrap_used)]
static ARTICLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(a|an|the)\b").unwrap());

static TOKENIZER_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/", tokenizer_filename!("en")));

#[allow(clippy::expect_used)]
pub static NLP: LazyLock<Tokenizer> = LazyLock::new(|| {
    Tokenizer::from_reader(std::io::Cursor::new(TOKENIZER_BYTES))
        .expect("Failed to load embedded NLP model")
});

struct Phrase {
    text: String,
    plural: bool,
}

pub fn extract_nouns_with_correct_verb(text: &str) -> Option<String> {
    let mut phrases: Vec<Phrase> = Vec::new();
    let mut current_words: Vec<String> = Vec::new();

    let mut is_plural = false;
    let mut has_noun = false;

    let sentences = NLP.pipe(text);

    let mut flush = |words: &mut Vec<String>, plural: &mut bool, noun: &mut bool| {
        // only keep the phrase if it actually contained a noun (not just adjectives)
        if !words.is_empty() && *noun {
            phrases.push(Phrase {
                text: words.join(" "),
                plural: *plural,
            });
        }
        words.clear();
        *plural = false;
        *noun = false;
    };

    for sentence in sentences {
        for token in sentence.tokens() {
            let word = token.word().text().as_str().trim();
            if word.is_empty() {
                continue;
            }

            let lower_word = word.to_lowercase();

            // actively break on words that act as verbs/pronouns
            if PHRASE_BREAKERS.contains(lower_word.as_str()) {
                flush(&mut current_words, &mut is_plural, &mut has_noun);
                continue;
            }

            let primary_tag = token.word().tags().first().map_or("", |t| t.pos().as_str());

            let is_nn = primary_tag.starts_with("NN");
            let is_jj = primary_tag.starts_with("JJ");

            if is_nn || is_jj {
                current_words.push(word.to_string());

                if is_nn {
                    has_noun = true;
                    is_plural = primary_tag == "NNS" || primary_tag == "NNPS";
                }
            } else {
                flush(&mut current_words, &mut is_plural, &mut has_noun);
            }
        }
    }

    flush(&mut current_words, &mut is_plural, &mut has_noun);

    // only target the last valid noun phrase to make the joke punchy
    let target_phrase = phrases.last()?;

    let cleaned = ARTICLE_REGEX
        .replace_all(&target_phrase.text, "")
        .trim()
        .to_string();
    if cleaned.is_empty() {
        return None;
    }

    let words: Vec<&str> = cleaned.split_whitespace().collect();
    let last_word = *words.last().unwrap_or(&cleaned.as_str());
    let head_lower = last_word.to_lowercase();

    let final_head = ALLOWED_ABBREVIATIONS
        .get(head_lower.as_str())
        .copied()
        .unwrap_or(last_word);

    let mut resolved = cleaned.clone();
    if let Some(idx) = resolved.rfind(last_word) {
        resolved.replace_range(idx.., final_head);
    }

    Some(format!(
        "{} {}",
        resolved,
        if target_phrase.plural { "are" } else { "is" }
    ))
}