use std::env;
use std::io::Read;

use songbird::SerenityInit;

use serenity::{
    async_trait,
    client::{Client, EventHandler, Context},
    framework::{
        StandardFramework,
        standard::{
            Args, CommandResult,
            macros::{command, group},
        },
    },
    model::{channel::Message, gateway::Ready},
    prelude::*,
    Result as SerenityResult,
};

#[group]
#[commands(milk, join)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let mut token: String = String::new();
    let mut token_file = std::fs::File::open("token.txt").expect("token.txt not found");
    token_file.read_to_string(&mut token).unwrap();
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}

#[command]
async fn milk(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(why) = msg.channel_id.say(ctx, "Milk!").await {
        println!("Failed to send message: {:?}", why);
    }

    Ok(())
}

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild.voice_states.get(&msg.author.id).and_then(|voice_state| voice_state.channel_id);

    let channel = match channel_id {
        Some(c) => c,
        None => {
            check_msg(msg.channel_id.say(ctx, "You are not in a voice channel!").await);
            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await.expect("Songbird voice client not initialised");
    let _handler = manager.join(guild_id, channel).await;

    Ok(())
}

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}