# Stat Summoner Bot with Shuttle

This project is a Discord bot called **Stat Summoner**, built using the [Poise](https://docs.rs/poise) and [Serenity](https://docs.rs/serenity) libraries, and deployed using [Shuttle](https://docs.shuttle.rs/). The bot fetches and displays League of Legends statistics such as player rank, top champions, and recent match data. It responds to custom commands like `/lolstats` and delivers information directly to the Discord server in an embed format.

## Features

- Fetches **League of Legends player statistics** based on user input.
- Shows **Solo/Duo and Flex rank** information.
- Displays the **top champions** of a player and their mastery level.
- Provides **match details** including K/D/A, farm, game result, and more.
- Embeds information in a clear, formatted message in Discord.

## Prerequisites

To run this bot, you need:
1. A valid **Discord Token**.
2. A **Riot Games API Key**.

## Setup

### Step 1: Create a Discord Application
1. Log in to the [Discord Developer Portal](https://discord.com/developers/applications).
2. Click the **New Application** button, give your application a name, and click **Create**.
3. Navigate to the **Bot** tab on the left-hand menu, and add a new bot.
4. On the bot page, click the **Reset Token** button to reveal your token. Copy this token and store it in your `Secrets.toml` file. Do not share this token with anyone.
    - Make sure you add `Secrets.toml` to your `.gitignore` to keep it out of version control.
5. Enable **Message Content Intent** on the bot page to allow the bot to respond to content.

### Step 2: Generate an Invite Link
1. On the application page, navigate to **OAuth2** in the left-hand panel.
2. Go to the **URL Generator**, select the `bot` scope, and under **Bot Permissions**, select the necessary permissions for your bot such as `Send Messages`, `Embed Links`, and `Use Slash Commands`.
3. Copy the generated URL and use it to invite the bot to your Discord server.

### Step 3: Set Up the Riot API
1. Go to the [Riot Developer Portal](https://developer.riotgames.com/) and create an API Key.
2. Add your Riot API Key to the `Secrets.toml` file.

### Step 4: Deploy with Shuttle
1. Install Shuttle CLI if you haven't already. Follow the [Shuttle documentation](https://docs.shuttle.rs/).
2. To deploy your bot, simply run:
    ```bash
    shuttle deploy
    ```
3. Make sure your `Secrets.toml` file contains both `DISCORD_TOKEN` and `RIOT_API_KEY`.

### Example of `Secrets.toml`
```toml
DISCORD_TOKEN = "your-discord-token-here"
RIOT_API_KEY = "your-riot-api-key-here"
```

## Configuration

Make sure to set your `RIOT_API_KEY` in your environment variables or `Secrets.toml`.

## Available Commands

### `/lolstats`
Fetch and display League of Legends player statistics by allowing the user to input their game name and tag. The bot retrieves information such as:

- **Solo/Duo rank** and **Flex rank**.
- **Top champions** with their mastery level and points.
- **Recent match details** (kills, deaths, assists, farm, game result).

## Documentation

The full Rust documentation for the project is available [here](https://shvvkz.github.io/stat-summoner/).

## Additional Resources

- [Poise Documentation](https://docs.rs/poise)
- [Serenity Documentation](https://docs.rs/serenity)
- [Riot Games Developer Portal](https://developer.riotgames.com)
- [Shuttle Documentation](https://docs.shuttle.rs)

## License

This project is licensed under the terms of the [MIT license](./LICENSE).
