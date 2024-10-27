use log::{debug, error, info};
use std::env::var;

use poise::serenity_prelude as serenity;
use regex::Regex;
use reqwest::Error as ReqwestError;

const PLAYER_IDS: &[&str] = &[
    "9997875",  // Kratos
    "6903668",  // Nagraj
    "1489563",  // deadmeat
    "15625569", // CVS
    "2543215",  // marathaSun
    "1228227",  // hjpotter92
];
const NIGBOT_API_URL: &str = "https://data.aoe2companion.com/api/nightbot/rank";

// Custom user data passed to all command functions
pub struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

fn generate_reply(match_id: &str) -> poise::CreateReply {
    let response = format!("Extracted Match ID: **{}**", match_id);

    let components = vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new_link(format!(
            "https://httpbin.org/redirect-to?url=aoe2de://0/{}",
            match_id
        ))
        .label("Join lobby in game")
        .style(poise::serenity_prelude::ButtonStyle::Success),
        serenity::CreateButton::new_link(format!(
            "https://httpbin.org/redirect-to?url=aoe2de://1/{}",
            match_id
        ))
        .label("Spectate match by clicking here")
        .style(poise::serenity_prelude::ButtonStyle::Primary),
        serenity::CreateButton::new_link(format!(
            "https://www.aoe2insights.com/match/{}/",
            match_id
        ))
        .label("Post-match analysis (on aoe2insights)")
        .style(poise::serenity_prelude::ButtonStyle::Secondary),
    ])];

    poise::CreateReply::default()
        .content(response)
        .components(components)
}

/// Show match information after extracting a 9-digit match ID
#[poise::command(slash_command)]
async fn match_info(
    ctx: Context<'_>,
    #[description = "aoe2 insight link, or lobby link or just plain old match id"] match_id: String,
) -> Result<(), Error> {
    // Define a regular expression to extract a 9-digit number
    let re = Regex::new(r"\b\d{9}\b").unwrap();

    // Search for the 9-digit number in the match_id string
    if let Some(captures) = re.captures(&match_id) {
        let extracted_id = &captures[0]; // This is the extracted 9-digit number
        let reply = generate_reply(extracted_id);
        ctx.send(reply).await?;
    } else {
        error!("Invalid input for match_id: {}", &match_id);
        ctx.say("No 9-digit match ID found in the input.").await?;
    }

    Ok(())
}

// Show player rank statistic from aoe companion
#[poise::command(slash_command)]
async fn rank(
    ctx: Context<'_>,
    #[description = "In-game player name to search"] player_name: String,
) -> Result<(), Error> {
    let response = read_text_from_url(format!(
        "{}?leaderboard_id=3&search={}&profile_id=12348548&flag=true",
        NIGBOT_API_URL, player_name
    ))
    .await?;
    ctx.say(response).await?;
    Ok(())
}

// Show player team-rank statistic from aoe companion
#[poise::command(slash_command)]
async fn team_rank(
    ctx: Context<'_>,
    #[description = "In-game player name to search"] player_name: String,
) -> Result<(), Error> {
    let response = read_text_from_url(format!(
        "{}?leaderboard_id=4&search={}&profile_id=12348548&flag=true",
        NIGBOT_API_URL, player_name
    ))
    .await?;
    ctx.say(response).await?;
    Ok(())
}

// Show server-local leaderboard
#[poise::command(slash_command)]
async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let user_ids: String = PLAYER_IDS.join(",");
    let response = read_text_from_url(format!(
        "https://www.aoe2insights.com/nightbot/leaderboard/3/?user_ids={}&rank=global&limit=5",
        user_ids
    ))
    .await?;
    ctx.say(
        response
            .replace("(by aoe2insights.com)", "")
            .replace(", ", "\n"),
    )
    .await?;
    Ok(())
}

/// Function to make the HTTP GET request and fetch the text
async fn read_text_from_url(url: String) -> Result<String, ReqwestError> {
    let response = reqwest::get(url).await?.text().await?;
    Ok(response)
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    let re = Regex::new(r"\b\d{9}\b").unwrap();

    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            debug!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message.content.to_lowercase().contains("aoe2de")
                && new_message.author.id != ctx.cache.current_user().id
            {
                if let Some(captures) = re.captures(&new_message.content.to_lowercase()) {
                    let extracted_id = &captures[0]; // This is the extracted 9-digit number
                    let reply = generate_reply(extracted_id);
                    info!("Received aoe2de link {}", new_message.content);
                    new_message
                        .channel_id
                        .send_message(
                            ctx,
                            poise::serenity_prelude::CreateMessage::default()
                                .content(reply.content.expect("Not message content?"))
                                .components(reply.components.expect("No components found")),
                        )
                        .await?;
                } else {
                    error!("Failed to find aoe2de link in {}", new_message.content)
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let token = var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), match_info(), rank(), team_rank(), leaderboard()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                info!("Bot is online");
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
