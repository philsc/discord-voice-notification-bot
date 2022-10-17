use std::env;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::{Channel, ChannelType, Message};
use serenity::model::id::ChannelId;
use serenity::model::voice::VoiceState;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult};

#[derive(Default)]
struct BotState {
    voice_active: bool,
    channel_id: Option<ChannelId>,
}

struct BotStateKey;

impl TypeMapKey for BotStateKey {
    type Value = BotState;
}

struct Handler;

async fn get_channel_member_count(ctx: &Context, voice_state: &VoiceState) -> Option<usize> {
    let id = voice_state.channel_id?;

    let channel = id.to_channel(ctx).await.or_else(|why| {
        println!("Failed to get channel: {:?}", why);
        Err(why)
    }).ok()?;

    let guild_channel = match channel {
        Channel::Guild(guild_channel) => guild_channel,
        _ => {
            println!("Got something other than a guild channel");
            return None
        },
    };
    if guild_channel.kind != ChannelType::Voice {
        println!("Got something other than a voice channel");
        return None;
    }
    println!("Got event for channel called \"{}\"", guild_channel.name());

    let member_count = match guild_channel.members(&ctx).await {
        Ok(members) => members.len(),
        Err(why) => {
            println!("Failed to get member count: {:?}", why);
            return None;
        },
    };

    Some(member_count)
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content != "~voice_notify" {
            return;
        }

        let mut data = ctx.data.write().await;
        let bot_state = data.get_mut::<BotStateKey>().unwrap();
        bot_state.channel_id = Some(msg.channel_id);

        msg.reply(&ctx, "Bot will now announce voice events to this channel!").await;
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        let mut data = ctx.data.write().await;
        let bot_state = data.get_mut::<BotStateKey>().unwrap();
        if bot_state.channel_id.is_none() {
            println!("Ignoring channel event because we didn't receive voice command.");
            return;
        }

        let member_count = get_channel_member_count(&ctx, &new).await.unwrap_or(0);

        let old_member_count = match old {
            None => 0,
            Some(voice_state) => get_channel_member_count(&ctx, &voice_state).await.unwrap_or(0),
        };

        println!("member_count: {}  old_member_count: {}", member_count, old_member_count);

        if member_count == 0 {
            bot_state.voice_active = false;
        } else if !bot_state.voice_active {
            bot_state.voice_active = true;
            bot_state.channel_id.unwrap().send_message(&ctx, |m| {
                m.content("Someone joined a voice channel.")
            }).await;
        }
    }
}

#[tokio::main]
async fn main() {
	let framework = StandardFramework::new();

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<BotStateKey>(BotState::default())
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
