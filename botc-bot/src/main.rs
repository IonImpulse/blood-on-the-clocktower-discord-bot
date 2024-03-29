mod banners;
mod games;

use games::*;
use std::{collections::*, env, sync::Arc};
use tokio::sync::Mutex;
use lazy_static::lazy_static;
use serenity::{framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
}, model::prelude::EmojiIdentifier};
use serenity::{
    async_trait, client::bridge::gateway::ShardManager, client::*, prelude::*,
};
use serenity::model::{channel::*, event::*, gateway::*, guild::*, id::*};
use serenity_utils::prompt::reaction_prompt;

use colored::*;
use csv::Reader;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

async fn has_discord_role(ctx: &Context, msg: &Message, role_string: &str) -> bool {
    let roles: &Vec<RoleId> = &msg.member.as_ref().unwrap().roles;
    let mut is_role: bool = false;

    for role in roles {
        if role.to_role_cached(&ctx.cache).await.as_ref().is_some() {
            if role
                .to_role_cached(&ctx.cache)
                .await
                .as_ref()
                .unwrap()
                .name
                .to_lowercase()
                .contains(role_string)
            {
                is_role = true;
            }
        }
    }

    return is_role;
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
                if has_discord_role(&ctx, &msg, "storytell").await {
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
                                if msg.content.len() > 0 {
                                    let params: Vec<&str> = msg.content.split(" ").collect();

                                    let first_param = params.get(0).unwrap().clone();
                                    match first_param {
                                        "roles" => roles(&ctx, &msg).await,
                                        "dm" => dm_roles(&ctx, &msg).await,
                                        "night" => night(&ctx, &msg).await,
                                        "sleep" => night(&ctx, &msg).await,
                                        "day" => day(&ctx, &msg).await,
                                        "wake" => day(&ctx, &msg).await,
                                        "edit" => edit_role(&ctx, &msg).await,
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
}

#[group]
#[commands(start, end, help)]
struct General;

// Different coloured print functions
// Just for cosmetic purposes, but it does look very nice

fn print_info(string: &str) {
    println!("{}    █ {}", "INFO".green().bold(), string.normal());
}

fn print_status(string: &str) {
    println!("{}  █ {}", "STATUS".cyan().bold(), string.normal());
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
        "{}    █ {} : {}",
        "ECHO".blue().bold(),
        author_name.bold(),
        message.normal()
    );
}

fn print_command(msg: &Message) {
    println!(
        "{} █ [{}] {} {}#{}",
        "COMMAND".yellow().bold(),
        &msg.content.purple(),
        "by".yellow().italic(),
        &msg.author.name,
        &msg.author.discriminator
    );
}

fn print_error(msg: &str) {
    println!("{}   █ {}", "ERROR".red().bold(), msg);
}

// Function to send a message to a channel safely
async fn send_msg(msg: &Message, ctx: &Context, content: String) {
    if let Err(why) = &msg.channel_id.say(&ctx.http, content).await {
        print_error(&format!("Could not send message: {:?}", why));
    }
}

// Function to load game from CSV file

async fn load_game(game_name: String, path: &str) -> GameType {
    let mut rdr = Reader::from_path(path).unwrap();
    let mut temp_hashmap: HashMap<String, Character> = HashMap::new();

    for result in rdr.records() {
        let record = result.unwrap();

        let name = String::from(record.get(0).unwrap());
        let char_type: CharacterType;

        match record.get(1).unwrap() {
            "Demon" => char_type = CharacterType::Demon,
            "Minion" => char_type = CharacterType::Minion,
            "Outsider" => char_type = CharacterType::Outsider,
            "Townsfolk" => char_type = CharacterType::Townsfolk,
            "Traveler" => char_type = CharacterType::Traveler,
            "Fabled" => char_type = CharacterType::Fabled,
            "Decoy" => char_type = CharacterType::Decoy,
            _ => char_type = CharacterType::Other,
        }
        let first_order_index: i32 = record.get(2).unwrap().parse().unwrap();
        let order_index: i32 = record.get(3).unwrap().parse().unwrap();
        let night_action: ActionTime;

        match record.get(4).unwrap() {
            "EveryNight" => night_action = ActionTime::EveryNight,
            "NoNight" => night_action = ActionTime::NoNight,
            "VariableNight" => night_action = ActionTime::VariableNight,
            "OnlyFirstNight" => night_action = ActionTime::OnlyFirstNight,
            "EveryNightNotFirst" => night_action = ActionTime::EveryNightNotFirst,
            "DeathNight" => night_action = ActionTime::DeathNight,
            _ => night_action = ActionTime::NoNight,
        }

        temp_hashmap.insert(
            String::from(&name),
            Character::new(
                name,
                char_type,
                first_order_index,
                order_index,
                night_action,
            ),
        );
    }

    return GameType::new(game_name, temp_hashmap);
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

#[derive(Copy, Clone)]
pub enum Time {
    Day,
    Night,
}

impl Time {
    pub fn as_str(&self) -> &str {
        match *self {
            Time::Day => return "Day",
            Time::Night => return "Night",
        }
    }
}

#[derive(Clone)]
pub struct BloodGuild {
    id: u64,
    game_state: GameState,
    storyteller_channel: u64,
    roles: Vec<(u64, Member, Option<Character>)>,
    game_type: GameType,
    time: Time,
    day_index: u32,
}

// Global HashMap struct to hold all global data
#[derive(Clone)]
pub struct GlobalBloodState {
    blood_guilds: HashMap<u64, BloodGuild>,
    games: Vec<GameType>,
}

impl BloodGuild {
    /// Create a new, empty, instance of "BloodGuild".
    fn new(id: u64, storyteller_channel: u64, game_type: GameType) -> Self {
        BloodGuild {
            id: id,
            game_state: GameState::SettingUp,
            storyteller_channel: storyteller_channel,
            roles: Vec::new(),
            time: Time::Day,
            day_index: 0,
            game_type: game_type,
        }
    }

    pub fn get_time_str(&self) -> String {
        return format!("Day: {} | Time: {}", self.day_index, self.time.as_str());
    }
}

impl GlobalBloodState {
    /// Create a new, empty, instance of "GlobalBloodState".
    fn new() -> Self {
        GlobalBloodState {
            blood_guilds: HashMap::new(),
            games: Vec::new(),
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
    // Print banner startup art
    println!("\n{}", banners::LINE.black());
    println!("{}", banners::STARTUP.red());
    println!("{}\n", banners::LINE.black());

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

    print_status("Loading games...");

    // Loop through 'games' folder and load all games
    // where the name is the filename without .csv

    for entry in std::fs::read_dir("games").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let game_name = file_name.split('.').collect::<Vec<&str>>()[0];
        let game_type = load_game(String::from(game_name), path.to_str().unwrap()).await;

        // Lock and unlock the database to add the game
        BLOOD_DATABASE.lock().await.games.push(game_type);
    }

    print_info("Loaded games:");
    for game in &BLOOD_DATABASE.lock().await.games {
        print_info(&format!(" - {}", game.get_name()));
    }

    print_info("Started!");
    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn start(ctx: &Context, msg: &Message) -> CommandResult {
    if has_discord_role(&ctx, &msg, "storytell").await {
        print_command(&msg);

        let is_guild: bool = msg.guild_id.as_ref().is_some();

        if is_guild {
            let guild_id = msg.guild_id.as_ref().unwrap().as_u64();

            let channel_id: &u64 = msg.channel_id.as_u64();

            print_status(&format!("Setting up new server with id [{}]", guild_id));

            let content =
                String::from("**New game has been created!** Now bound to this channel...");

            send_msg(&msg, &ctx, content).await;

            let emojis_list = vec![
                '❤',
                '🧡',
                '💛',
                '💚',
                '💙',
                '💜',
                '🖤',
                '🤍',
                '🤎',
                '💖',
                '💗',
                '💓',
                '💞',
                '💘',
                '💝',
                '🩸',
                '🦇',
                '🔪',
                '🧛',
                '🧟',
                '🧙',
                '🧚',
                '🧜',
                '🧞',
                '👻',
                '👽',
                '👾',
                '🤖',
                '🎃',
                '👹',
                '👺',
                '🤡',
                '👿',
                '💩',
                '🤠',
                '👻',
                '👽',
                '👾',
                '🤖',
                '🎃',
                '👹',
                '👺',
                '🤡',
                '👿',
                '💩',
                '🤠',
                '👻',
                '👽',
                '👾',
                '🤖',
                '🎃',
                '👹',
                '👺',
                '🤡',
                '👿',
                '💩',
                '🤠',
                '👻',
                '👽',
                '👾',
                '🤖',
                '🎃',
                '👹',
                '👺',
                '🤡',
                '👿',
                '💩',
                '🤠',
                '👻',
                '👽',
                '👾',
                '🤖',
                '🎃',
                '👹',
                '👺',
                '🤡',
                '👿',
                '💩',
                '🤠',
            ];

            // Ask what game edition to play with serenity-utils
            let mut emojis = Vec::new();
            let mut desc = String::new();

            let mut i = 0;
            for game in &BLOOD_DATABASE.lock().await.games {
                emojis.push(ReactionType::from(emojis_list[i]));

                desc = format!("{}\n\n{}: {}", desc, emojis_list[i], game.get_name());

                i += 1;
            }

            let prompt_msg = msg
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|mut e| {
                        e.title("Select Game Type:");
                        e.description(
                            desc
                        );
                        e
                    });
                    m
                })
                .await;

            // Creates the prompt and returns the result. Because of `reaction_prompt`'s
            // return type, you can use the `?` operator to get the result.
            // The `Ok()` value is the selected emoji's index (wrt the `emojis` slice)
            // and the emoji itself. We don't require the emoji here, so we ignore it.
            let (idx, _) =
                reaction_prompt(ctx, &prompt_msg.unwrap(), &msg.author, &emojis, 120.0).await?;

            let content = String::from(
                "**Type \"roles\" to start assigning roles once everyone is in voice chat!**",
            );

            send_msg(&msg, &ctx, content).await;

            // Start accesssing main database with lock
            let mut lock = BLOOD_DATABASE.lock().await;

            let game_type = lock.games.get(idx).unwrap().clone();

            let temp_server = BloodGuild::new(*guild_id, *channel_id, game_type.clone());

            lock.blood_guilds.insert(*guild_id, temp_server);

            let num_servers = lock.blood_guilds.len();

            drop(lock);
            // Unlock main database

            let mut content: String = String::from(
                "```markdown\n       Name       | Character Type |      Wake Condition      \n",
            );
            content += "--------------------------------------------------------------\n";

            let mut characters = game_type.get_all_characters();
            characters.sort_by_key(|d| d.char_type_str.clone());

            for character in characters {
                content += character.get_string().as_str();
                content += "\n";
            }
            content += "```";

            send_msg(&msg, &ctx, content).await;

            print_info(&format!("There are {} active games", num_servers));
        } else {
            print_error("Could not retrieve Guild ID (Command from a DM?)");
        }
    }

    Ok(())
}

#[command]
async fn end(ctx: &Context, msg: &Message) -> CommandResult {
    if has_discord_role(&ctx, &msg, "storytell").await {
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
    // Return a message with all the commands
    print_command(&msg);

    let help_msg = "```markdown
Use the command ~start to start a game.

Blood 🩸 will be bound to the channel that ~start was executed in, so for the duration of the game, you won't need to use the ~ prefix to execute commands.
Ex: Someone with a \"Storytellers\" role sends the message \"~start\" in a channel that only storytellers can see. Blood will respond with a confirmation, and will now respond to commands in that channel without needing to use the prefix ~.

Now you can send the following commands without a prefix in that channel to continue the flow of the game:
roles

    Starts a call/response to save the role of every player in the Voice Channel who is not the storyteller. Once done, you can type dm and start the game!.

edit [number]

    Will edit the role of the specified number!

dm

    Will DM all saved roles to each player. If there are no roles set for the session, this command will fail. If there are some players who have roles and some who don't, this command will fail.

night

    Moves everyone but the storyteller who executed the command (who is also in the Voice Channel the storyteller is in) to a night room. If there is a saved ordering of people, it will use that order.

day

    Saves the ordering of people in night rooms and moves everyone to the Voice Channel with \"town\" in the name.

save

    Saves the ordering of people in night rooms without moving them.

Then, use the command ~end to end the game. This will clear all data, including which channel is bound, which roles players have, and how many nights have passed.

Questions/Comments:
Please contact @IonImpulse#1190 on Discord. Or, email me at edv121@outlook.com

Privacy Policy:
https://raw.githubusercontent.com/IonImpulse/blood-on-the-clocktower-discord-bot/main/legal/privacy-policy.md
```";

    send_msg(&msg, &ctx, String::from(help_msg)).await;

    Ok(())
}

async fn roles(ctx: &Context, msg: &Message) {
    print_command(&msg);

    let guild_id = msg.guild_id.as_ref().unwrap().as_u64();

    let mut current_state = get_database(&guild_id).await;

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
                    if member.user.id != storyteller_id
                        && !has_discord_role(&ctx, &msg, "spectator").await
                    {
                        let mut taken = false;

                        for i in current_state.roles.clone() {
                            if &i.0 == member.user.id.as_u64() {
                                taken = true;
                            }
                        }
                        if taken == false {
                            current_state
                                .roles
                                .push((*member.user.id.as_u64(), member, None));
                        }
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
            // Was a continuation, so assign the role to the last blank user OR last decoy user
            for index in 0..current_state.roles.len() {
                let temp_char_type: &str;

                // Test if it's a real character
                if current_state.roles[index.clone()].2.as_ref().is_some() {
                    // If it is, is it a Decoy?
                    if current_state.roles[index.clone()]
                        .2
                        .as_ref()
                        .unwrap()
                        .char_type_str
                        == "Decoy"
                    {
                        // If it's a Decoy, is it full?
                        if current_state.roles[index.clone()]
                            .2
                            .as_ref()
                            .unwrap()
                            .decoy_character
                            .is_some()
                        {
                            // If it's full, make sure that we mark it as None
                            temp_char_type = "None";
                        } else {
                            // Else, mark it as a Decoy character to be filled
                            temp_char_type = "Decoy";
                        }
                    } else {
                        // If ti's not a decoy, mark it as None
                        temp_char_type = "None";
                    }
                } else {
                    // If it's empty, mark it as Normal
                    temp_char_type = "Normal";
                }

                // If it's either to be filled normally or as decoy, assign it a Character type
                if temp_char_type == "Decoy" || temp_char_type == "Normal" {
                    let mut found_character: Option<Character> = None;

                    for character in current_state.game_type.get_all_characters() {
                        if character
                            .name
                            .to_lowercase()
                            .contains(msg.content.clone().to_lowercase().as_str())
                        {
                            found_character = Some(character.clone());
                            break;
                        }
                    }

                    if let Some(c_value) = found_character.clone() {
                        if temp_char_type == "Decoy" {
                            let real_char = current_state.roles[index.clone()].clone().2.unwrap();

                            let decoy_char: DecoyCharacter = DecoyCharacter {
                                name: c_value.name,
                                alignment: c_value.alignment,
                                char_type: c_value.char_type,
                                char_type_str: c_value.char_type_str,
                            };

                            let char_to_assign: Character =
                                Character::add_decoy(real_char, decoy_char);

                            current_state.roles[index.clone()].2 = Some(char_to_assign);

                            print_info(&format!(
                                "User {}'s decoy role is the {}",
                                current_state.roles[index.clone()].1.user.name,
                                current_state.roles[index.clone()]
                                    .2
                                    .as_ref()
                                    .unwrap()
                                    .decoy_character
                                    .as_ref()
                                    .unwrap()
                                    .name
                            ));
                        } else {
                            current_state.roles[index.clone()].2 = found_character;
                            print_info(&format!(
                                "User {} is role {}",
                                current_state.roles[index.clone()].1.user.name,
                                current_state.roles[index.clone()].2.as_ref().unwrap().name
                            ));
                        }
                    } else {
                        let content = format!(
                            "Could not find role {} in current game. Please try again!",
                            msg.content.clone().as_str()
                        );
                        send_msg(&msg, &ctx, content).await;
                    }

                    ask_for_role(&ctx, &msg, current_state).await;

                    // Once user is found, break loop

                    break;
                }
            }
        }
    }
}

async fn dm_roles(ctx: &Context, msg: &Message) {
    print_command(&msg);

    send_msg(&msg, &ctx, String::from("**Sending...**")).await;

    let guild_id = msg.guild_id.as_ref().unwrap().as_u64();

    let mut current_state = get_database(&guild_id).await;

    if current_state.roles.len() > 0 {
        let mut successful_dms: u32 = 0;

        for member in &current_state.roles {
            let message_to_send: String;

            if let Some(value) = &member.2.as_ref().unwrap().decoy_character {
                message_to_send = format!(
                    "Your role this game is the **{}**, a **{}**.",
                    &value.name, &value.char_type_str,
                );
            } else {
                message_to_send = format!(
                    "Your role this game is the **{}**, a **{}**.",
                    &member.2.as_ref().unwrap().name,
                    &member.2.as_ref().unwrap().char_type_str,
                );
            }

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
            String::from(format!(
                "**Sent {} successful DMs!** Type \"night\" to send people to sleep!",
                successful_dms
            )),
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

    set_database(current_state).await;
}

async fn night(ctx: &Context, msg: &Message) {
    print_command(&msg);

    let guild_id = msg.guild_id.as_ref().unwrap().as_u64();

    let mut current_state = get_database(&guild_id).await;

    match current_state.time {
        Time::Day => current_state.day_index += 1,
        _ => (),
    }

    current_state.time = Time::Night;
    let title: &str;
    let mut content = String::from("");

    let mut characters = current_state.roles.clone();

    if current_state.day_index == 1 {
        title = "First Night Order";
        characters.sort_by_key(|d| d.2.as_ref().unwrap().first_order_index.clone());

        let mut index: u32 = 1;

        for character in characters.clone() {
            let character_role = character.2.as_ref().unwrap();

            if character_role.first_order_index != -1 {
                // Try to find a user.nick_in(), but if it doesn't exist, use user.name
                let name = match character.1.user.nick_in(&ctx, *guild_id).await {
                    Some(value) => value,
                    None => character.1.user.name.clone(),
                };

                content.push_str(
                    format!(
                        "{}) **{}** as the {}\n",
                        index, name, character_role.name
                    )
                    .as_str(),
                );

                index += 1;
            }
        }
    } else {
        title = "Night Order";
        characters.sort_by_key(|d| d.2.as_ref().unwrap().order_index.clone());

        let mut index: u32 = 1;

        for character in characters.clone() {
            let character_role = character.2.as_ref().unwrap();
            if character_role.order_index != -1 {
                let warning: &str;

                match character_role.night_action {
                    ActionTime::OnlyFirstNight => warning = " *if triggered*",
                    ActionTime::VariableNight => warning = " *if triggered*",
                    ActionTime::DeathNight => warning = " *if triggered*",
                    _ => warning = "",
                }
                
                let name = match character.1.user.nick_in(&ctx, *guild_id).await {
                    Some(value) => value,
                    None => character.1.user.name.clone(),
                };

                content.push_str(
                    format!(
                        "{}) **{}** as the {}{}\n",
                        index, name, character_role.name, warning,
                    )
                    .as_str(),
                );

                index += 1;
            }
        }
    }

    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|mut e| {
                e.title(title);
                e.description(content);
                e
            });
            m
        })
        .await;

    send_msg(&msg, &ctx, String::from("Sending members to sleep...")).await;

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

        night_channels.sort_by_key(|d| d.0.position);
        let mut r_night_channels = night_channels.clone();
        r_night_channels.reverse();

        let mut taken_channels: Vec<bool> = vec![false; night_channels.len()];

        if &night_channels.len() >= &current_state.roles.len() {
            for member in characters {
                let character_role = member.2.as_ref().unwrap();
                let mut found_channel: Option<GuildChannel> = None;

                if (current_state.day_index == 1 && character_role.first_order_index == -1)
                    || (current_state.day_index != 1 && character_role.order_index == -1)
                {
                    let mut index: usize = night_channels.len() - 1;
                    for value in r_night_channels.clone() {
                        if taken_channels.get(index).unwrap() == &false {
                            taken_channels[index] = true;
                            found_channel = Some(value.0);
                            break;
                        }

                        index -= 1;
                    }
                } else {
                    let mut index: usize = 0;

                    for value in night_channels.clone() {
                        if taken_channels.get(index).unwrap() == &false {
                            taken_channels[index] = true;
                            found_channel = Some(value.0);
                            break;
                        }

                        index += 1;
                    }
                }

                if let Some(value) = found_channel {
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

            set_database(current_state).await;

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

    let guild_id = msg.guild_id.as_ref().unwrap().as_u64();

    let mut current_state = get_database(&guild_id).await;

    current_state.time = Time::Day;

    set_database(current_state.clone()).await;

    if &current_state.roles.len() > &(0 as usize) {
        let all_channels = GuildId(guild_id.clone()).channels(&ctx.http).await.unwrap();

        let mut town_voice_channel: Option<GuildChannel> = None;

        for channel in all_channels.clone() {
            // Check if the channel is both a category and has "town" in the name
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

async fn edit_role(ctx: &Context, msg: &Message) {
    print_command(&msg);

    let guild_id = msg.guild_id.as_ref().unwrap().as_u64();

    // Start accesssing main database with lock
    let lock = BLOOD_DATABASE.lock().await;

    let mut current_state = lock.blood_guilds[guild_id].clone();

    drop(lock);
    // Unlock main database

    let params: Vec<&str> = msg.content.split(" ").collect();

    if params.len() == 2 {
        let try_num = params.get(1).unwrap().parse::<u16>();

        let num;
        match try_num {
            Ok(n) => num = n,
            Err(e) => num = 0,
        }

        if num > 0 {
            if num <= (current_state.roles.len() as u16) {
                let role_to_edit = current_state.roles.get((num - 1) as usize).unwrap();

                let role_to_return = (role_to_edit.0.clone(), role_to_edit.1.clone(), None);

                current_state.roles[(num - 1) as usize] = role_to_return;

                current_state.game_state = GameState::SettingRoles;

                ask_for_role(&ctx, &msg, current_state).await;
            } else {
                send_msg(
                    &msg,
                    &ctx,
                    String::from("Please provide a number in the valid range!"),
                )
                .await;
            }
        } else {
            send_msg(&msg, &ctx, String::from("Please provide a number to edit!")).await;
        }
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
        if user_tuple.2.is_some() == false {
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
        } else if &user_tuple.2.clone().unwrap().char_type_str == "Decoy"
            && &user_tuple.2.clone().unwrap().decoy_character.is_some() == &false
        {
            sent_request = true;

            let mut user_name: String = String::from(&user_tuple.1.user.name);

            if let Some(value) = &user_tuple.1.nick {
                user_name = value.clone();
            }

            send_msg(
                &msg,
                &ctx,
                String::from(format!("**Enter decoy role** for *{}*", user_name)),
            )
            .await;

            break;
        }
    }

    if sent_request == false {
        let mut content: String = String::from("");
        let mut num: u16 = 1;

        for user_tuple in &current_state.roles {
            let mut user_name: String = String::from(&user_tuple.1.user.name);

            if let Some(value) = &user_tuple.1.nick {
                user_name = value.clone();
            }

            content = format!(
                "{}{}) {} as the **{}**\n",
                content,
                num,
                user_name,
                user_tuple.2.as_ref().unwrap().name
            );
            num += 1;
        }

        let _ = msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|mut e| {
                    e.title("Role List:");
                    e.description(content);
                    e
                });
                m
            })
            .await;

        send_msg(
            &msg,
            &ctx,
            String::from(
                "Done setting up! Type \"edit\" to edit roles, or type \"dm\" to send roles out!",
            ),
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

async fn get_database(guild_id: &u64) -> BloodGuild {
    // Start accesssing main database with lock
    let lock = BLOOD_DATABASE.lock().await;

    let current_state = lock.blood_guilds[guild_id].clone();

    drop(lock);
    // Unlock main database

    return current_state;
}

async fn set_database(current_state: BloodGuild) {
    // Start accesssing main database with lock
    let mut lock = BLOOD_DATABASE.lock().await;

    lock.blood_guilds.insert(current_state.id, current_state);

    drop(lock);
    // Unlock main database
}
