mod action;
mod app;
mod components;
mod errors;
mod event;
mod game;
mod layout;
mod logging;
mod theme;
mod tui;

use color_eyre::eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("idle-terminal v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("idle-terminal v{}", env!("CARGO_PKG_VERSION"));
        println!("A TUI-based idle game with an IT/DevOps theme\n");
        println!("Usage: idle-terminal [OPTIONS]\n");
        println!("Options:");
        println!("  --reset    Delete save data and start fresh");
        println!("  --version  Print version information");
        println!("  --help     Print this help message");
        return Ok(());
    }

    if args.iter().any(|a| a == "--reset") {
        game::save::delete_save()?;
        println!("Save data deleted. Starting fresh.");
    }

    errors::install_hooks()?;
    logging::init()?;

    let mut app = app::App::new();
    app.run().await?;

    Ok(())
}
