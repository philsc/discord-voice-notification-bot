use std::env;

use serenity::async_trait;
use serenity::framework::standard::StandardFramework;
use serenity::model::channel::{Channel, ChannelType, Message};
use serenity::model::id::ChannelId;
use serenity::model::voice::VoiceState;
use serenity::prelude::*;
use tokio::signal::unix::{signal, SignalKind};

// The bot's internal state.
#[derive(Default)]
struct BotState {
    voice_active: bool,
    channel_id: Option<ChannelId>,
}

struct BotStateKey;

// Wrapper type so we can save BotState in serenity's context.
impl TypeMapKey for BotStateKey {
    type Value = BotState;
}

struct Handler;

// Gets information about the specified voice channel.
//
// Returns the channel ID as well as the number of people in that channel.
async fn get_channel_info(ctx: &Context, voice_state: &VoiceState) -> Option<(ChannelId, usize)> {
    let id = voice_state.channel_id?;

    // Convert "ChannelId" to a "Channel".
    let channel = id
        .to_channel(ctx)
        .await
        .or_else(|why| {
            println!("Failed to get channel: {:?}", why);
            Err(why)
        })
        .ok()?;

    // Ignore all channels other than voice channels.
    let Channel::Guild(guild_channel) = channel else {
        println!("Got something other than a guild channel");
        return None;
    };
    if guild_channel.kind != ChannelType::Voice {
        println!("Got something other than a voice channel");
        return None;
    }
    println!("Got event for channel called \"{}\"", guild_channel.name());

    // Find out how many folks are in that voice channel.
    let members = guild_channel
        .members(&ctx)
        .await
        .or_else(|why| {
            println!("Failed to get member count: {:?}", why);
            return Err(why);
        })
        .ok()?;

    Some((id, members.len()))
}

#[async_trait]
impl EventHandler for Handler {
    // When the user types "~voice_notify", start announcing on the corresponding channel.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content != "~voice_notify" {
            return;
        }

        let mut data = ctx.data.write().await;
        let bot_state = data.get_mut::<BotStateKey>().unwrap();
        bot_state.channel_id = Some(msg.channel_id);

        msg.reply(&ctx, "Bot will now announce voice events to this channel!")
            .await
            .unwrap();
    }

    // Announce the first time someone joins a voice channel. When subsequent folks join, there is
    // no announcement.
    async fn voice_state_update(&self, ctx: Context, _old: Option<VoiceState>, new: VoiceState) {
        let mut data = ctx.data.write().await;
        let bot_state = data.get_mut::<BotStateKey>().unwrap();
        if bot_state.channel_id.is_none() {
            println!("Ignoring channel event because we didn't receive \"~voice_notify\" command.");
            return;
        }

        let info = get_channel_info(&ctx, &new).await;
        let (voice_channel_id, member_count) = info.unwrap_or((ChannelId::default(), 0));

        if member_count == 0 {
            // Everyone has left. Reset the state.
            bot_state.voice_active = false;
        } else if !bot_state.voice_active {
            // Someone joined the voice channel for the first time. Let folks know.
            bot_state.voice_active = true;
            let name = voice_channel_id
                .name(&ctx)
                .await
                .unwrap_or("<unknown>".to_owned());
            bot_state
                .channel_id
                .unwrap()
                .send_message(&ctx, |m| {
                    m.content(format!("Someone joined the \"{}\" voice channel.", name))
                })
                .await
                .unwrap();
        }
    }
}

// Finds the discord token via the environment.
//
// First, it checks for the DISCORD_TOKEN variable. If that is empty or doesn't exist, then it
// tries to read the token from the file specified via DISCORD_TOKEN_FILE.
async fn get_discord_token() -> String {
    let token = env::var("DISCORD_TOKEN").unwrap_or_else(|why| {
        println!("Failed to read DISCORD_TOKEN: {:?}", why);
        println!("Trying DISCORD_TOKEN_FILE.");
        "".to_owned()
    });
    if !token.is_empty() {
        return token;
    }
    let token_file = env::var("DISCORD_TOKEN_FILE").expect("Could not read DISCORD_TOKEN_FILE");
    let contents = tokio::fs::read_to_string(&token_file)
        .await
        .unwrap_or_else(|why| {
            panic!("Failed to read {}: {:?}", token_file.as_str(), why);
        });
    contents
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new();

    // Log in with a bot token from the environment.
    let token = get_discord_token().await;
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<BotStateKey>(BotState::default())
        .await
        .expect("Error creating client");

    // Deal with shutdown signals like CTRL-C cleanly.
    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        let mut hangup_stream = signal(SignalKind::hangup()).unwrap();
        let mut interrupt_stream = signal(SignalKind::interrupt()).unwrap();
        let mut terminate_stream = signal(SignalKind::terminate()).unwrap();
        tokio::select! {
            _ = hangup_stream.recv() => (),
            _ = interrupt_stream.recv() => (),
            _ = terminate_stream.recv() => (),
        }
        println!("Received shutdown signal.");
        shard_manager.lock().await.shutdown_all().await;
    });

    // Start listening for events by starting a single shard.
    println!("Starting bot.");
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    println!("Bot has shut down.");
}
