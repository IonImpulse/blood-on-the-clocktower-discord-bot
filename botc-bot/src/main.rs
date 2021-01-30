use tokio;
use lazy_static::lazy_static;

use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::guild::*;
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group
    }
};

use std::{
    collections::*,
    env,
    sync::{Arc, Mutex},
};
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    http::Http,
    model::{event::ResumedEvent, gateway::Ready, id::RoleId},
    prelude::*,
};

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
            if role.to_role_cached(&ctx.cache).await
            .as_ref()
            .unwrap()
            .name
            .to_lowercase()
            .contains("storytell") {
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
        if !msg.content.starts_with("~") {
            let message_in_guild: bool = msg.member.as_ref().is_some();

            if message_in_guild {
                
                if is_storyteller(&ctx, &msg).await {
                    print_echo(&msg);
                }
            }
        }
        
    }
}

#[group]
#[commands(start,end)]
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
    
    println!("{}    | {} : {}", "ECHO".blue().bold(), author_name.bold(), message.normal());
}

fn print_command(ctx: &Context, msg: &Message) {
    println!("{} | [{}] | {}#{}", "COMMAND".yellow().bold(), &msg.content.purple(), &msg.author.name, &msg.author.discriminator);
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

pub enum GameState {
    Nothing,
    SettingUp,
    Playing
}

pub struct BloodGuild {
    id: u64,
    game_state: GameState,
    storyteller_channel: u64,
}

// Global HashMap struct to hold all global data
pub struct GlobalBloodState {
    blood_guilds: HashMap<u64, BloodGuild>,
}

impl BloodGuild {
    /// Create a new, empty, instance of "BloodGuild".
    fn new(id: u64, storyteller_channel: u64,) -> Self {
        BloodGuild {
            id: id,
            game_state: GameState::SettingUp,
            storyteller_channel: storyteller_channel,
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
    static ref BLOOD_DATABASE: Arc<Mutex<GlobalBloodState>> = Arc::new(Mutex::new(GlobalBloodState::new()));
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
        print_command(&ctx, &msg);

        let is_guild: bool = msg.guild_id.as_ref().is_some();
        
        if is_guild {
            let guild_id = msg.guild_id.as_ref().unwrap().as_u64();
    
            let channel_id: &u64 = msg.channel_id.as_u64();
    
            print_status(&format!("Setting up new server with id [{}]", guild_id));
    
            let content = String::from("**New game has been created!** Now bound to this channel...");
            
            send_msg(&msg, &ctx, content).await;
    
            let temp_server = BloodGuild::new(*guild_id, *channel_id);
            
            // Start accesssing main database with lock
            let mut lock = BLOOD_DATABASE.lock().unwrap();
            
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
		print_command(&ctx, &msg);

        let is_guild: bool = msg.guild_id.as_ref().is_some();
          
        if is_guild {
            let guild_id = msg.guild_id.as_ref().unwrap().as_u64();
      
            let content = String::from("**Ended game!**");
			send_msg(&msg, &ctx, content).await;

            // Start accesssing main database with lock
            let mut lock = BLOOD_DATABASE.lock().unwrap();
                
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