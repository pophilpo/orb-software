
use anyhow::{anyhow, Result};
use zenoh_orbcomm::orb_actions::Query;
use std::collections::HashMap;
use std::process::Command as ShellCommand;
use std::time::Duration;
use tokio::{signal, time};
use tracing::{info, warn};
use tracing_subscriber::FmtSubscriber;
use zenoh::{
    config::Config,
    handlers::FifoChannelHandler,
    pubsub::Subscriber,
    query::{Query as ZenohQuery, Queryable},
    sample::Sample,
};

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    // Retrieve orb properties dynamically using shell commands.
    let orb_id = get_orb_property("orb-id", "UnknownOrb")?;
    let orb_name = get_orb_property("cat /usr/persistent/orb-name", "DevOrb")?;
    let orb_hw_version = get_orb_property("cat /usr/persistent/hardware_version", "UnknownHWVersion")?;

    info!("Starting Orb server with ID: {}", orb_id);

    // Open Zenoh session for pub/sub communication.
    let session = zenoh::open(Config::default())
        .await
        .map_err(|e| anyhow!("Failed to open zenoh session: {}", e))?;

    // Populate orb data with key-value pairs for query responses.
    let mut orb_data = HashMap::new();
    orb_data.insert(Query::Id.to_key(&orb_id), orb_id.clone());
    orb_data.insert(Query::Name.to_key(&orb_id), orb_name);
    orb_data.insert(Query::HardwareVersion.to_key(&orb_id), orb_hw_version);

    // Declare queryable resources for orb properties.
    for key in orb_data.keys() {
        let queryable = session
            .declare_queryable(key)
            .await
            .map_err(|e| anyhow!("Failed to declare queryable for {}: {}", key, e))?;
        info!("Declared queryable for key: {}", key);
        tokio::spawn(handle_queries(queryable, orb_data.clone()));
    }

    // Subscribe to commands directed at this orb.
    let command_subscriber = session
        .declare_subscriber(&format!("orb/{}/command/*", orb_id))
        .await
        .map_err(|e| anyhow!("Failed to declare command subscriber: {}", e))?;

    // Periodically broadcast the orb's ID for discovery purposes.
    let discovery_publisher = session
        .declare_publisher("orb/id")
        .await
        .map_err(|e| anyhow!("Failed to declare discovery publisher: {}", e))?;

    // Orb ID broadcasting loop used for discovery.
    let broadcast_task = tokio::spawn(async move {
        loop {
            if let Err(e) = discovery_publisher.put(orb_id.clone()).await {
                warn!("Failed to publish orb ID: {}", e);
            }
            time::sleep(Duration::from_secs(1)).await;
        }
    });

    // Handle incoming commands or server shutdown.
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received Ctrl+C. Shutting down...");
        }
        res = handle_commands(command_subscriber) => {
            if let Err(e) = res {
                warn!("Command handling ended with an error: {}", e);
            } else {
                warn!("Command handling ended unexpectedly. Shutting down...");
            }
        }
    }

    broadcast_task.abort();
    info!("Server shutdown complete.");
    Ok(())
}

/// Retrieve orb properties using shell commands.
fn get_orb_property(command: &str, default: &str) -> Result<String> {
    let output = ShellCommand::new("sh")
        .arg("-c")
        .arg(command)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        Ok(output) => {
            warn!(
                "Command '{}' failed with status {}: {:?}",
                command,
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
            Ok(default.to_string())
        }
        Err(e) => {
            warn!("Failed to execute command '{}': {}", command, e);
            Ok(default.to_string())
        }
    }
}

/// Handle queries by responding with orb data.
async fn handle_queries(
    queryable: Queryable<FifoChannelHandler<ZenohQuery>>,
    orb_data: HashMap<String, String>,
) -> Result<()> {
    while let Ok(query) = queryable.recv_async().await {
        let requested_key_str = query.key_expr().as_str();
        info!("Received query for key: {}", requested_key_str);

        if let Some(value) = orb_data.get(requested_key_str) {
            if let Err(e) = query.reply(requested_key_str, value.clone()).await {
                warn!("Failed to reply to query for {}: {}", requested_key_str, e);
            }
        } else if let Err(e) = query.reply(requested_key_str, "Error: no such resource".to_string()).await {
            warn!("Failed to reply with error for {}: {}", requested_key_str, e);
        }
    }
    Ok(())
}

/// Handle commands such as shutdown, reboot, or reset gimbal.
async fn handle_commands(command_subscriber: Subscriber<FifoChannelHandler<Sample>>) -> Result<()> {
    while let Ok(command) = command_subscriber.recv_async().await {
        let key = command.key_expr().clone();
        info!("Received command: {}", key);

        let response = if key.ends_with("shutdown") {
            info!("Shutdown command received.");
            run_shell_command("shutdown now")
        } else if key.ends_with("reboot") {
            info!("Reboot command received.");
            run_shell_command("sudo reboot")
        } else if key.ends_with("reset_gimbal") {
            info!("Reset gimbal command received.");
            Ok("Reset gimbal command executed successfully".to_string())
        } else {
            let msg = format!("Error: Unknown command '{}'", key);
            warn!("{}", msg);
            Ok(msg)
        }?;

        info!("Command response: {}", response);
    }
    Ok(())
}

/// Execute shell commands for handling operations.
fn run_shell_command(command: &str) -> Result<String> {
    let output = ShellCommand::new("sh")
        .arg("-c")
        .arg(command)
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
        Ok(output) => Err(anyhow!(
            "Command '{}' failed with status {}: {:?}",
            command,
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )),
        Err(e) => Err(anyhow!("Failed to execute command '{}': {}", command, e)),
    }
}

/// Initialize structured logging with tracing.
fn init_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global tracing subscriber");
}

