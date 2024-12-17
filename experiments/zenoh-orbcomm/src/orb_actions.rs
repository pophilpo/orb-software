use std::str::FromStr;

/// Enum representing possible query types for an orb.
#[derive(Debug)]
pub enum Query {
    Name,
    Id,
    HardwareVersion,
}

impl FromStr for Query {
    type Err = ();

    /// Converts a string into a corresponding `Query` enum variant.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(Query::Name),
            "id" => Ok(Query::Id),
            "hardware_version" => Ok(Query::HardwareVersion),
            _ => Err(()),
        }
    }
}

impl Query {
    /// Generates the corresponding key for a query using the orb ID.
    pub fn to_key(&self, orb_id: &str) -> String {
        match self {
            Query::Name => format!("orb/{}/name", orb_id),
            Query::Id => format!("orb/{}/id", orb_id),
            Query::HardwareVersion => format!("orb/{}/hardware_version", orb_id),
        }
    }
}

/// Enum representing available commands that can be sent to an orb.
#[derive(Debug)]
pub enum Command {
    Reboot,
    Shutdown,
    ResetGimbal,
}

impl FromStr for Command {
    type Err = ();

    /// Converts a string into a corresponding `Command` enum variant.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "reboot" => Ok(Command::Reboot),
            "shutdown" => Ok(Command::Shutdown),
            "reset_gimbal" => Ok(Command::ResetGimbal),
            _ => Err(()),
        }
    }
}

impl Command {
    /// Generates the corresponding key for a command using the orb ID.
    pub fn to_key(&self, orb_id: &str) -> String {
        match self {
            Command::Reboot => format!("orb/{}/command/reboot", orb_id),
            Command::Shutdown => format!("orb/{}/command/shutdown", orb_id),
            Command::ResetGimbal => format!("orb/{}/command/reset_gimbal", orb_id),
        }
    }
}

