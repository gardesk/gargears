mod app;
mod config;
mod daemon;
mod ipc;
mod panels;
mod ui;

use anyhow::Result;
use clap::Parser;
use std::thread;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser, Debug)]
#[command(name = "gargears")]
#[command(about = "Centralized GUI for gardesk configuration")]
struct Args {
    /// Run as daemon (persistent, with tray integration)
    #[arg(short, long)]
    daemon: bool,

    /// Don't fork to background (daemon mode only)
    #[arg(long)]
    no_fork: bool,

    /// Initial panel to show
    #[arg(short, long)]
    panel: Option<String>,
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    tracing::info!("Starting gargears");

    if args.daemon {
        run_daemon(&args)
    } else {
        run_gui(&args)
    }
}

fn run_daemon(args: &Args) -> Result<()> {
    // Check if another daemon is already running
    let socket_path = gargears_ipc::socket_path();
    if socket_path.exists() {
        // Try to connect to see if it's actually running
        if std::os::unix::net::UnixStream::connect(&socket_path).is_ok() {
            tracing::error!("Another gargears daemon is already running");
            return Err(anyhow::anyhow!("Daemon already running"));
        }
        // Stale socket, remove it
        let _ = std::fs::remove_file(&socket_path);
    }

    // Create command channel
    let (command_tx, command_rx) = daemon::create_command_channel();

    // Create app in daemon mode (starts hidden)
    let initial_panel = args.panel.as_deref();
    let mut app = app::App::new_daemon(initial_panel)?;

    // Get visibility state for IPC server
    let visible_state = app.visible_state();

    // Start IPC server in background thread
    let ipc_server = daemon::IpcServer::new(command_tx)?;
    thread::spawn(move || {
        ipc_server.run(visible_state);
    });

    tracing::info!("gargears daemon started");
    tracing::info!("Use 'gargearsctl show' or 'gargearsctl toggle' to show the window");

    // Send startup notification
    daemon::notify_startup();

    // Run app event loop
    app.run_daemon(command_rx)
}

fn run_gui(args: &Args) -> Result<()> {
    let initial_panel = args.panel.as_deref();
    let mut app = app::App::new(initial_panel)?;
    app.run()
}
