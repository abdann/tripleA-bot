mod client;
use client::commands::general::MIMIC_COMMAND;
use client::commands::owner::STOP_COMMAND;
use client::database::interface::DbInterface;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{error, info};

// Manages shards. Basically use this to kill bot.
pub struct ShardManagerContainer;

// Here we endow ShardManagerContainer with the ability to be stored in a type map (for serenity contexts)
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

// Here is our event handler.
struct Handler;

// Here we implement basic functionality for event commands. TODO: Implement more functionality by calling async functions from submodules of client. (ex: storing message)
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn message(&self, ctx: Context, msg: serenity::model::channel::Message) {
        if !msg.author.bot {
            msg.channel_id
                .say(
                    &ctx.http,
                    msg.content_safe(&ctx.cache)
                        .chars()
                        .rev()
                        .collect::<String>(),
                )
                .await
                .expect("Should be able to speak");
        } else {
        }
    }
}

// Here we store commands in the appropriate groups.
#[group]
#[commands(mimic)]
struct General;

#[group]
#[owners_only]
#[commands(stop)]
struct Owner;

#[tokio::main]
async fn main() {
    // This will load the environment variables located at `./.env`, relative to
    // the CWD. See `./.env.example` for an example on how to structure this.
    dotenv::dotenv().expect("Failed to load .env file");

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to `debug`.

    let token = env::var("DISCORD_TOKEN_TEST").expect("Expected a token in the environment");
    let database_url =
        env::var("DATABASE_URL").expect("Should have DATABASE_URL present in .env file.");

    let http = Http::new(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("triple ").ignore_bots(true))
        .group(&GENERAL_GROUP)
        .group(&OWNER_GROUP);

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // This block is here to ensure that the lock is released from data after we insert the shard manager
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    // This spawns a kill switch thread to shut down the bot using CTRL+C
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    // This starts the bot, and if an error occurs it logs it to logs.
    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
