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

struct Data {} // User data, which is stored and accessible in all command invocations

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
        let response = format!("Extracted Match ID: {}", extracted_id);

        let reply = {
            let components = vec![serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new_link(format!(
                    "https://httpbin.org/redirect-to?url=aoe2de://0/{}",
                    extracted_id
                ))
                .label("Join lobby in game") // Button label
                .style(poise::serenity_prelude::ButtonStyle::Success),
                serenity::CreateButton::new_link(format!(
                    "https://httpbin.org/redirect-to?url=aoe2de://1/{}",
                    extracted_id
                ))
                .label("Spectate match by clicking here") // Button label
                .style(poise::serenity_prelude::ButtonStyle::Primary),
            ])];

            poise::CreateReply::default()
                .content(response)
                .components(components)
        };
        ctx.send(reply).await?;
    } else {
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
        "https://www.aoe2insights.com/nightbot/rank/3/?query={}&default_user_id=12348548",
        player_name
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
        "https://www.aoe2insights.com/nightbot/rank/4/?query={}&default_user_id=12348548",
        player_name
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

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), match_info(), rank(), team_rank(), leaderboard()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
