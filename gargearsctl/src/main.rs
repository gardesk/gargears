//! CLI control utility for gargears daemon

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use gargears_ipc::{socket_path, Command, Response};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

#[derive(Parser, Debug)]
#[command(name = "gargearsctl")]
#[command(about = "Control the gargears daemon")]
struct Args {
    #[command(subcommand)]
    command: CtlCommand,
}

#[derive(Subcommand, Debug)]
enum CtlCommand {
    /// Show the gargears window
    Show,
    /// Hide the gargears window
    Hide,
    /// Toggle gargears window visibility
    Toggle,
    /// Get daemon status
    Status,
    /// Quit the daemon
    Quit,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let command = match args.command {
        CtlCommand::Show => Command::Show,
        CtlCommand::Hide => Command::Hide,
        CtlCommand::Toggle => Command::Toggle,
        CtlCommand::Status => Command::Status,
        CtlCommand::Quit => Command::Quit,
    };

    let response = send_command(&command)?;

    if response.success {
        if let Some(data) = response.data {
            println!("{}", serde_json::to_string_pretty(&data)?);
        } else {
            println!("OK");
        }
    } else {
        eprintln!("Error: {}", response.error.unwrap_or_else(|| "Unknown error".into()));
        std::process::exit(1);
    }

    Ok(())
}

fn send_command(command: &Command) -> Result<Response> {
    let path = socket_path();

    let mut stream = UnixStream::connect(&path)
        .with_context(|| format!("Failed to connect to gargears daemon at {:?}", path))?;

    // Send command
    let json = serde_json::to_string(command)?;
    writeln!(stream, "{}", json)?;
    stream.flush()?;

    // Read response
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let response: Response = serde_json::from_str(&line)
        .with_context(|| "Failed to parse daemon response")?;

    Ok(response)
}
