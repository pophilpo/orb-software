use clap::{Parser, Subcommand};
use zenoh_orbcomm::orb_actions::{Query, Command};
use std::time::Duration;
use std::str::FromStr;
use tokio::time::timeout;
use zenoh::config::Config;
use colored::*;

/// CLI structure for the Orb client.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands, // Defines the type of command to execute
}

/// Enum defining supported CLI commands.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Discover all orb IDs available on the network.
    Ping,
    /// Query a specific orb for information.
    Query {
        /// The orb ID to query.
        #[arg(long)]
        id: String,
        /// The type of query (e.g., name, id, hardware_version).
        query_type: String,
    },
    /// Execute a command on a specified orb.
    Command {
        /// The orb ID to target.
        #[arg(long)]
        id: String,
        /// The type of command to execute (e.g., reboot, shutdown).
        command_type: String,
    },
}

/// Main asynchronous function for the client program.
#[tokio::main]
async fn main() -> zenoh::Result<()> {
    zenoh::init_log_from_env_or("error");

    println!("Opening Zenoh session...");
    let session = zenoh::open(Config::default()).await?;

    let cli = Cli::parse();
    match cli.command {
        Commands::Ping => {
            discover_orbs(&session).await?;
        }
        Commands::Query { id, query_type } => {
            if let Ok(query) = Query::from_str(&query_type) {
                let key = query.to_key(&id);
                perform_query(&session, &key).await?;
            } else {
                eprintln!("Invalid query type: {}", query_type);
            }
        }
        Commands::Command { id, command_type } => {
            if let Ok(command) = Command::from_str(&command_type) {
                let key = command.to_key(&id);
                perform_command(&session, &key).await?;
            } else {
                eprintln!("Invalid command type: {}", command_type);
            }
        }
    }

    Ok(())
}

/// Discover orbs by subscribing to 'orb/id' topics and listening for IDs.
async fn discover_orbs(session: &zenoh::Session) -> zenoh::Result<()> {
    let subscriber = session
        .declare_subscriber("orb/id")
        .await
        .expect("Failed to declare subscriber");

    println!("Waiting for responses from orbs...");
    let timeout_duration = Duration::from_secs(3);
    let start_time = tokio::time::Instant::now();
    let mut orb_ids = Vec::new();

    while start_time.elapsed() < timeout_duration {
        if let Ok(Ok(sample)) = timeout(Duration::from_millis(1000), subscriber.recv_async()).await {
            let orb_id = String::from_utf8_lossy(&sample.payload().to_bytes()).to_string();
            if !orb_ids.contains(&orb_id) {
                println!("Discovered orb with ID: {}", orb_id.green());
                orb_ids.push(orb_id);
            }
        }
    }

    if orb_ids.is_empty() {
        println!("No orbs found!");
    }

    Ok(())
}

/// Perform a query on a specified key and display the result.
async fn perform_query(session: &zenoh::Session, key: &str) -> zenoh::Result<()> {
    println!("Querying key: {}", key);
    let replies = session.get(key).await?;

    while let Ok(Ok(reply)) = timeout(Duration::from_millis(1000), replies.recv_async()).await {
        if let Ok(sample) = reply.result() {
            println!(
                ">> Received value for {}: {}",
                key.yellow(),
                String::from_utf8_lossy(&sample.payload().to_bytes()).green()
            );
        }
    }

    Ok(())
}

/// Send a command to a specified orb using its command key.
async fn perform_command(session: &zenoh::Session, command_key: &str) -> zenoh::Result<()> {
    println!("Sending command: {}", command_key.yellow());
    session.put(command_key, "").await?;
    println!("Command sent successfully.");
    Ok(())
}

