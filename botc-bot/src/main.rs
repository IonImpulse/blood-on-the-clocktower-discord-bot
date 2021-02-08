use tokio;
use tokio::sync::Mutex;

use lazy_static::lazy_static;

use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::*;
use serenity::model::guild::*;
use serenity::model::id::*;

use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    http::Http,
    model::{event::ResumedEvent, gateway::Ready, id::RoleId},
    prelude::*,
};

use std::{collections::*, env, sync::Arc};

use colored::*;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

async fn is_storyteller(ctx: &Context, msg: &Message) -> bool {
    let roles: &Vec<RoleId> = &msg.member.as_ref().unwrap().roles;
    let mut storyteller: bool = false;

    for role in roles {
        if role.to_role_cached(&ctx.cache).await.as_ref().is_some() {
            if role
                .to_role_cached(&ctx.cache)
                .await
                .as_ref()
                .unwrap()
                .name
                .to_lowercase()
                .contains("storytell")
            {
                storyteller = true;
            }
        }
    }

    return storyteller;
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        print_status(&format!("Connected as {}", ready.user.name));
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        print_info("Resumed");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // First, check to see if the message is a command. If it's not, discard
        if !msg.content.starts_with("~") {
            // Next, check if this was a message in a guild or not
            let message_in_guild: bool = msg.member.as_ref().is_some();

            if message_in_guild {
                // Next, check if this sent by a storyteller, as only they can use commands
                if is_storyteller(&ctx, &msg).await {
                    // Print the message to console
                    print_echo(&msg);

                    // Get guild ID
                    let current_guild_id = msg.guild_id.as_ref().unwrap().as_u64();

                    // Create a variable to hold the current state of everything to check against
                    let state = BLOOD_DATABASE.lock().await;

                    // Has this game been setup before? If it hasn't, ignore it.
                    if state.blood_guilds.contains_key(current_guild_id) {
                        // Get channel ID
                        let current_channel_id = msg.channel_id.as_u64();

                        // Was this sent in the correct channel? If it hasn't, ignore it
                        if &state.blood_guilds[current_guild_id].storyteller_channel
                            == current_channel_id
                        {
                            // Check if in the middle of setting roles
                            let is_rolling;

                            match state.blood_guilds[current_guild_id].game_state {
                                GameState::SettingRoles => is_rolling = true,
                                _ => is_rolling = false,
                            }

                            drop(state);

                            if is_rolling {
                                roles(&ctx, &msg).await;
                            } else {
                                match msg.content.as_str() {
                                    "roles" => roles(&ctx, &msg).await,
                                    "dm roles" => dm_roles(&ctx, &msg).await,
                                    "night" => night(&ctx, &msg).await,
                                    "sleep" => night(&ctx, &msg).await,
                                    "day" => day(&ctx, &msg).await,
                                    "wake" => day(&ctx, &msg).await,
                                    "save" => save(&ctx, &msg).await,
                                    _ => nothing(&ctx, &msg).await,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[group]
#[commands(start, end)]
struct General;

// Different coloured print functions
// Just for cosmetic purposes, but it does look very nice

fn print_info(string: &str) {
    println!("{}    | {}", "INFO".green().bold(), string.normal());
}

fn print_status(string: &str) {
    println!("{}  | {}", "STATUS".cyan().bold(), string.normal());
}

fn print_echo(msg: &Message) {
    let message: String = String::from(&msg.content);
    let mut author_name: String = String::from(&msg.author.name);

    let author_member_in_guild: bool = msg.member.as_ref().is_some();

    if author_member_in_guild {
        let author_member_has_nick: bool = msg.member.as_ref().unwrap().nick.is_some();

        if author_member_has_nick {
            author_name = String::from(msg.member.as_ref().unwrap().nick.as_ref().unwrap());
        }
    }

    println!(
        "{}    | {} : {}",
        "ECHO".blue().bold(),
        author_name.bold(),
        message.normal()
    );
}

fn print_command(msg: &Message) {
    println!(
        "{} | [{}] | {}#{}",
        "COMMAND".yellow().bold(),
        &msg.content.purple(),
        &msg.author.name,
        &msg.author.discriminator
    );
}

fn print_error(msg: &str) {
    println!("{} | {}", "ERROR".red().bold(), msg);
}

// Function to send a message to a channel safely
async fn send_msg(msg: &Message, ctx: &Context, content: String) {
    if let Err(why) = &msg.channel_id.say(&ctx.http, content).await {
        print_error(&format!("Could not send message: {:?}", why));
    }
}

// Here are the custom enums and structs for each server
// Each server has a BloodGuild struct assigned to it in order to keep
// track of the game as it goes on, and is saved to a shared async
// dictionary at each GameState change

#[derive(Copy, Clone)]
pub enum GameState {
    Nothing,
    SettingUp,
    SettingRoles,
    Playing,
}

#[derive(Clone)]
pub struct BloodGuild {
    id: u64,
    game_state: GameState,
    storyteller_channel: u64,
    roles: Vec<(u64, Member, String)>,
    room_assignments: HashMap<u64, ChannelId>,
}

// Global HashMap struct to hold all global data
#[derive(Clone)]
pub struct GlobalBloodState {
    blood_guilds: HashMap<u64, BloodGuild>,
}

impl BloodGuild {
    /// Create a new, empty, instance of "BloodGuild".
    fn new(id: u64, storyteller_channel: u64) -> Self {
        BloodGuild {
            id: id,
            game_state: GameState::SettingUp,
            storyteller_channel: storyteller_channel,
            roles: Vec::new(),
            room_assignments: HashMap::new(),
        }
    }
}

impl GlobalBloodState {
    /// Create a new, empty, instance of "GlobalBloodState".
    fn new() -> Self {
        GlobalBloodState {
            blood_guilds: HashMap::new(),
        }
    }
}

// GLOBAL database variable
// Not the best way of doing this but it's hard with serenity
// as functions are called with no easy way to pass
// a main database in the function
lazy_static! {
    static ref BLOOD_DATABASE: Arc<Mutex<GlobalBloodState>> =
        Arc::new(Mutex::new(GlobalBloodState::new()));
}

#[tokio::main]
async fn main() {
    println!("=======================================");
    println!("");

    print_info("Starting up...");

    // Setup the async hashmap to store BloodGuild structs

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("BLOOD_TOKEN")
    .expect("Please set your BLOOD_TOKEN! Follow instructions at https://github.com/IonImpulse/blood-on-the-clocktower-discord-bot!");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    print_info("Started!");
    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn start(ctx: &Context, msg: &Message) -> CommandResult {
    if is_storyteller(&ctx, &msg).await {
        print_command(&msg);

        let is_guild: bool = msg.guild_id.as_ref().is_some();

        if is_guild {
            let guild_id = msg.guild_id.as_ref().unwrap().as_u64();

            let channel_id: &u64 = msg.channel_id.as_u64();

            print_status(&format!("Setting up new server with id [{}]", guild_id));

            let content =
                String::from("**New game has been created!** Now bound to this channel...");

            send_msg(&msg, &ctx, content).await;

            let content = String::from(
                "**Type \"roles\" to start assigning roles once everyone is in voice chat!**",
            );

            send_msg(&msg, &ctx, content).await;

            let temp_server = BloodGuild::new(*guild_id, *channel_id);

            // Start accesssing main database with lock
            let mut lock = BLOOD_DATABASE.lock().await;

            lock.blood_guilds.insert(*guild_id, temp_server);

            let num_servers = lock.blood_guilds.len();

            drop(lock);
            // Unlock main database

            print_info(&format!("There are {} active games", num_servers));
        } else {
            print_error("Could not retrieve Guild ID (Command from a DM?)");
        }
    }

    Ok(())
}

#[command]
async fn end(ctx: &Context, msg: &Message) -> CommandResult {
    if is_storyteller(&ctx, &msg).await {
        print_command(&msg);

        let is_guild: bool = msg.guild_id.as_ref().is_some();

        if is_guild {
            let guild_id = msg.guild_id.as_ref().unwrap().as_u64();

            let content = String::from("**Ended game!**");
            send_msg(&msg, &ctx, content).await;

            // Start accesssing main database with lock
            let mut lock = BLOOD_DATABASE.lock().await;

            lock.blood_guilds.remove(&guild_id);

            let num_servers = lock.blood_guilds.len();

            drop(lock);
            // Unlock main database

            print_info(&format!("There are {} active games", num_servers));
        } else {
            print_error("Could not retrieve Guild ID (Command from a DM?)");
        }
    }

    Ok(())
}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}
async fn roles(ctx: &Context, msg: &Message) {
    print_command(&msg);

    let guild_id = msg.guild_id.as_ref().unwrap().as_u64();

    // Start accesssing main database with lock
    let lock = BLOOD_DATABASE.lock().await;

    let mut current_state = lock.blood_guilds[guild_id].clone();

    drop(lock);
    // Unlock main database

    let mut is_correct = false;

    match current_state.game_state {
        GameState::Nothing => send_msg(&msg, &ctx, String::from("Game not active!")).await,
        GameState::Playing => {
            send_msg(&msg, &ctx, String::from("Cannot edit roles in-game!")).await
        }
        _ => is_correct = true,
    }

    if is_correct {
        // Set the game state
        current_state.game_state = GameState::SettingRoles;

        // Check to see if this was called for the first time or is a continuation
        if &msg.content == "roles" {
            // If it hasn't, get all channels, check each one to see if it's
            // a voice channel, and if it is a voice channel, see if the
            // storyteller who sent the command is in it. If something fails,
            // send an error message to the channel.

            send_msg(&msg, &ctx, String::from("Getting members in your VC...")).await;

            let all_channels = GuildId(guild_id.clone()).channels(&ctx.http).await.unwrap();

            let mut storyteller_voice_channel: Option<GuildChannel> = None;

            let storyteller_id = msg.author.id;

            for channel in all_channels {
                if channel.1.kind == ChannelType::Voice {
                    let temp_members = channel.1.members(&ctx.cache).await.unwrap();

                    for member in temp_members {
                        if member.user.id == storyteller_id {
                            storyteller_voice_channel = Some(channel.1.clone());
                        }
                    }
                }
            }

            if let Some(value) = storyteller_voice_channel {
                let members_in_vc = value.members(&ctx.cache).await.unwrap();

                for member in members_in_vc {
                    if member.user.id != storyteller_id {
                        current_state.roles.push((
                            *member.user.id.as_u64(),
                            member,
                            String::from("none"),
                        ));
                    }
                }

                if current_state.roles.len() > 0 {
                    send_msg(
                        &msg,
                        &ctx,
                        String::from("**Done!** Please respond to the prompts for each member:"),
                    )
                    .await;

                    ask_for_role(&ctx, &msg, current_state).await;
                } else {
                    send_msg(
                        &msg,
                        &ctx,
                        String::from("**Error:** Could not find any other members in VC!"),
                    )
                    .await;
                }
            } else {
                send_msg(
                    &msg,
                    &ctx,
                    String::from("**Error:** Please join a voice channel!"),
                )
                .await;
            }
        } else {
            // Was a continuation, so assign the role to the last blank user
            for index in 0..current_state.roles.len() {
                if current_state.roles[index].2 == String::from("none") {
                    current_state.roles[index].2 = msg.content.clone();
                    // Once user is found, break loop

                    print_info(&format!(
                        "User {} is role {}",
                        current_state.roles[index].1.user.name, current_state.roles[index].2
                    ));
                    ask_for_role(&ctx, &msg, current_state).await;
                    break;
                }
            }
        }
    }
}

async fn dm_roles(ctx: &Context, msg: &Message) {
    print_command(&msg);

    let guild_id = msg.guild_id.as_ref().unwrap().as_u64();

    // Start accesssing main database with lock
    let lock = BLOOD_DATABASE.lock().await;

    let mut current_state = lock.blood_guilds[guild_id].clone();

    drop(lock);
    // Unlock main database

    if current_state.roles.len() > 0 {
        let mut successful_dms: u32 = 0;

        for member in &current_state.roles {
            let message_to_send: String = format!(
                "Your role this game is **{}**. Don't tell anyone!",
                &member.2
            );

            let result = &member
                .1
                .user
                .direct_message(&ctx.http, |m| m.content(&message_to_send))
                .await;

            match result {
                Ok(_) => {
                    successful_dms += 1;
                }
                Err(_why) => {
                    print_error(&format!(
                        "Could not send message to {}",
                        &member.1.user.name
                    ));

                    send_msg(
                        &msg,
                        &ctx,
                        String::from(format!(
                            "**Error:** could not send {} their role!",
                            &member.1.user.name
                        )),
                    )
                    .await;
                }
            };
        }

        send_msg(
            &msg,
            &ctx,
            String::from(format!("**Sent {} successful DMs!**", successful_dms)),
        )
        .await;
    } else {
        send_msg(
            &msg,
            &ctx,
            String::from("**Error:** No roles have been set!"),
        )
        .await;
    }

    // Once completed without errors, gamestate is set to playing
    current_state.game_state = GameState::Playing;

    // Start accesssing main database with lock
    let mut lock = BLOOD_DATABASE.lock().await;

    lock.blood_guilds.insert(current_state.id, current_state);

    drop(lock);
    // Unlock main database
}

async fn night(ctx: &Context, msg: &Message) {
    print_command(&msg);

    send_msg(&msg, &ctx, String::from("Sending members to sleep...")).await;

    let guild_id = msg.guild_id.as_ref().unwrap().as_u64();

    // Start accesssing main database with lock
    let lock = BLOOD_DATABASE.lock().await;

    let mut current_state = lock.blood_guilds[guild_id].clone();

    drop(lock);
    // Unlock main database

    let all_channels = GuildId(guild_id.clone()).channels(&ctx.http).await.unwrap();

    let mut night_category: Option<GuildChannel> = None;

    for channel in all_channels.clone() {
        // Check if the channel is both a category and has "night" in the name
        if channel.1.kind == ChannelType::Category
            && channel.1.name.to_lowercase().contains("night")
        {
            night_category = Some(channel.1);
            break;
        }
    }

    if let Some(value) = night_category {
        let night_category: GuildChannel = value;

        let mut night_channels: Vec<(GuildChannel, bool)> = Vec::new();

        for channel in all_channels.clone() {
            if channel.1.kind == ChannelType::Voice {
                if let Some(value) = channel.1.category_id {
                    if value.as_u64() == night_category.id.as_u64() {
                        night_channels.push((channel.1.clone(), false));
                    }
                }
            }
        }

        if &night_channels.len() >= &current_state.roles.len() {
            for member in current_state.roles.clone() {
                // If they are not in the room assignment HashMap, create
                // an assigned room for them
                if !current_state.room_assignments.contains_key(&member.0) {
                    for index in 0..night_channels.len() {
                        if night_channels[index].1 == false {
                            current_state
                                .room_assignments
                                .insert(member.0, night_channels[index].0.id);

                            night_channels[index].1 = true;
                            break;
                        }
                    }
                }

                let temp_assignment = current_state.room_assignments.get(&member.0);

                if let Some(value) = temp_assignment {
                    // Move them to the assigned room
                    member.1.move_to_voice_channel(&ctx.http, value).await;
                } else {
                    send_msg(
                        &msg,
                        &ctx,
                        String::from(format!(
                            "**Error:** Corrupted room assignment for {}",
                            member.1.user.name
                        )),
                    )
                    .await;
                }
            }

            // Start accesssing main database with lock
            let mut lock = BLOOD_DATABASE.lock().await;

            lock.blood_guilds.insert(current_state.id, current_state);

            drop(lock);
            // Unlock main database

            send_msg(&msg, &ctx, String::from("**Sent!**")).await;
        } else {
            send_msg(
                &msg,
                &ctx,
                String::from("**Error:** Not enough night Voice Channels!"),
            )
            .await;
        }
    } else {
        send_msg(
            &msg,
            &ctx,
            String::from("**Error:** Could not find a category of night channels!"),
        )
        .await;
    }
}

async fn day(ctx: &Context, msg: &Message) {
    print_command(&msg);

    send_msg(&msg, &ctx, String::from("Waking up members...")).await;

    save(&ctx, &msg).await;

    let guild_id = msg.guild_id.as_ref().unwrap().as_u64();

    // Start accesssing main database with lock
    let lock = BLOOD_DATABASE.lock().await;

    let current_state = lock.blood_guilds[guild_id].clone();

    drop(lock);
    // Unlock main database

    if &current_state.roles.len() > &(0 as usize) {
        let all_channels = GuildId(guild_id.clone()).channels(&ctx.http).await.unwrap();

        let mut town_voice_channel: Option<GuildChannel> = None;

        for channel in all_channels.clone() {
            // Check if the channel is both a category and has "night" in the name
            if channel.1.kind == ChannelType::Voice
                && channel.1.name.to_lowercase().contains("town")
            {
                town_voice_channel = Some(channel.1);
                break;
            }
        }

        if let Some(value) = town_voice_channel {
            for member in &current_state.roles {
                member
                    .1
                    .move_to_voice_channel(&ctx.http, value.clone())
                    .await;
            }
        } else {
            send_msg(
                &msg,
                &ctx,
                String::from("**Error:** No Voice Channel with \"town\" in the name was found!"),
            )
            .await;
        }
    } else {
        send_msg(
            &msg,
            &ctx,
            String::from("**Error:** Roles have not been assigned yet, so no members were moved!"),
        )
        .await;
    }
}

async fn save(ctx: &Context, msg: &Message) {
    print_command(&msg);

    let guild_id = msg.guild_id.as_ref().unwrap().as_u64();

    // Start accesssing main database with lock
    let lock = BLOOD_DATABASE.lock().await;

    let mut current_state = lock.blood_guilds[guild_id].clone();

    drop(lock);
    // Unlock main database

    let all_channels = GuildId(guild_id.clone()).channels(&ctx.http).await.unwrap();

    let mut night_category: Option<GuildChannel> = None;

    for channel in all_channels.clone() {
        // Check if the channel is both a category and has "night" in the name
        if channel.1.kind == ChannelType::Category
            && channel.1.name.to_lowercase().contains("night")
        {
            night_category = Some(channel.1);
            break;
        }
    }

    if let Some(value) = night_category {
        let night_category: GuildChannel = value;

        for channel in all_channels.clone() {
            if channel.1.kind == ChannelType::Voice {
                if let Some(value) = channel.1.category_id {
                    if value.as_u64() == night_category.id.as_u64() {
                        let temp_members = channel.1.members(&ctx.cache).await.unwrap();

                        if &temp_members.len() == &(1 as usize) {
                            current_state
                                .room_assignments
                                .insert(temp_members[0].user.id.as_u64().clone(), channel.0);
                        }
                    }
                }
            }
        }

        // Start accesssing main database with lock
        let mut lock = BLOOD_DATABASE.lock().await;

        lock.blood_guilds.insert(current_state.id, current_state);

        drop(lock);
    // Unlock main database
    } else {
        send_msg(
            &msg,
            &ctx,
            String::from("**Error:** Could not save night positions!"),
        )
        .await;
    }
}

async fn nothing(ctx: &Context, msg: &Message) {
    let content = String::from("Command not found. Please try again!");
    send_msg(&msg, &ctx, content).await;
}

// Helper functions

async fn ask_for_role(ctx: &Context, msg: &Message, mut current_state: BloodGuild) {
    let mut sent_request = false;

    for user_tuple in current_state.roles.clone() {
        if user_tuple.2 == String::from("none") {
            sent_request = true;

            let mut user_name: String = String::from(&user_tuple.1.user.name);

            if let Some(value) = &user_tuple.1.nick {
                user_name = value.clone();
            }

            send_msg(
                &msg,
                &ctx,
                String::from(format!("**Enter role** for *{}*", user_name)),
            )
            .await;

            break;
        }
    }

    if sent_request == false {
        send_msg(
            &msg,
            &ctx,
            String::from("All roles assigned! Ready to start the game..."),
        )
        .await;

        current_state.game_state = GameState::SettingUp;
    }

    // Start accesssing main database with lock
    let mut lock = BLOOD_DATABASE.lock().await;

    lock.blood_guilds.insert(current_state.id, current_state);

    drop(lock);
    // Unlock main database
}
