# Orb Communication

This project provides a **client-server system** to manage and interact with Orbs using Zenoh for communication. The client allows discovery, querying, and sending commands to Orbs, while the server handles responses and command execution.

## Table of Contents
- [Overview](#overview)
- [Usage](#usage)
  - [Running the Client](#running-the-client)
  - [Running the Server](#running-the-server)
- [Available Commands](#available-commands)
- [Adding New Queries/Commands](#adding-new-queriescommands)
- [Project Structure](#project-structure)
- [Dependencies](#dependencies)

---

## Overview
The system operates in two parts:
1. **Client**: Sends requests to discover Orbs, queries information, or executes commands.
2. **Server**: Runs on each Orb, responds to queries, and performs requested commands.

Zenoh is used as the communication layer for lightweight pub-sub and query capabilities.

## Usage

### Running the Client

1. Build the project:
   ```sh
   cargo build --release
   ```
2. Run the client:
   ```sh
   ./client <command>
   ```

### Running the Server

1. Bbuild the server and deploy on your Orb:
   ```sh
    cargo zigbuild --target aarch64-unknown-linux-gnu --release --bin server -p zenoh-orbcomm
   ```
2. Start the server:
   ```sh
   ./server
   ```

The server automatically broadcasts its Orb ID for discovery and listens for incoming commands.

## Available Client Commands

### `Ping`
Discover all available Orbs on the network.
```sh
./client ping
```

### `Query`
Query specific information from an Orb.
- Example: Query the hardware version of an Orb with ID `orb123`
```sh
./client query --id orb123 hardware_version
```

### `Command`
Execute a command on an Orb.
- Example: Reboot the Orb with ID `orb123`
```sh
./client command --id orb123 reboot
```

Available commands:
- `reboot` - Reboots the Orb.
- `shutdown` - Shuts down the Orb.
- `reset_gimbal` - Resets the Orb's gimbal.

## Adding New Queries/Commands

### Adding a New Query
1. Open `orb_actions.rs`.
2. Add a new variant to the `Query` enum:
   ```rust
   #[derive(Debug)]
   pub enum Query {
       Name,
       Id,
       HardwareVersion,
       NewQuery, // Add your query here
   }
   ```
3. Update the `from_str` and `to_key` methods:
   ```rust
   pub fn from_str(s: &str) -> Option<Self> {
       match s {
           "name" => Some(Query::Name),
           "id" => Some(Query::Id),
           "hardware_version" => Some(Query::HardwareVersion),
           "new_query" => Some(Query::NewQuery), // Add string mapping
           _ => None,
       }
   }

   pub fn to_key(&self, orb_id: &str) -> String {
       match self {
           Query::Name => format!("orb/{}/name", orb_id),
           Query::Id => format!("orb/{}/id", orb_id),
           Query::HardwareVersion => format!("orb/{}/hardware_version", orb_id),
           Query::NewQuery => format!("orb/{}/new_query", orb_id), // Define the key
       }
   }
   ```

### Adding a New Command
1. Open `orb_actions.rs`.
2. Add a new variant to the `Command` enum:
   ```rust
   #[derive(Debug)]
   pub enum Command {
       Reboot,
       Shutdown,
       ResetGimbal,
       NewCommand, // Add your command here
   }
   ```
3. Update the `from_str` and `to_key` methods:
   ```rust
   pub fn from_str(s: &str) -> Option<Self> {
       match s {
           "reboot" => Some(Command::Reboot),
           "shutdown" => Some(Command::Shutdown),
           "reset_gimbal" => Some(Command::ResetGimbal),
           "new_command" => Some(Command::NewCommand), // Add string mapping
           _ => None,
       }
   }

   pub fn to_key(&self, orb_id: &str) -> String {
       match self {
           Command::Reboot => format!("orb/{}/command/reboot", orb_id),
           Command::Shutdown => format!("orb/{}/command/shutdown", orb_id),
           Command::ResetGimbal => format!("orb/{}/command/reset_gimbal", orb_id),
           Command::NewCommand => format!("orb/{}/command/new_command", orb_id), // Define the key
       }
   }
   ```

4. Update `server.rs` to handle the new command in `handle_commands`.

## Project Structure
```
.
├── client.rs       # Client implementation for queries, commands, and discovery
├── server.rs       # Server implementation to handle queries and commands
├── orb_actions.rs  # Definitions of Query and Command enums
├── Cargo.toml      # Project dependencies and metadata
└── README.md       # Documentation
```

## Dependencies
This project relies on the following crates:
- **Zenoh**: Lightweight pub-sub and query framework.
- **Tokio**: Asynchronous runtime for Rust.
- **Clap**: Command-line argument parsing.
- **Tracing**: Structured logging support.
- **Anyhow**: Error handling.
