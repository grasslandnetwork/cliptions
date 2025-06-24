THIS SHOULD BE A LINTER ERROR//! RealMir CLI - Unified command-line interface for RealMir prediction markets
//! 
//! This enhanced CLI provides comprehensive functionality for managing RealMir rounds,
//! calculating scores, processing payouts, and verifying commitments with improved
//! error handling, batch processing, and user experience.

use std::process;
use std::path::PathBuf;
use clap::{Parser, Subcommand, Args};
use colored::Colorize;

use realmir_core::embedder::{MockEmbedder, ClipEmbedder, EmbedderTrait};
use realmir_core::scoring::{BaselineAdjustedStrategy, ScoreValidator, calculate_rankings, calculate_payouts};
use realmir_core::round::RoundProcessor;
use realmir_core::config::ConfigManager;
use realmir_core::types::ScoringResult;

#[derive(Parser)]
#[command(name = "realmir")]
#[command(about = "RealMir prediction market CLI - comprehensive tooling for rounds, scoring, and verification")]
#[command(version = "2.0")]
#[command(long_about = "
RealMir CLI provides comprehensive functionality for managing prediction market rounds:

• Calculate scores and payouts with CLIP embeddings
• Verify cryptographic commitments 
• Process batch operations across multiple rounds
• Manage configuration and settings
• Monitor round statistics and performance

Examples:
  realmir scores calculate --image target.jpg --prize-pool 100.0 \"guess1\" \"guess2\" \"guess3\"
  realmir rounds process --round round1 --rounds-file rounds.json
  realmir verify commitments --round round1 --verbose
  realmir config set daily-spending-limit 10.0
")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Score calculation and payout operations
    Scores {
        #[command(subcommand)]
        action: ScoreActions,
    },
    /// Round management and processing
    Rounds {
        #[command(subcommand)]
        action: RoundActions,
    },
    /// Commitment verification operations
    Verify {
        #[command(subcommand)]
        action: VerifyActions,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigActions,
    },
    /// System information and statistics
    Info {
        #[command(subcommand)]
        action: InfoActions,
    },
}

#[derive(Subcommand)]
enum ScoreActions {
    /// Calculate scores and payouts for guesses
    Calculate(CalculateArgs),
    /// Batch process multiple score calculations
    Batch(BatchScoreArgs),
}

#[derive(Subcommand)]
enum RoundActions {
    /// Process payouts for a specific round
    Process(ProcessRoundArgs),
    /// Process all rounds in batch
    ProcessAll(ProcessAllRoundsArgs),
    /// Show round statistics
    Stats(RoundStatsArgs),
    /// List available rounds
    List(ListRoundsArgs),
}

#[derive(Subcommand)]
enum VerifyActions {
    /// Verify commitments for a round
    Commitments(VerifyCommitmentsArgs),
    /// Batch verify multiple rounds
    Batch(BatchVerifyArgs),
}

#[derive(Subcommand)]
enum ConfigActions {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set(SetConfigArgs),
    /// Validate configuration
    Validate,
    /// Reset configuration to defaults
    Reset,
}

#[derive(Subcommand)]
enum InfoActions {
    /// Show system information
    System,
    /// Show embedding model status
    Embedders,
    /// Show performance benchmarks
    Benchmark,
}

#[derive(Args)]
struct CalculateArgs {
    /// Path to the target image
    #[arg(long, short)]
    image: String,

    /// Prize pool amount
    #[arg(long, short)]
    prize_pool: f64,

    /// List of guesses to rank
    guesses: Vec<String>,

    /// Output format (table, json, csv)
    #[arg(long, default_value = "table")]
    output: String,

    /// Save results to file
    #[arg(long, short)]
    output_file: Option<PathBuf>,

    /// Use CLIP embedder instead of mock
    #[arg(long)]
    use_clip: bool,

    /// CLIP model path (if using real CLIP)
    #[arg(long)]
    clip_model: Option<PathBuf>,
}

#[derive(Args)]
struct BatchScoreArgs {
    /// Directory containing target images
    #[arg(long, short)]
    images_dir: PathBuf,

    /// Prize pool amount (same for all)
    #[arg(long, short)]
    prize_pool: f64,

    /// File containing guesses (JSON format)
    #[arg(long, short)]
    guesses_file: PathBuf,

    /// Output directory for results
    #[arg(long, short)]
    output_dir: PathBuf,

    /// Number of parallel workers
    #[arg(long, default_value = "4")]
    workers: usize,
}

#[derive(Args)]
struct ProcessRoundArgs {
    /// Round ID to process
    #[arg(long, short)]
    round: String,

    /// Path to rounds file
    #[arg(long, default_value = "rounds.json")]
    rounds_file: PathBuf,

    /// Output format (table, json, csv)
    #[arg(long, default_value = "table")]
    output: String,

    /// Save results to file
    #[arg(long, short)]
    output_file: Option<PathBuf>,
}

#[derive(Args)]
struct ProcessAllRoundsArgs {
    /// Path to rounds file
    #[arg(long, default_value = "rounds.json")]
    rounds_file: PathBuf,

    /// Output directory for results
    #[arg(long, short)]
    output_dir: Option<PathBuf>,

    /// Number of parallel workers
    #[arg(long, default_value = "4")]
    workers: usize,

    /// Continue on errors
    #[arg(long)]
    continue_on_error: bool,
}

#[derive(Args)]
struct RoundStatsArgs {
    /// Round ID to analyze
    #[arg(long, short)]
    round: Option<String>,

    /// Path to rounds file
    #[arg(long, default_value = "rounds.json")]
    rounds_file: PathBuf,

    /// Show detailed statistics
    #[arg(long)]
    detailed: bool,
}

#[derive(Args)]
struct ListRoundsArgs {
    /// Path to rounds file
    #[arg(long, default_value = "rounds.json")]
    rounds_file: PathBuf,

    /// Show only active rounds
    #[arg(long)]
    active_only: bool,
}

#[derive(Args)]
struct VerifyCommitmentsArgs {
    /// Round ID to verify
    #[arg(long, short)]
    round: String,

    /// Path to rounds file
    #[arg(long, default_value = "rounds.json")]
    rounds_file: PathBuf,

    /// Show detailed verification results
    #[arg(long)]
    detailed: bool,

    /// Output format (table, json, csv)
    #[arg(long, default_value = "table")]
    output: String,
}

#[derive(Args)]
struct BatchVerifyArgs {
    /// Path to rounds file
    #[arg(long, default_value = "rounds.json")]
    rounds_file: PathBuf,

    /// Output directory for results
    #[arg(long, short)]
    output_dir: Option<PathBuf>,

    /// Continue on errors
    #[arg(long)]
    continue_on_error: bool,
}

#[derive(Args)]
struct SetConfigArgs {
    /// Configuration key to set
    key: String,

    /// Configuration value
    value: String,
}

fn main() {
    let cli = Cli::parse();

    // Initialize colored output
    if cli.no_color {
        colored::control::set_override(false);
    }

    // Load configuration if specified
    let config_manager = if let Some(config_path) = &cli.config {
        match ConfigManager::with_path(config_path) {
            Ok(manager) => Some(manager),
            Err(e) => {
                eprintln!("{} Failed to load config from {}: {}", 
                    "Error:".red().bold(), 
                    config_path.display(), 
                    e
                );
                process::exit(1);
            }
        }
    } else {
        match ConfigManager::new() {
            Ok(manager) => Some(manager),
            Err(_) => {
                if cli.verbose {
                    eprintln!("{} No configuration file found, using defaults", 
                        "Warning:".yellow().bold()
                    );
                }
                None
            }
        }
    };

    let result = match cli.command {
        Commands::Scores { action } => handle_scores(action, &cli, config_manager.as_ref()),
        Commands::Rounds { action } => handle_rounds(action, &cli, config_manager.as_ref()),
        Commands::Verify { action } => handle_verify(action, &cli, config_manager.as_ref()),
        Commands::Config { action } => handle_config(action, &cli, config_manager.as_ref()),
        Commands::Info { action } => handle_info(action, &cli, config_manager.as_ref()),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}

fn handle_scores(action: ScoreActions, cli: &Cli, _config: Option<&ConfigManager>) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ScoreActions::Calculate(args) => {
            if cli.verbose {
                println!("{} Calculating scores for {} guesses", 
                    "Info:".blue().bold(), 
                    args.guesses.len()
                );
            }

            // Validate inputs
            if args.prize_pool <= 0.0 {
                return Err("Prize pool must be greater than zero".into());
            }

            if args.guesses.is_empty() {
                return Err("At least one guess must be provided".into());
            }

            // Create embedder and validator based on user preference
            let (ranked_results, payouts) = if args.use_clip {
                if let Some(model_path) = &args.clip_model {
                    match ClipEmbedder::from_path(&model_path.to_string_lossy()) {
                        Ok(embedder) => {
                            if cli.verbose {
                                println!("{} Using CLIP embedder from {}", 
                                    "Info:".blue().bold(), 
                                    model_path.display()
                                );
                            }
                            let strategy = BaselineAdjustedStrategy::new();
                            let validator = ScoreValidator::new(embedder, strategy);
                            let rankings = calculate_rankings(&args.image, &args.guesses, &validator)?;
                            let payouts = calculate_payouts(&rankings, args.prize_pool)?;
                            (rankings, payouts)
                        }
                        Err(e) => {
                            eprintln!("{} Failed to load CLIP model: {}", 
                                "Warning:".yellow().bold(), 
                                e
                            );
                            eprintln!("{} Falling back to MockEmbedder", 
                                "Info:".blue().bold()
                            );
                            let embedder = MockEmbedder::clip_like();
                            let strategy = BaselineAdjustedStrategy::new();
                            let validator = ScoreValidator::new(embedder, strategy);
                            let rankings = calculate_rankings(&args.image, &args.guesses, &validator)?;
                            let payouts = calculate_payouts(&rankings, args.prize_pool)?;
                            (rankings, payouts)
                        }
                    }
                } else {
                    match ClipEmbedder::new() {
                        Ok(embedder) => {
                            if cli.verbose {
                                println!("{} Using default CLIP embedder", 
                                    "Info:".blue().bold()
                                );
                            }
                            let strategy = BaselineAdjustedStrategy::new();
                            let validator = ScoreValidator::new(embedder, strategy);
                            let rankings = calculate_rankings(&args.image, &args.guesses, &validator)?;
                            let payouts = calculate_payouts(&rankings, args.prize_pool)?;
                            (rankings, payouts)
                        }
                        Err(e) => {
                            eprintln!("{} Failed to load default CLIP model: {}", 
                                "Warning:".yellow().bold(), 
                                e
                            );
                            eprintln!("{} Falling back to MockEmbedder", 
                                "Info:".blue().bold()
                            );
                            let embedder = MockEmbedder::clip_like();
                            let strategy = BaselineAdjustedStrategy::new();
                            let validator = ScoreValidator::new(embedder, strategy);
                            let rankings = calculate_rankings(&args.image, &args.guesses, &validator)?;
                            let payouts = calculate_payouts(&rankings, args.prize_pool)?;
                            (rankings, payouts)
                        }
                    }
                }
            } else {
                if cli.verbose {
                    println!("{} Using MockEmbedder for testing", 
                        "Info:".blue().bold()
                    );
                }
                let embedder = MockEmbedder::clip_like();
                let strategy = BaselineAdjustedStrategy::new();
                let validator = ScoreValidator::new(embedder, strategy);
                let rankings = calculate_rankings(&args.image, &args.guesses, &validator)?;
                let payouts = calculate_payouts(&rankings, args.prize_pool)?;
                (rankings, payouts)
            };

            // Rankings and payouts already calculated above
            if cli.verbose {
                println!("{} Calculations completed", "Info:".blue().bold());
            }

            // Display results
            display_score_results(&ranked_results, &payouts, args.prize_pool, &args.output, cli.verbose)?;

            // Save to file if requested
            if let Some(output_file) = &args.output_file {
                save_score_results(&ranked_results, &payouts, args.prize_pool, output_file, &args.output)?;
                println!("{} Results saved to {}", 
                    "Success:".green().bold(), 
                    output_file.display()
                );
            }

            Ok(())
        }
        ScoreActions::Batch(_args) => {
            // TODO: Implement batch processing
            Err("Batch score processing not yet implemented".into())
        }
    }
}

fn handle_rounds(action: RoundActions, cli: &Cli, _config: Option<&ConfigManager>) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        RoundActions::Process(args) => {
            if cli.verbose {
                println!("{} Processing round: {}", 
                    "Info:".blue().bold(), 
                    args.round
                );
            }

            let embedder = MockEmbedder::clip_like();
            let strategy = BaselineAdjustedStrategy::new();
            let mut processor = RoundProcessor::new(args.rounds_file.to_string_lossy().to_string(), embedder, strategy);

            let results = processor.process_round_payouts(&args.round)?;

            display_round_results(&results, &args.output, cli.verbose)?;

            if let Some(output_file) = &args.output_file {
                save_round_results(&results, output_file, &args.output)?;
                println!("{} Results saved to {}", 
                    "Success:".green().bold(), 
                    output_file.display()
                );
            }

            Ok(())
        }
        RoundActions::ProcessAll(_args) => {
            // TODO: Implement batch round processing
            Err("Batch round processing not yet implemented".into())
        }
        RoundActions::Stats(_args) => {
            // TODO: Implement round statistics
            Err("Round statistics not yet implemented".into())
        }
        RoundActions::List(_args) => {
            // TODO: Implement round listing
            Err("Round listing not yet implemented".into())
        }
    }
}

fn handle_verify(action: VerifyActions, cli: &Cli, _config: Option<&ConfigManager>) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        VerifyActions::Commitments(args) => {
            if cli.verbose {
                println!("{} Verifying commitments for round: {}", 
                    "Info:".blue().bold(), 
                    args.round
                );
            }

            let embedder = MockEmbedder::clip_like();
            let strategy = BaselineAdjustedStrategy::new();
            let mut processor = RoundProcessor::new(args.rounds_file.to_string_lossy().to_string(), embedder, strategy);

            // Load rounds to get participant info
            processor.load_rounds()?;

            let round = processor.get_round(&args.round)?;
            
            println!("{} Verifying commitments for round: {}", 
                "Info:".blue().bold(), 
                args.round
            );
            println!("Round title: {}", round.title);
            println!("Participants: {}", round.participants.len());

            if round.participants.is_empty() {
                println!("{} No participants to verify.", "Info:".blue().bold());
                return Ok(());
            }

            // Verify commitments
            let results = processor.verify_commitments(&args.round)?;
            
            display_verification_results(&results, &round.participants, &args.output, args.detailed, cli.verbose)?;

            Ok(())
        }
        VerifyActions::Batch(_args) => {
            // TODO: Implement batch verification
            Err("Batch verification not yet implemented".into())
        }
    }
}

fn handle_config(action: ConfigActions, cli: &Cli, config: Option<&ConfigManager>) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ConfigActions::Show => {
            if let Some(config_manager) = config {
                println!("{} Current Configuration:", "Info:".blue().bold());
                // TODO: Display configuration in a nice format
                println!("Configuration display not yet implemented");
            } else {
                println!("{} No configuration loaded", "Info:".blue().bold());
            }
            Ok(())
        }
        ConfigActions::Set(_args) => {
            // TODO: Implement configuration setting
            Err("Configuration setting not yet implemented".into())
        }
        ConfigActions::Validate => {
            if let Some(config_manager) = config {
                // TODO: Validate configuration
                println!("{} Configuration validation not yet implemented", "Info:".blue().bold());
            } else {
                println!("{} No configuration to validate", "Warning:".yellow().bold());
            }
            Ok(())
        }
        ConfigActions::Reset => {
            // TODO: Implement configuration reset
            Err("Configuration reset not yet implemented".into())
        }
    }
}

fn handle_info(action: InfoActions, cli: &Cli, _config: Option<&ConfigManager>) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        InfoActions::System => {
            println!("{} RealMir System Information", "Info:".blue().bold());
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            println!("Rust Version: {}", env!("RUSTC"));
            // TODO: Add more system information
            Ok(())
        }
        InfoActions::Embedders => {
            println!("{} Available Embedders:", "Info:".blue().bold());
            println!("• MockEmbedder: ✓ Available (for testing)");
            
            // Test CLIP embedder availability
            match ClipEmbedder::new() {
                Ok(_) => println!("• ClipEmbedder: ✓ Available"),
                Err(_) => println!("• ClipEmbedder: ✗ Not available (no model files)"),
            }
            Ok(())
        }
        InfoActions::Benchmark => {
            // TODO: Implement performance benchmarks
            Err("Performance benchmarks not yet implemented".into())
        }
    }
}

fn display_score_results(
    ranked_results: &[(String, f64)], 
    payouts: &[f64], 
    prize_pool: f64, 
    output_format: &str,
    verbose: bool
) -> Result<(), Box<dyn std::error::Error>> {
    match output_format {
        "table" => {
            println!("\n{}", "Rankings and Payouts:".bold());
            println!("{}", "-".repeat(60));
            
            for (i, ((guess, similarity), payout)) in ranked_results.iter().zip(payouts.iter()).enumerate() {
                println!("{}. \"{}\"", 
                    format!("{}", i + 1).bold().blue(), 
                    guess
                );
                println!("   Similarity score: {:.4}", similarity);
                println!("   Payout: {:.9}", payout);
                println!();
            }
            
            println!("{}", "-".repeat(60));
            println!("Total prize pool: {:.9}", prize_pool);
            println!("Total payout: {:.9}", payouts.iter().sum::<f64>());
        }
        "json" => {
            // TODO: Implement JSON output
            return Err("JSON output format not yet implemented".into());
        }
        "csv" => {
            // TODO: Implement CSV output
            return Err("CSV output format not yet implemented".into());
        }
        _ => {
            return Err(format!("Unsupported output format: {}", output_format).into());
        }
    }
    Ok(())
}

fn save_score_results(
    _ranked_results: &[(String, f64)], 
    _payouts: &[f64], 
    _prize_pool: f64, 
    _output_file: &PathBuf,
    _format: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement file saving
    Err("File saving not yet implemented".into())
}

fn display_round_results(
    _results: &[ScoringResult], 
    _output_format: &str,
    _verbose: bool
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement round results display
    Err("Round results display not yet implemented".into())
}

fn save_round_results(
    _results: &[ScoringResult], 
    _output_file: &PathBuf,
    _format: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement round results saving
    Err("Round results saving not yet implemented".into())
}

fn display_verification_results(
    results: &[bool],
    participants: &[realmir_core::types::Participant],
    output_format: &str,
    detailed: bool,
    verbose: bool
) -> Result<(), Box<dyn std::error::Error>> {
    let valid_count = results.iter().filter(|&&r| r).count();
    let total_count = results.len();

    match output_format {
        "table" => {
            println!("\n{}", "Verification Results:".bold());
            println!("Valid commitments: {}/{}", 
                format!("{}", valid_count).green().bold(),
                total_count
            );

            if detailed {
                println!("\n{}", "Detailed Results:".bold());
                for (i, (participant, &is_valid)) in participants.iter().zip(results.iter()).enumerate() {
                    let status = if is_valid { 
                        "✓ VALID".green().bold() 
                    } else { 
                        "✗ INVALID".red().bold() 
                    };
                    
                    println!("  {}. {} ({}): {}", 
                        i + 1, 
                        participant.username,
                        participant.user_id,
                        status
                    );
                    
                    if verbose {
                        println!("     Guess: \"{}\"", participant.guess.text);
                        println!("     Commitment: {}", participant.commitment);
                        if let Some(salt) = &participant.salt {
                            println!("     Salt: {}", salt);
                        } else {
                            println!("     Salt: [NOT PROVIDED]");
                        }
                    }
                    println!();
                }
            }

            if valid_count == total_count {
                println!("{} All commitments verified successfully!", 
                    "Success:".green().bold()
                );
            } else {
                println!("{} {} commitment(s) failed verification.", 
                    "Warning:".yellow().bold(),
                    total_count - valid_count
                );
            }
        }
        "json" | "csv" => {
            return Err(format!("Output format '{}' not yet implemented", output_format).into());
        }
        _ => {
            return Err(format!("Unsupported output format: {}", output_format).into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Test basic command parsing
        let cli = Cli::try_parse_from(&[
            "realmir",
            "scores",
            "calculate",
            "--image", "test.jpg",
            "--prize-pool", "100.0",
            "guess1", "guess2"
        ]);
        
        assert!(cli.is_ok());
    }

    #[test]
    fn test_verbose_flag() {
        let cli = Cli::try_parse_from(&[
            "realmir",
            "--verbose",
            "info",
            "system"
        ]).unwrap();
        
        assert!(cli.verbose);
    }

    #[test]
    fn test_config_path() {
        let cli = Cli::try_parse_from(&[
            "realmir",
            "--config", "/path/to/config.yaml",
            "info",
            "system"
        ]).unwrap();
        
        assert_eq!(cli.config, Some(PathBuf::from("/path/to/config.yaml")));
    }
}