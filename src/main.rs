use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

use ssc_tui::app::App;
use ssc_tui::ui;

fn main() -> Result<()> {
    // Check if we have a terminal
    if !crossterm::tty::IsTty::is_tty(&io::stdout()) {
        eprintln!("Error: No TTY detected. This application requires a terminal.");
        eprintln!("If using WSL, try running from:");
        eprintln!("  - Windows Terminal");
        eprintln!("  - The Docker container: docker compose run --rm dev");
        eprintln!("  - WSL terminal with proper TTY support");
        std::process::exit(1);
    }

    // Setup terminal
    enable_raw_mode().map_err(|e| {
        anyhow::anyhow!("Failed to enable raw mode: {}. Make sure you're running in a proper terminal.", e)
    })?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key_event(key.code);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
