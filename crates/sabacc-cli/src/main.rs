mod animation;
mod app;
mod events;
mod ui;
mod widgets;

use std::io::{self, stdout};
use std::panic::{set_hook, take_hook};

use clap::Parser;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::Terminal;

use app::{AppState, Command};
use events::EventHandler;

/// Kessel Sabacc — TUI frontend
#[derive(Parser, Debug)]
#[command(name = "sabacc-cli", version, about)]
struct Cli {
    /// Quick start: 3 bots, 100 credits, tokens on
    #[arg(long)]
    quick: bool,

    /// Number of bots (1-7)
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=7))]
    bots: Option<u8>,

    /// Buy-in amount (50, 100, 150, or 200 credits)
    #[arg(long)]
    buy_in: Option<u32>,

    /// Player name
    #[arg(long, default_value = "Player")]
    name: String,

    /// Disable ShiftTokens
    #[arg(long)]
    no_tokens: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    init_panic_hook();
    let mut terminal = init_terminal()?;

    let mut state = if cli.quick {
        // --quick: 3 bots, 100cr, tokens on
        AppState::quick_start(cli.name, 3, 100, true)
    } else if cli.bots.is_some() && cli.buy_in.is_some() {
        // Both --bots and --buy-in: skip setup
        AppState::quick_start(
            cli.name,
            cli.bots.unwrap(),
            cli.buy_in.unwrap(),
            !cli.no_tokens,
        )
    } else {
        // Interactive setup — apply any partial CLI args
        let mut s = AppState::new();
        s.setup.player_name = cli.name;
        if let Some(bots) = cli.bots {
            s.setup.num_bots = bots;
        }
        if let Some(buy_in) = cli.buy_in {
            s.setup.buy_in_index = app::SetupState::BUY_IN_OPTIONS
                .iter()
                .position(|&b| b == buy_in)
                .unwrap_or(1);
        }
        if cli.no_tokens {
            s.setup.tokens_enabled = false;
        }
        s
    };

    // If we started directly into a game and it's a bot's turn, run bots
    if state.screen == app::Screen::Playing && state.game.is_some() && !state.is_human_turn() {
        state = app::run_bots(state);
    }

    loop {
        terminal.draw(|frame| ui::render(frame, &state))?;

        let animating = state.is_animating();
        let event = EventHandler::next(animating)?;

        let (new_state, cmd) = app::update(state, event);
        state = new_state;

        match cmd {
            Command::None => {}
            Command::RunBots => {
                state = app::run_bots(state);
            }
            Command::Quit => break,
        }
    }

    restore_terminal()?;
    Ok(())
}

/// Sets up a panic hook that restores the terminal before printing the panic.
fn init_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));
}

/// Enters raw mode + alternate screen and returns the terminal.
fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

/// Restores the terminal to its original state.
fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
