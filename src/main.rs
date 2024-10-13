mod roulette;
mod search;

use std::{collections::{HashMap, HashSet}, fmt::format, io::Read, sync::Arc};

use roulette::RouletteState;
use songbird::{
    input::Input, Event, EventContext, EventHandler as VoiceEventHandler, SerenityInit, TrackEvent,
};

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            help_commands,
            macros::{command, group, help},
            Args, CommandGroup, CommandResult, HelpOptions,
        },
        StandardFramework,
    },
    http::Http,
    model::{
        channel::Message,
        gateway::Ready,
        prelude::{ChannelId, UserId},
    },
    prelude::*,
    Result as SerenityResult,
};

#[group]
#[commands(milk, join, leave, fuckoff, play, skip, queue, roll, rbet, rbets, rbalance, rclearlast, rclearall)]
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

struct SongPlayNotifier {
    artist: String,
    title: String,
    channel_id: ChannelId,
    http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for SongPlayNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        check_msg(
            self.channel_id
                .say(&self.http, format!("🎵🎵 Now playing {} - {} 🎵🎵", self.artist, self.title))
                .await,
        );
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // set the bot's prefix to "~"
        .help(&HELP)
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
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let channel = match channel_id {
        Some(c) => c,
        None => {
            check_msg(
                msg.channel_id
                    .say(ctx, "You are not in a voice channel!")
                    .await,
            );
            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird voice client not initialised");
    let handler = manager.join(guild_id, channel).await;
    let handler_lock = handler.0;
    let mut handler = handler_lock.lock().await;
    let _ = handler.deafen(true).await;
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird voice client not initialised");
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Hmmmmmm, something went wrong: {:?}", e))
                    .await,
            );
        }
        check_msg(
            msg.channel_id
                .say(&ctx.http, format!("Left channel: {}", msg.channel_id))
                .await,
        );
    } else {
        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Are you fucking retarded?! I'm not in a voice channel!"),
                )
                .await,
        );
    }

    Ok(())
}

#[command]
async fn fuckoff(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, format!("no u")).await);
    Ok(())
}

async fn get_source(ctx: &Context, msg: &Message, url: &str) -> Option<Input> {
    println!("Attempting to get source with url: {url}");
    match songbird::ytdl(url).await {
        Ok(source) => Some(source),
        Err(err) => {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Could not get song... Error:\n```\n{err}\n```"))
                    .await,
            );
            None
        }
    }
}

#[command]
#[aliases("p")]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let song = args.rest();
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let has_handler = manager.get(guild_id).is_some();
    if !has_handler {
        // We should join the voice chat if we don't have a handler yet
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);

        let channel = match channel_id {
            Some(c) => c,
            None => {
                check_msg(
                    msg.channel_id
                        .say(ctx, "❌ You are not in a voice channel!")
                        .await,
                );
                return Ok(());
            }
        };

        let _handler = manager.join(guild_id, channel).await;
    }

    check_msg(
        msg.channel_id
            .say(&ctx.http, format!("🔍 Searching for song \"{}\"", song))
            .await,
    );
    if let Some(url) = search::search(song).await {
        if let Some(source) = get_source(ctx, msg, &url).await {
            if let Some(handler_lock) = manager.get(guild_id) {
                let mut handler = handler_lock.lock().await;
                if !handler.is_deaf() {
                    let _ = handler.deafen(true).await;
                }
                // handler.play_source(source);
                let track_handle = handler.enqueue_source(source);
                let _ = track_handle.add_event(
                    Event::Track(TrackEvent::Play),
                    SongPlayNotifier {
                        artist: track_handle.metadata().artist.clone().unwrap_or("Unknown Artist".into()),
                        title: track_handle.metadata().title.clone().unwrap_or("Unknown Title".into()),
                        channel_id: msg.channel_id,
                        http: ctx.http.clone(),
                    },
                );
                check_msg(
                    msg.channel_id
                        .say(&ctx.http, format!("💿 Queued {}", track_handle.metadata().title.clone().unwrap_or("Unknown Title".into())))
                        .await,
                );
            } else {
                check_msg(
                    msg.channel_id
                        .say(&ctx.http, "❌ Not in a voice channel")
                        .await,
                );
            }
        }
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, format!("❌ Couldn't find song"))
                .await,
        );
        return Ok(());
    }

    Ok(())
}

#[help]
async fn help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[command]
#[aliases("s")]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let queue = handler.queue();
        if let Some(current_track) = queue.current() {
            if let Some(song_name) = &current_track.metadata().title {
                if let Ok(_) = queue.skip() {
                    reply(&ctx, &msg, format!("→ Skipped {}", song_name)).await;
                }
            }
        } else {
            reply(&ctx, &msg, "❌ No song to skip").await;
        }
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "❌ Not in a voice channel")
                .await,
        );
    }

    Ok(())
}

#[command]
#[aliases("q")]
#[only_in(guilds)]
async fn queue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let queue = handler.queue();
        let mut response = String::new();
        for (pos, track) in queue.current_queue().iter().enumerate() {
            let title = track
                .metadata()
                .title
                .clone()
                .unwrap_or("Unknown title".into());
            let artist = track
                .metadata()
                .artist
                .clone()
                .unwrap_or("Unknown artist".into());
            response += &format!("\n  {}) [{}] {}", pos + 1, &artist, &title);
        }
        reply(ctx, msg, format!("💿💿 Songs in queue 💿💿{}", response)).await;
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "❌ Not in a voice channel")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let roll_expression = args.rest();
    if let Ok(result) = d20::roll_dice(roll_expression) {
        reply(&ctx, &msg, format!("You rolled: {}", result.to_string())).await;
    } else {
        reply(&ctx, &msg, format!("Invalid roll expression")).await;
    }
    Ok(())
}

struct RouletteData {
    channel_state: HashMap<ChannelId, Arc<Mutex<RouletteState<UserId>>>>
}

impl TypeMapKey for RouletteData {
    type Value = RouletteData;
}

#[command]
#[only_in(guilds)]
async fn rbet(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut ctx_data = ctx.data.write().await;
    let roulette_data = ctx_data.entry::<RouletteData>().or_insert(RouletteData{ channel_state: Default::default() });
    let roulette_state = roulette_data.channel_state.entry(msg.channel_id).or_insert(
        Arc::new(Mutex::new(RouletteState::new()))
    );
    let player_id = msg.author.id;
    let player_name = msg.author.name.clone();
    let mut roulette_state_mut = roulette_state.lock().await;
    roulette_state_mut.register_player(player_id, &player_name);
    let current_balance = roulette_state_mut.get_balance(player_id);
    if let Ok(current_balance) = current_balance {
        if let Ok(current_bets) = roulette_state_mut.get_bets(player_id) {
            if current_balance == 0 && current_bets.len() == 0 {
                let set_money_result = roulette_state_mut.set_balance(player_id, 5);
                if set_money_result.is_ok() {
                    reply(ctx, msg, format!("```\nYou seem broke. Since I feel sorry for you, have 5 money units...\n```")).await;
                }
            }
        }
    }
    let bet_result = roulette_state_mut.play_bet_command(player_id, args.rest());
    if let Err(e) = bet_result {
        reply(ctx, msg, format!("Betting failed:\n{}", e.to_string())).await;
    }
    let current_bets = roulette_state_mut.get_bets(player_id);
    match current_bets {
        Ok(current_bets) => {
            let bets: Vec<String> = current_bets.into_iter().map(|bet| format!("- {bet:?}")).collect();
            reply(ctx, msg, format!("Current bets:\n```\n{}\n```", bets.join("\n"))).await;
        },
        Err(e) => {
            reply(ctx, msg, format!("Unable to get current bets: {}", e.to_string())).await;
        }
    }
    let current_balance = roulette_state_mut.get_balance(player_id);
    match current_balance {
        Ok(current_balance) => {
            reply(ctx, msg, format!("```\nYour new balance is {current_balance}\n```")).await;
        },
        Err(e) => {
            reply(ctx, msg, format!("Unable to get current balance: {}", e.to_string())).await;
        }
    }

    if !roulette_state_mut.spin_scheduled {
        roulette_state_mut.spin_scheduled = true;
        let _ = msg.channel_id.say(ctx, "```\nWheel will stop spinning in 25 seconds. Place your bets!\n```").await;
        let http = ctx.http.clone();
        let roulette_state = roulette_state.clone();
        let channel_id = msg.channel_id.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
            let _ = channel_id.say(&http, "```\nWheel will stop spinning in 10 seconds. Finalize your bets!\n```").await;
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            let mut roulette_state = roulette_state.lock().await;
            roulette_state.lock_bets();
            let _ = channel_id.say(&http, "Bets have been finalized!").await;
            let spin_result = roulette_state.spin();
            let payouts: Vec<String> = spin_result.payouts.into_iter().map(
                |(player_id, (payout, balance))| format!("- {player_id}: {payout} (new balance: {balance})")
            ).collect();
            let payouts = payouts.join("\n");
            let color = match spin_result.result {
                0 => "Green",
                _ => {
                    if roulette::is_red(spin_result.result) {
                        "Red"
                    }
                    else {
                        "Black"
                    }
                }
            };
            let _ = channel_id.say(&http, format!("**Landed on {}** (**{color}**)\nPayouts:\n```\n{payouts}\n```", spin_result.result)).await;
        });
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn rclearlast(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut ctx_data = ctx.data.write().await;
    let roulette_data = ctx_data.entry::<RouletteData>().or_insert(RouletteData{ channel_state: Default::default() });
    let roulette_state = roulette_data.channel_state.entry(msg.channel_id).or_insert(
        Arc::new(Mutex::new(RouletteState::new()))
    );
    let player_id = msg.author.id;
    let player_name = msg.author.name.clone();
    let mut roulette_state_mut = roulette_state.lock().await;
    roulette_state_mut.register_player(player_id, &player_name);
    let bet_result = roulette_state_mut.clear_last_bet(player_id);
    if let Err(e) = bet_result {
        reply(ctx, msg, format!("```\nUnable to clear last bet:\n{}\n```", e.to_string())).await;
    }
    let current_bets = roulette_state_mut.get_bets(player_id);
    match current_bets {
        Ok(current_bets) => {
            let bets: Vec<String> = current_bets.into_iter().map(|bet| format!("- {bet:?}")).collect();
            reply(ctx, msg, format!("Current bets:\n```\n{}\n```", bets.join("\n"))).await;
        },
        Err(e) => {
            reply(ctx, msg, format!("Unable to get current bets: {}", e.to_string())).await;
        }
    }
    let current_balance = roulette_state_mut.get_balance(player_id);
    match current_balance {
        Ok(current_balance) => {
            reply(ctx, msg, format!("```\nYour new balance is {current_balance}\n```")).await;
        },
        Err(e) => {
            reply(ctx, msg, format!("Unable to get current balance: {}", e.to_string())).await;
        }
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn rclearall(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut ctx_data = ctx.data.write().await;
    let roulette_data = ctx_data.entry::<RouletteData>().or_insert(RouletteData{ channel_state: Default::default() });
    let roulette_state = roulette_data.channel_state.entry(msg.channel_id).or_insert(
        Arc::new(Mutex::new(RouletteState::new()))
    );
    let player_id = msg.author.id;
    let player_name = msg.author.name.clone();
    let mut roulette_state_mut = roulette_state.lock().await;
    roulette_state_mut.register_player(player_id, &player_name);
    let bet_result = roulette_state_mut.clear_all_bets(player_id);
    if let Err(e) = bet_result {
        reply(ctx, msg, format!("```\nUnable to clear all bets:\n{}\n```", e.to_string())).await;
    }
    let current_bets = roulette_state_mut.get_bets(player_id);
    match current_bets {
        Ok(current_bets) => {
            let bets: Vec<String> = current_bets.into_iter().map(|bet| format!("- {bet:?}")).collect();
            reply(ctx, msg, format!("Current bets:\n```\n{}\n```", bets.join("\n"))).await;
        },
        Err(e) => {
            reply(ctx, msg, format!("Unable to get current bets: {}", e.to_string())).await;
        }
    }
    let current_balance = roulette_state_mut.get_balance(player_id);
    match current_balance {
        Ok(current_balance) => {
            reply(ctx, msg, format!("```\nYour new balance is {current_balance}\n```")).await;
        },
        Err(e) => {
            reply(ctx, msg, format!("Unable to get current balance: {}", e.to_string())).await;
        }
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn rbalance(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut ctx_data = ctx.data.write().await;
    let roulette_data = ctx_data.entry::<RouletteData>().or_insert(RouletteData{ channel_state: Default::default() });
    let roulette_state = roulette_data.channel_state.entry(msg.channel_id).or_insert(
        Arc::new(Mutex::new(RouletteState::new()))
    );
    let player_id = msg.author.id;
    let player_name = msg.author.name.clone();
    let mut roulette_state_mut = roulette_state.lock().await;
    roulette_state_mut.register_player(player_id, &player_name);
    let current_balance = roulette_state_mut.get_balance(player_id);
    match current_balance {
        Ok(current_balance) => {
            reply(ctx, msg, format!("```\nYour balance is {current_balance}\n```")).await;
        },
        Err(e) => {
            reply(ctx, msg, format!("Unable to get current balance: {}", e.to_string())).await;
        }
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn rbets(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut ctx_data = ctx.data.write().await;
    let roulette_data = ctx_data.entry::<RouletteData>().or_insert(RouletteData{ channel_state: Default::default() });
    let roulette_state = roulette_data.channel_state.entry(msg.channel_id).or_insert(
        Arc::new(Mutex::new(RouletteState::new()))
    );
    let player_id = msg.author.id;
    let player_name = msg.author.name.clone();
    let mut roulette_state_mut = roulette_state.lock().await;
    roulette_state_mut.register_player(player_id, &player_name);
    let current_bets = roulette_state_mut.get_bets(player_id);
    match current_bets {
        Ok(current_bets) => {
            let bets: Vec<String> = current_bets.into_iter().map(|bet| format!("- {bet:?}")).collect();
            reply(ctx, msg, format!("Current bets:\n```\n{}\n```", bets.join("\n"))).await;
        },
        Err(e) => {
            reply(ctx, msg, format!("Unable to get current bets: {}", e.to_string())).await;
        }
    }
    Ok(())
}

async fn reply(ctx: &Context, msg: &Message, response: impl Into<String>) {
    check_msg(msg.reply(&ctx.http, response.into()).await);
}

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
