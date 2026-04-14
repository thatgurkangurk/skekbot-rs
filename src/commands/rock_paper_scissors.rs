use crate::{Context, Error};
use ::serenity::all::{CreateInteractionResponseMessage, EditMessage};
use poise::{CreateReply, serenity_prelude as serenity};
use rand::seq::IndexedRandom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RockPaperScissorsOption {
    Rock,
    Paper,
    Scissors,
}

impl RockPaperScissorsOption {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Rock => "Rock",
            Self::Paper => "Paper",
            Self::Scissors => "Scissors",
        }
    }

    fn as_emoji(self) -> serenity::ReactionType {
        match self {
            Self::Paper => serenity::ReactionType::Unicode("📄".to_string()),
            Self::Rock => serenity::ReactionType::Unicode("🪨".to_string()),
            Self::Scissors => serenity::ReactionType::Unicode("✂️".to_string()),
        }
    }

    fn as_button(self, interaction_id: u64) -> serenity::CreateButton {
        serenity::CreateButton::new(format!("{interaction_id}.{}", self.as_str()))
            .style(serenity::ButtonStyle::Primary)
            .emoji(self.as_emoji())
            .label(self.as_str())
    }

    const fn beats(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Rock, Self::Scissors)
                | (Self::Scissors, Self::Paper)
                | (Self::Paper, Self::Rock)
        )
    }

    fn get_random_option() -> Self {
        let options = [Self::Rock, Self::Paper, Self::Scissors];

        let mut rng = rand::rng();
        options.choose(&mut rng).copied().unwrap_or(Self::Rock)
    }
}

async fn announce(
    ctx: Context<'_>,
    interaction: &serenity::ComponentInteraction,
    user1: &serenity::User,
    user2: &serenity::User,
    user1_choice: RockPaperScissorsOption,
    user2_choice: RockPaperScissorsOption,
) -> Result<(), Error> {
    let outcome: Option<String>;
    let mut losers: Vec<serenity::User> = vec![];

    let mut msg = interaction.message.clone();

    // Remove buttons
    msg.edit(&ctx.http(), EditMessage::new().components(vec![]))
        .await?;

    if user1_choice == user2_choice {
        outcome = if user1 == user2 {
            Some("You played yourself and tied!".to_string())
        } else {
            Some("It's a tie!".to_string())
        };

        losers.push(user1.clone());
        if user1 != user2 {
            losers.push(user2.clone());
        }
    } else if user1_choice.beats(user2_choice) {
        outcome = if user1 == user2 {
            Some("You played yourself and won!".to_string())
        } else {
            Some(format!("{} wins!", user1.display_name()))
        };

        if user1 != user2 {
            losers.push(user2.clone());
        }
    } else {
        outcome = if user1 == user2 {
            Some("You played yourself and lost!".to_string())
        } else {
            Some(format!("{} wins!", user2.display_name()))
        };

        if user1 != user2 {
            losers.push(user1.clone());
        }
    }

    // Timeout losers
    if let Some(guild_id) = interaction.guild_id {
        for loser in losers {
            if loser.id == ctx.cache().current_user().id {
                continue;
            }

            if let Ok(mut member) = guild_id.member(&ctx.http(), loser.id).await {
                let future_time = std::time::SystemTime::now()
                    .checked_add(std::time::Duration::from_secs(60))
                    .ok_or("Overflow occurred while calculating future time")?
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs();
                let timestamp =
                    serenity::Timestamp::from_unix_timestamp(future_time.cast_signed())?;
                let result = member
                    .disable_communication_until_datetime(&ctx.http(), timestamp)
                    .await;

                match result {
                    Ok(()) => {}
                    Err(serenity::Error::Http(error)) => {
                        let status = error.status_code().ok_or("Missing status code")?;
                        println!("error: {status}");
                    }
                    Err(e) => {
                        println!("Other error: {e}");
                    }
                }
            }
        }
    }

    interaction
        .create_response(
            &ctx.http(),
            serenity::CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content(format!(
                    "{} chose {}.\n{} chose {}\n\n{}.",
                    user1,
                    user1_choice.as_str(),
                    user2,
                    user2_choice.as_str(),
                    outcome.unwrap_or_else(|| "this was unexpected... NO ONE WON".to_string())
                )),
            ),
        )
        .await?;

    Ok(())
}

#[poise::command(slash_command, rename = "rock-paper-scissors")]
/// play a game of rock paper scissors
pub async fn rock_paper_scissors(
    ctx: Context<'_>,
    #[description = "The user to play against."] against: serenity::all::User,
) -> Result<(), Error> {
    let rock_paper_scissors_id = ctx.id();

    let reply = {
        let components = vec![serenity::CreateActionRow::Buttons(vec![
            RockPaperScissorsOption::Rock.as_button(rock_paper_scissors_id),
            RockPaperScissorsOption::Paper.as_button(rock_paper_scissors_id),
            RockPaperScissorsOption::Scissors.as_button(rock_paper_scissors_id),
        ])];

        CreateReply::default()
            .content(format!(
                "{against}, {} has challenged you to a game of rock paper scissors!",
                ctx.author()
            ))
            .components(components)
    };

    ctx.send(reply).await?;

    let user_1_id = ctx.author().id;
    let user_2_id = against.id;

    let mut user_1_choice: Option<RockPaperScissorsOption> = None;
    let mut user_2_choice: Option<RockPaperScissorsOption> = None;

    while let Some(mci) = serenity::ComponentInteractionCollector::new(ctx)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .await
    {
        let choice = match mci.data.custom_id.as_str() {
            id if id.ends_with(".Rock") => Ok(RockPaperScissorsOption::Rock),
            id if id.ends_with(".Paper") => Ok(RockPaperScissorsOption::Paper),
            id if id.ends_with(".Scissors") => Ok(RockPaperScissorsOption::Scissors),
            _ => Err("not a valid option"),
        };

        let choice = match choice {
            Ok(c) => c,
            Err(msg) => {
                ctx.say(msg).await?;
                return Ok(());
            }
        };

        if mci.user.id == user_1_id {
            user_1_choice = Some(choice);
        } else if mci.user.id == user_2_id {
            user_2_choice = Some(choice);
        } else {
            mci.create_followup(
                ctx,
                serenity::CreateInteractionResponseFollowup::new()
                    .content("You aren't a part of this battle!")
                    .ephemeral(true),
            )
            .await?;
        }

        if against.bot || against.id == ctx.author().id {
            let random_option = RockPaperScissorsOption::get_random_option();

            user_2_choice = Some(random_option);
        }

        if let (Some(c1), Some(c2)) = (user_1_choice, user_2_choice) {
            announce(ctx, &mci, ctx.author(), &against, c1, c2).await?;
            break;
        }

        mci.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge)
            .await?;
    }

    Ok(())
}
