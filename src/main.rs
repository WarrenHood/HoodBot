mod search;

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
            macros::{command, group, hook},
        },
    },
    model::{channel::Message, gateway::Ready},
    prelude::*,
    Result as SerenityResult,
};

#[group]
#[commands(milk, join, leave, fuckoff, play)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.to_lowercase().contains("cringe") {
          if let Err(why) = msg.reply(&ctx.http, "Gay").await {
              println!("Error sending message: {:?}", why);
        }
      }
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

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await.expect("Songbird voice client not initialised");
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Hmmmmmm, something went wrong: {:?}", e)).await);
        }
        check_msg(msg.channel_id.say(&ctx.http, format!("Left channel: {}", msg.channel_id)).await);
    }
    else {
        check_msg(msg.channel_id.say(&ctx.http, format!("Are you fucking retarded?! I'm not in a voice channel!")).await);
    }

    Ok(())
}

#[command]
async fn fuckoff(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, format!("no u")).await);

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let song = args.rest();
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        check_msg(msg.channel_id.say(&ctx.http, format!("Searching for {}", song)).await);

        if let Some(url) = search::search(song).await {
            // check_msg(msg.channel_id.say(&ctx.http, format!("Found song: {}... What? You think I'm going to actually play it?", url)).await);
            
            let source = match songbird::ytdl(&url).await {
                Ok(source) => source,
                Err(why) => {
                    println!("Error starting source: {:?}", why);
    
                    check_msg(msg.channel_id.say(&ctx.http, "Hmmmm, something went wrong with ffmpeg. Oh well...").await);
    
                    return Ok(());
                },
            };
    
            handler.play_source(source);
    
            check_msg(msg.channel_id.say(&ctx.http, format!("Playing {}", song)).await);
        }
        else {
            check_msg(msg.channel_id.say(&ctx.http, format!("Couldn't find song")).await);
            return Ok(())
        }
    } 
    else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel").await);
    }
    
    Ok(())
}


#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("Message is not a command '{}'", msg.content);
}


fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}