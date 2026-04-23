use ::serenity::all::{EditMember, Timestamp};
use nlprule::{Tokenizer, tokenizer_filename};
use phf::{phf_map, phf_set};
use poise::serenity_prelude as serenity;
use rand::RngExt;
use regex::Regex;
use serenity::all::{CreateAllowedMentions, CreateMessage, UserId};
use std::sync::LazyLock;
use tracing::{error, info};

use crate::{Data, Error};

#[allow(clippy::unreadable_literal)]
// this is a discord user id so its FINE.
const HIDDEN_USER_ID: UserId = UserId::new(475851244737396740);

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

#[allow(clippy::unwrap_used)]
static MENTIONS_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<@!?\d+>").unwrap());

static TOKENIZER_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/", tokenizer_filename!("en")));

#[allow(clippy::expect_used)]
static NLP: LazyLock<Tokenizer> = LazyLock::new(|| {
    Tokenizer::from_reader(std::io::Cursor::new(TOKENIZER_BYTES))
        .expect("Failed to load embedded NLP model")
});

struct Phrase {
    text: String,
    plural: bool,
}

fn extract_nouns_with_correct_verb(text: &str) -> Option<String> {
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

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    if let serenity::FullEvent::Ready { data_about_bot: _ } = event {
        info!("warming up the nlp engine...");
        let _ = NLP.pipe("warmup");
        info!("nlp engine loaded. ready to annoy hidden teehee.");
    }

    if let serenity::FullEvent::Message { new_message } = event {
        if new_message.author.bot {
            return Ok(());
        }
        if new_message.author.id != HIDDEN_USER_ID {
            return Ok(());
        }

        // let should_timeout = {
        //     let mut rng = rand::rng();
        //     rng.random_bool(0.45)
        // };

        // if should_timeout {
        //     #[allow(clippy::expect_used)]
        //     let timeout_until =
        //         Timestamp::from_unix_timestamp(Timestamp::now().unix_timestamp() + 300)
        //             .expect("Invalid timestamp");

        //     if let Some(guild_id) = new_message.guild_id {
        //         let builder = EditMember::new().disable_communication_until_datetime(timeout_until);

        //         if let Err(why) = guild_id
        //             .edit_member(&ctx.http, &HIDDEN_USER_ID, builder)
        //             .await
        //         {
        //             error!("Error timing out user: {why:?}");
        //         } else {
        //             info!("User {HIDDEN_USER_ID} has been timed out.");
        //         }
        //     }
        // }

        let should_reply = {
            let mut rng = rand::rng();
            rng.random_bool(0.7)
        };

        if !should_reply {
            return Ok(());
        }

        let content = &new_message.content;
        if content.len() < 3 {
            return Ok(());
        }

        let sanitised_content = MENTIONS_REGEX.replace_all(content, "").into_owned();

        let Some(noun) = extract_nouns_with_correct_verb(&sanitised_content) else {
            return Ok(());
        };

        let reply_content = format!("maybe the {noun} hidden");

        let mentions_builder = CreateAllowedMentions::new().empty_roles().empty_roles();
        let message_builder = CreateMessage::new()
            .content(reply_content)
            .reference_message(new_message)
            .allowed_mentions(mentions_builder);

        let _ = new_message
            .channel_id
            .send_message(&ctx.http, message_builder)
            .await;
    }
    Ok(())
}
