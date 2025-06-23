//! RealMIR Discord Bot
//! 
//! Simplified Discord bot implementation that provides an interface to the RealMIR
//! prediction market system using serenity v0.12.

use std::env;
use std::sync::Arc;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult, StandardFramework};
use tokio::sync::Mutex;

use realmir_core::{
    CommitmentGenerator, CommitmentVerifier, RoundProcessor,
    MockEmbedder, BaselineAdjustedStrategy, PayoutCalculator,
    ConfigManager,
    Guess, Participant,
    Result as RealMirResult,
};
use realmir_core::types::{RoundConfig, RoundStatus};

/// Bot data shared across all command handlers
struct BotData {
    round_processor: Arc<Mutex<RoundProcessor<MockEmbedder, BaselineAdjustedStrategy>>>,
    commitment_generator: CommitmentGenerator,
    commitment_verifier: CommitmentVerifier,
    payout_calculator: PayoutCalculator,
    config_manager: ConfigManager,
}

impl BotData {
    fn new() -> RealMirResult<Self> {
        // Initialize embedder and scoring strategy
        let embedder = MockEmbedder::clip_like();
        let strategy = BaselineAdjustedStrategy::new();
        
        // Create round processor with persistent storage
        let rounds_file = "discord_rounds.json".to_string();
        let round_processor = RoundProcessor::new(rounds_file, embedder, strategy);
        
        // Initialize other components
        let commitment_generator = CommitmentGenerator::new();
        let commitment_verifier = CommitmentVerifier::new();
        let payout_calculator = PayoutCalculator::new();
        let config_manager = ConfigManager::new()?;
        
        Ok(Self {
            round_processor: Arc::new(Mutex::new(round_processor)),
            commitment_generator,
            commitment_verifier,
            payout_calculator,
            config_manager,
        })
    }
}

impl TypeMapKey for BotData {
    type Value = Arc<BotData>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("🚀 {} is connected!", ready.user.name);
        println!("📋 Use !help to see available commands");
    }
}

#[group]
#[commands(start_round, commit, reveal, score_round, status, generate_hash, list_rounds, help_cmd)]
struct General;

#[command]
#[aliases("start")]
#[description = "Start a new prediction round"]
#[usage = "!start_round <round_id> <title> <description> <image_url> [prize_pool]"]
async fn start_round(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let round_id = match args.single::<String>() {
        Ok(id) => id,
        Err(_) => {
            msg.reply(ctx, "❌ **Error:** Please provide a round ID").await?;
            return Ok(());
        }
    };

    let title = match args.single::<String>() {
        Ok(title) => title,
        Err(_) => {
            msg.reply(ctx, "❌ **Error:** Please provide a title").await?;
            return Ok(());
        }
    };

    let description = match args.single::<String>() {
        Ok(desc) => desc,
        Err(_) => {
            msg.reply(ctx, "❌ **Error:** Please provide a description").await?;
            return Ok(());
        }
    };

    let image_url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            msg.reply(ctx, "❌ **Error:** Please provide an image URL").await?;
            return Ok(());
        }
    };

    let prize_pool = args.single::<f64>().unwrap_or(100.0);

    let data = ctx.data.read().await;
    let bot_data = data.get::<BotData>().unwrap();
    let mut processor = bot_data.round_processor.lock().await;
    
    let config = RoundConfig {
        prize_pool,
        max_guess_length: 300,
        use_baseline_adjustment: true,
        baseline_text: Some("[UNUSED]".to_string()),
    };

    match processor.create_round(round_id.clone(), title.clone(), description, image_url, Some(config)) {
        Ok(_) => {
            if let Err(e) = processor.save_current_rounds() {
                msg.reply(ctx, format!("Round created but failed to save: {}", e)).await?;
                return Ok(());
            }
            msg.reply(ctx, format!("✅ **Round Created Successfully!**\n\n**Round ID:** `{}`\n**Title:** {}\n**Prize Pool:** ${:.2}\n\nPlayers can now use `!commit` to participate!", round_id, title, prize_pool)).await?;
        }
        Err(e) => {
            msg.reply(ctx, format!("❌ **Error creating round:** {}", e)).await?;
        }
    }

    Ok(())
}

#[command]
#[description = "Submit a commitment hash for a round"]
#[usage = "!commit <round_id> <hash>"]
async fn commit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let round_id = match args.single::<String>() {
        Ok(id) => id,
        Err(_) => {
            msg.reply(ctx, "❌ **Error:** Please provide a round ID").await?;
            return Ok(());
        }
    };

    let hash = match args.single::<String>() {
        Ok(h) => h,
        Err(_) => {
            msg.reply(ctx, "❌ **Error:** Please provide a commitment hash").await?;
            return Ok(());
        }
    };

    let user_id = msg.author.id.to_string();
    let username = msg.author.name.clone();

    let data = ctx.data.read().await;
    let bot_data = data.get::<BotData>().unwrap();
    let mut processor = bot_data.round_processor.lock().await;

    // Check if round exists and is open
    let round = match processor.get_round(&round_id) {
        Ok(round) => round,
        Err(_) => {
            msg.reply(ctx, format!("❌ **Round not found:** `{}`", round_id)).await?;
            return Ok(());
        }
    };

    if !round.is_open() {
        msg.reply(ctx, format!("❌ **Round is not accepting submissions:** `{}`", round_id)).await?;
        return Ok(());
    }

    // Create participant with empty guess (will be filled during reveal)
    let guess = Guess::new("".to_string()); // Placeholder
    let participant = Participant::new(user_id, username.clone(), guess, hash.clone());

    match processor.add_participant(&round_id, participant) {
        Ok(_) => {
            if let Err(e) = processor.save_current_rounds() {
                msg.reply(ctx, format!("Commitment recorded but failed to save: {}", e)).await?;
                return Ok(());
            }
            msg.reply(ctx, format!("✅ **Commitment Recorded!**\n\n**Player:** {}\n**Round:** `{}`\n**Hash:** `{}`\n\nUse `!reveal` to reveal your guess when the round is ready!", username, round_id, hash)).await?;
        }
        Err(e) => {
            msg.reply(ctx, format!("❌ **Error recording commitment:** {}", e)).await?;
        }
    }

    Ok(())
}

#[command]
#[description = "Reveal your guess for verification"]
#[usage = "!reveal <round_id> <guess> <salt>"]
async fn reveal(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let round_id = match args.single::<String>() {
        Ok(id) => id,
        Err(_) => {
            msg.reply(ctx, "❌ **Error:** Please provide a round ID").await?;
            return Ok(());
        }
    };

    let guess_text = match args.single::<String>() {
        Ok(guess) => guess,
        Err(_) => {
            msg.reply(ctx, "❌ **Error:** Please provide your guess").await?;
            return Ok(());
        }
    };

    let salt = match args.single::<String>() {
        Ok(s) => s,
        Err(_) => {
            msg.reply(ctx, "❌ **Error:** Please provide the salt").await?;
            return Ok(());
        }
    };

    let user_id = msg.author.id.to_string();

    let data = ctx.data.read().await;
    let bot_data = data.get::<BotData>().unwrap();
    let mut processor = bot_data.round_processor.lock().await;

    // Find the participant
    let round = match processor.get_round(&round_id) {
        Ok(round) => round,
        Err(_) => {
            msg.reply(ctx, format!("❌ **Round not found:** `{}`", round_id)).await?;
            return Ok(());
        }
    };

    let participant = match round.participants.iter().find(|p| p.user_id == user_id) {
        Some(p) => p,
        None => {
            msg.reply(ctx, format!("❌ **No commitment found for you in round:** `{}`", round_id)).await?;
            return Ok(());
        }
    };

    // Verify the commitment
    let expected_hash = match bot_data.commitment_generator.generate(&guess_text, &salt) {
        Ok(hash) => hash,
        Err(e) => {
            msg.reply(ctx, format!("❌ **Error generating verification hash:** {}", e)).await?;
            return Ok(());
        }
    };

    if participant.commitment != expected_hash {
        msg.reply(ctx, "❌ **Commitment verification failed!** Your guess and salt don't match your original commitment.").await?;
        return Ok(());
    }

    // Update the participant with the revealed guess
    if let Err(e) = processor.reveal_participant(&round_id, &user_id, &guess_text, &salt) {
        msg.reply(ctx, format!("❌ **Error recording reveal:** {}", e)).await?;
        return Ok(());
    }

    // Save the updated data
    if let Err(e) = processor.save_current_rounds() {
        msg.reply(ctx, format!("Reveal recorded but failed to save: {}", e)).await?;
        return Ok(());
    }

    msg.reply(ctx, format!("✅ **Guess Revealed and Verified!**\n\n**Player:** {}\n**Round:** `{}`\n**Guess:** \"{}\"\n\n✅ Commitment verification successful!", msg.author.name, round_id, guess_text)).await?;

    Ok(())
}

#[command]
#[aliases("score")]
#[description = "Process scoring and calculate payouts for a round"]
#[usage = "!score_round <round_id>"]
async fn score_round(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let round_id = match args.single::<String>() {
        Ok(id) => id,
        Err(_) => {
            msg.reply(ctx, "❌ **Error:** Please provide a round ID").await?;
            return Ok(());
        }
    };

    let data = ctx.data.read().await;
    let bot_data = data.get::<BotData>().unwrap();
    let mut processor = bot_data.round_processor.lock().await;

    // Process the round scoring
    match processor.process_round_scoring(&round_id) {
        Ok(_) => {
            // Get the updated round with results
            match processor.get_round(&round_id) {
                Ok(round) => {
                    if round.results.is_empty() {
                        msg.reply(ctx, format!("❌ **No results available for round:** `{}`", round_id)).await?;
                        return Ok(());
                    }

                    let mut response = format!("🏆 **Round `{}` Scoring Complete!**\n\n", round_id);
                    response.push_str(&format!("**Prize Pool:** ${:.2}\n\n", round.config.prize_pool));
                    response.push_str("**Final Rankings:**\n");

                    for (i, result) in round.results.iter().enumerate() {
                        let rank_emoji = match i {
                            0 => "🥇",
                            1 => "🥈", 
                            2 => "🥉",
                            _ => "🏅",
                        };
                        
                        response.push_str(&format!(
                            "{} **#{}** {} - Score: {:.4} - Payout: ${:.2}\n   Guess: \"{}\"\n\n",
                            rank_emoji,
                            i + 1,
                            result.participant.username,
                            result.effective_score(),
                            result.payout.unwrap_or(0.0),
                            result.participant.guess.text
                        ));
                    }

                    if let Err(e) = processor.save_current_rounds() {
                        response.push_str(&format!("\n⚠️ Results calculated but failed to save: {}", e));
                    }

                    msg.reply(ctx, response).await?;
                }
                Err(e) => {
                    msg.reply(ctx, format!("❌ **Error retrieving results:** {}", e)).await?;
                }
            }
        }
        Err(e) => {
            msg.reply(ctx, format!("❌ **Error processing round scoring:** {}", e)).await?;
        }
    }

    Ok(())
}

#[command]
#[description = "Get the status and details of a round"]
#[usage = "!status <round_id>"]
async fn status(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let round_id = match args.single::<String>() {
        Ok(id) => id,
        Err(_) => {
            msg.reply(ctx, "❌ **Error:** Please provide a round ID").await?;
            return Ok(());
        }
    };

    let data = ctx.data.read().await;
    let bot_data = data.get::<BotData>().unwrap();
    let mut processor = bot_data.round_processor.lock().await;

    match processor.get_round(&round_id) {
        Ok(round) => {
            let status_emoji = match round.status {
                RoundStatus::Open => "🟢",
                RoundStatus::Processing => "🟡",
                RoundStatus::Complete => "✅",
                RoundStatus::Cancelled => "❌",
            };

            let mut response = format!("{} **Round Status: `{}`**\n\n", status_emoji, round_id);
            response.push_str(&format!("**Title:** {}\n", round.title));
            response.push_str(&format!("**Description:** {}\n", round.description));
            response.push_str(&format!("**Status:** {:?}\n", round.status));
            response.push_str(&format!("**Prize Pool:** ${:.2}\n", round.config.prize_pool));
            response.push_str(&format!("**Total Participants:** {}\n", round.participants.len()));
            response.push_str(&format!("**Verified Participants:** {}\n", round.verified_participants().len()));
            response.push_str(&format!("**Created:** <t:{}:f>\n\n", round.created_at.timestamp()));

            if round.is_complete() && !round.results.is_empty() {
                response.push_str("**🏆 Results Available:**\n");
                for (i, result) in round.results.iter().take(3).enumerate() {
                    let rank_emoji = match i {
                        0 => "🥇",
                        1 => "🥈", 
                        2 => "🥉",
                        _ => "🏅",
                    };
                    response.push_str(&format!(
                        "{} {} - ${:.2}\n",
                        rank_emoji,
                        result.participant.username,
                        result.payout.unwrap_or(0.0)
                    ));
                }

                if round.results.len() > 3 {
                    response.push_str(&format!("... and {} more participants\n", round.results.len() - 3));
                }
            } else if !round.participants.is_empty() {
                response.push_str("**👥 Participants:**\n");
                for participant in &round.participants {
                    let status = if participant.verified { "✅" } else { "⏳" };
                    response.push_str(&format!("{} {}\n", status, participant.username));
                }
            }

            msg.reply(ctx, response).await?;
        }
        Err(_) => {
            msg.reply(ctx, format!("❌ **Round not found:** `{}`", round_id)).await?;
        }
    }

    Ok(())
}

#[command]
#[aliases("hash")]
#[description = "Generate a commitment hash for your guess"]
#[usage = "!generate_hash <guess> [salt]"]
async fn generate_hash(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guess = match args.single::<String>() {
        Ok(g) => g,
        Err(_) => {
            msg.reply(ctx, "❌ **Error:** Please provide a guess").await?;
            return Ok(());
        }
    };

    let data = ctx.data.read().await;
    let bot_data = data.get::<BotData>().unwrap();

    let salt = args.single::<String>()
        .unwrap_or_else(|_| bot_data.commitment_generator.generate_salt());

    match bot_data.commitment_generator.generate(&guess, &salt) {
        Ok(hash) => {
            msg.reply(ctx, format!("🔐 **Commitment Hash Generated!**\n\n**Hash:** `{}`\n**Salt:** `{}`\n\n⚠️ **Important:** Save your salt! You'll need it to reveal your guess.\n\nUse `!commit` with this hash to participate in a round.", hash, salt)).await?;
        }
        Err(e) => {
            msg.reply(ctx, format!("❌ **Error generating hash:** {}", e)).await?;
        }
    }

    Ok(())
}

#[command]
#[aliases("list")]
#[description = "List all available rounds"]
#[usage = "!list_rounds"]
async fn list_rounds(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let bot_data = data.get::<BotData>().unwrap();
    let mut processor = bot_data.round_processor.lock().await;

    match processor.get_round_stats_all() {
        Ok(stats_list) => {
            if stats_list.is_empty() {
                msg.reply(ctx, "📋 **No rounds available yet.**\n\nUse `!start_round` to create a new round!").await?;
                return Ok(());
            }

            let mut response = "📋 **Available Rounds:**\n\n".to_string();

            for stats in stats_list {
                let status_emoji = if stats.is_complete { "✅" } else { "🟢" };

                response.push_str(&format!(
                    "{} **`{}`** - {} participants - ${:.2} prize pool\n",
                    status_emoji,
                    stats.round_id,
                    stats.total_participants,
                    stats.total_prize_pool
                ));
            }

            response.push_str("\nUse `!status <round_id>` for detailed information about a specific round.");
            msg.reply(ctx, response).await?;
        }
        Err(e) => {
            msg.reply(ctx, format!("❌ **Error loading rounds:** {}", e)).await?;
        }
    }

    Ok(())
}

#[command]
#[aliases("help")]
#[description = "Show help information about the bot"]
#[usage = "!help"]
async fn help_cmd(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let help_text = r#"🤖 **RealMIR Discord Bot - Help**

**🎯 How to Play:**
1. Admin creates a round with `!start_round`
2. Generate a commitment hash with `!generate_hash`
3. Submit your commitment with `!commit`
4. Reveal your guess with `!reveal` when ready
5. Admin processes scoring with `!score_round`

**📋 Available Commands:**

**🎮 Player Commands:**
• `!generate_hash <guess> [salt]` - Generate commitment hash
• `!commit <round_id> <hash>` - Submit your commitment
• `!reveal <round_id> <guess> <salt>` - Reveal and verify your guess
• `!status <round_id>` - Check round status and results
• `!list_rounds` - Show all available rounds

**⚙️ Admin Commands:**
• `!start_round <id> <title> <description> <image> [prize]` - Create new round
• `!score_round <round_id>` - Process scoring and payouts

**📖 Other:**
• `!help` - Show this help message

**💡 Tips:**
- Save your salt when generating hashes!
- You must reveal with the exact same guess and salt
- Rounds use CLIP AI for objective scoring
- Prize pools are distributed based on similarity rankings

**🔗 Game Flow:**
Start Round → Generate Hash → Commit → Reveal → Score → Results"#;

    msg.reply(ctx, help_text).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    // Load Discord token from environment
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected DISCORD_TOKEN environment variable");

    // Initialize bot data
    let bot_data = match BotData::new() {
        Ok(data) => Arc::new(data),
        Err(e) => {
            eprintln!("Failed to initialize bot data: {}", e);
            std::process::exit(1);
        }
    };

    let mut framework = StandardFramework::new();
    framework.configure(serenity::framework::standard::Configuration::new().prefix("!"));
    let framework = framework.group(&GENERAL_GROUP);

    // Create Discord client
    let intents = GatewayIntents::GUILD_MESSAGES 
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // Add bot data to client context
    {
        let mut data = client.data.write().await;
        data.insert::<BotData>(bot_data);
    }

    println!("🚀 Starting RealMIR Discord Bot...");

    // Start the client
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}