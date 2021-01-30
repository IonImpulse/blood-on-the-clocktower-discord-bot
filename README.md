# Blood 🩸 Discord Bot
 
 ## Hello, and welcome to the *unofficial* **Blood On The Clocktower** Discord bot, ***Blood*** 🩸!
If you're interested in *hosting it yourself*, jump down to the bottom, where technical aspects will be lined out.

If you're interested in *how to use it*, simply continue reading.

## How to Use
### 1: Invite
Use this [link](https://discord.com/oauth2/authorize?&client_id=804522025946578974&scope=bot&permissions=17034304) to invite Blood 🩸 to your server!
### 2: Setup your server
Make sure that your storyteller(s) has a role with the word "storyteller" in it, case insensitive. It does not need any permissions, it is just a way to differentiate who can control games and who shouldn't be moved at night/wakeup.

#### Ex: there are 10 people in a server, and two of them have a role named "Storytellers". They will be able to control the flow of the game, but no one else will.

Make sure that you have a Voice Channel category with the word "night" in it, case insensitive. There should also be enough channels in that category for each player.

#### Ex: There are three Voice Channel catagories: "General", "Day", and "Night Rooms". The "Night Rooms" category has 20 voice channels with random names.

Additionally, sure that there is a text channel that only the storytellers can access. This is where they will execute commands and where Blood 🩸 will respond.

### 3: Start a game
Use the command **~start** to start a game.

Blood 🩸 will be bound to the channel that **~start** was executed in, so for the duration of the game, you won't need to use the **~** prefix to execute commands.

#### Ex: Someone with a "Storytellers" role sends the message "\~start" in a channel that only storytellers can see. Blood will respond with a confirmation, and will now respond to commands in that channel without needing to use the prefix **\~**.

Now you can send the following commands without a prefix in that channel to continue the flow of the game:

### roles
    Starts a call/response to save the role of every player in the Voice Channel who is not the storyteller. Once done, will respond with a complete list of all **player/role** pairs, and will ask to either save, save and auto-DM, edit, or discard.

### dm roles
    Will DM all saved roles to each player. If there are no roles set for the session, this command will fail. If there are some players who have roles and some who don't, this command will fail, but can be overridden by specifying **"dm roles force"** 

### night
    Moves everyone *but* the storyteller who executed the command (who is also in the Voice Channel the storyteller is in) to a night room. If there is a saved ordering of people, it will use that order.

### day
    Saves the ordering of people in night rooms and moves everyone to the Voice Channel that the storyteller who executed the command is in.

### save order
    Saves the ordering of people in night rooms without moving them.

Then, use the command **~end** to end the game. This will clear all data, including which channel is bound, which roles players have, and how many nights have passed.