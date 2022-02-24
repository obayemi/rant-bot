use poise::serenity_prelude as serenity;
use tracing::info;
use sqlx::postgres::{PgPoolOptions,PgPool};
use std::env;
use anyhow::anyhow;

pub struct Data {
    pool: PgPool
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>, #[flag] global: bool) -> Result<(), Error> {
    poise::builtins::register_application_commands(ctx, global).await?;
    Ok(())
}


/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            show_context_menu_commands: true,
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

async fn autocomplete_rant(
    ctx: Context<'_>,
    partial: String,
) -> impl Iterator<Item = poise::AutocompleteChoice<String>> {
    println!("thing, guild: {:?}, partial: {}", ctx.guild_id(), partial);
    // Dummy choices
    sqlx::query!(
        "SELECT name FROM rants WHERE guild_id = $1 AND name LIKE $2",
        ctx.guild_id().ok_or(anyhow!("guild only command")).unwrap().0 as i64,
        format!("%{}%", partial)
    )
    .fetch_all(&ctx.data().pool)
    .await
    .unwrap()
        .into_iter()
        .map(|rant| {
            println!("aaaa {}", rant.name);
            poise::AutocompleteChoice {
            name: rant.name.clone(),
            value: rant.name,
        }})
}

/// Rant
#[poise::command(prefix_command, slash_command)]
pub async fn rant(
    ctx: Context<'_>,
    #[description = "name"] #[autocomplete = "autocomplete_rant"] name: String,
    ) -> Result<(), Error>{
        let rant = sqlx::query!(
            "SELECT rant FROM rants WHERE guild_id = $1 AND name = $2",
            ctx.guild_id().ok_or(anyhow!("guild only command"))?.0 as i64,
            name
        )
        .fetch_optional(&ctx.data().pool)
        .await?
        .ok_or(anyhow!("rant not found"))?;

    ctx.say(rant.rant).await?;

    Ok(())
}

/// register a rant
#[poise::command(prefix_command, slash_command)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "name"] name: String,
    #[description = "rant"] rant: String,
    ) -> Result<(), Error>{
        sqlx::query!(
            "INSERT INTO rants (guild_id, name, rant, author) VALUES ($1, $2, $3, $4)
            ON CONFLICT (guild_id, name) DO UPDATE SET rant = EXCLUDED.rant RETURNING id",
            ctx.guild_id().ok_or(anyhow!("guild only command"))?.0 as i64,
            name,
            rant,
            ctx.author().id.0 as i64,
        )
        .fetch_optional(&ctx.data().pool)
        .await?
        .ok_or(anyhow!("rant registering failed"))?;

    ctx.say("rant received").await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").expect("missing `DATABASE_URL` env variable"))
        .await
        .expect("error connecting to the db");

    info!(test_value = 2, "aaa");
    poise::Framework::build()
        .token(std::env::var("DISCORD_TOKEN").unwrap())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data{pool}) }))
        .options(poise::FrameworkOptions {
            // configure framework here
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: std::env::var("DISCORD_PREFIX").ok(),
                ..Default::default()
            },
            commands : vec![
                set(),
                rant(),
                help(),
                register(),
            ],
            ..Default::default()
        })
        .run().await.unwrap();
}