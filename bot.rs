use std::env;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::{Channel, ChannelType, Message};
use serenity::model::id::ChannelId;
use serenity::model::voice::VoiceState;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult};

#[group]
#[commands(ping)]
struct General;

#[derive(Default)]
struct Handler {
    voice_active: bool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        let id = match new.channel_id {
            None => return,
            Some(id) => id,
        };
        let channel = match id.to_channel(&ctx).await {
            Err(why) => {
                println!("Failed to get channel: {:?}", why);
                return;
            },
            Ok(channel) => channel,
        };
        let guild_channel = match channel {
            Channel::Guild(guild_channel) => guild_channel,
            _ => return,
        };
        if guild_channel.kind != ChannelType::Voice {
            return;
        }
        println!("Got event for channel called \"{}\"", guild_channel.name());

        let member_count = match guild_channel.member_count {
            Some(count) => count,
            _ => {
                println!("No member_count!");
                return;
            },
        };

        if member_count > 0 && !self.voice_active {
            //self.voice_active = true;
        }
    }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler::default())
        .framework(framework)
        .await
        .expect("Error creating client");

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        shard_manager.lock().await.shutdown_all().await;
    });

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    println!("Bot has shut down.");
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}
