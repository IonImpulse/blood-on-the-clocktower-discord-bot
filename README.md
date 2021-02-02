# Blood 🩸 Discord Bot
 
 ## Hello, and welcome to the *unofficial* **Blood On The Clocktower** Discord bot, ***Blood*** 🩸!
If you're interested in *hosting it yourself*, jump down to the bottom, where technical aspects will be lined out.

If you're interested in *how to use it*, simply continue reading.

## How to Use
### 1: Invite
Use this [link](https://discord.com/oauth2/authorize?&client_id=804522025946578974&scope=bot&permissions=17034304) to invite Blood 🩸 to your server!
### 2: Setup your server
This is an example of what your server should look like:

![Example Discord Server](https://github.com/IonImpulse/blood-on-the-clocktower-discord-bot/raw/main/assets/Setup%20Photo.png)

Make sure that your storyteller has a role with the word "storyteller" in it, case insensitive. It does not need any permissions, it is just a way to differentiate who can control games and who shouldn't be moved at night/wake-up.

#### Ex: there are 10 people in a server, and two of them have a role named "Storytellers". The one who uses the command "~start" will be able to control the flow of the game, but no one else will.

Make sure that you have a Voice Channel category with the word "night" in it, case insensitive. There should also be enough channels in that category for each player. Additionally, make sure that there is a Voice Channel *not* in the night category that has the word "town" in it, case insensitive.

#### Ex: There are two Voice Channel catagories: "General" and "Night Rooms". The "Night Rooms" category has 20 voice channels with random names.

Additionally, sure that there is a text channel that only the storyteller can access. This is where they will execute commands and where Blood 🩸 will respond.

### 3: Start a game
Use the command **~start** to start a game.

Blood 🩸 will be bound to the channel that **~start** was executed in, so for the duration of the game, you won't need to use the **~** prefix to execute commands.

#### Ex: Someone with a "Storytellers" role sends the message "\~start" in a channel that only storytellers can see. Blood will respond with a confirmation, and will now respond to commands in that channel without needing to use the prefix **\~**.

Now you can send the following commands without a prefix in that channel to continue the flow of the game:

### roles
>Starts a call/response to save the role of every player in the Voice Channel who is not the storyteller. Once done, you can type **dm roles** and start the game!.

### dm roles
>Will DM all saved roles to each player. If there are no roles set for the session, this command will fail. If there are some players who have roles and some who don't, this command will fail.

### night
>Moves everyone *but* the storyteller who executed the command (who is also in the Voice Channel the storyteller is in) to a night room. If there is a saved ordering of people, it will use that order.

### day
>Saves the ordering of people in night rooms and moves everyone to the Voice Channel with "town" in the name.

### save
>Saves the ordering of people in night rooms without moving them.

Then, use the command **~end** to end the game. This will clear all data, including which channel is bound, which roles players have, and how many nights have passed.


## How to host
### 1: Download Blood 🩸
Go to the releases page and grab the latest version for your OS, or if you're fancy, you can clone the git repository and compile it using Rust yourself.
### 2: Create a bot
To to this [link](https://discord.com/developers/applications) and login with your Discord account. Click **New Application** and setup a name and icon, maybe just something like *Blood on the Clocktower* with the icon in this repo. Next, go to the **Bot** tab at the left hand side and click **Create a Bot**. Name it whatever you want (or just *Blood*), and give it an icon. Once that's all saved, click **copy** under the *Token*.
### 3: Set your token
Run this command on Windows to set your token: `[System.Environment]::SetEnvironmentVariable('BLOOD_TOKEN','ENTER YOUR TOKEN HERE')`

Make sure to replace **ENTER YOUR TOKEN HERE** with the copied token.
### 4: Invite your bot
Go back to the Discord Developer Portal and click on **OAuth2** on the left hand side of your application. In the **Scopes** section, click *Bot*. Copy the URL that appears and paste it into a browser. **Make sure to replace the** `permissions=0` **part of the URL with** `permissions=17034304`. Once done, you can press Enter to navigate to the webpage and select the server you want to invite the bot to. Make sure to bookmark this so you can invite your bot again. If you don't, you'll need to do this step again.
### 5: Run the bot
Double-click the downloaded release file or run the compiled Rust file to run the bot. It should be as easy as that!

# 🩸🔛🕒🗼