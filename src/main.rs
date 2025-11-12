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
use std::sync::Arc;
use tokio::sync::Mutex;

use ssc_tui::app::App;
use ssc_tui::ui;

#[tokio::main]
async fn main() -> Result<()> {
    // Check if we have a terminal
    if !crossterm::tty::IsTty::is_tty(&io::stdout()) {
        eprintln!("Error: No TTY detected. This application requires a terminal.");
        eprintln!("If using WSL, try running from:");
        eprintln!("  - Windows Terminal");
        eprintln!("  - The Docker container: docker compose run -it --rm dev cargo run");
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

    // Create app
    let app = Arc::new(Mutex::new(App::new()));
    let app_clone = Arc::clone(&app);

    // Spawn initialization task
    tokio::spawn(async move {
        let mut app_locked = app_clone.lock().await;
        if let Err(e) = app_locked.initialize().await {
            app_locked.status_message = format!("Error loading satellites: {}", e);
            app_locked.loading = false;
        }
    });

    // Run app
    let res = run_app(&mut terminal, app).await;

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

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: Arc<Mutex<App>>,
) -> Result<()> {
    loop {
        // Draw UI
        {
            let mut app_locked = app.lock().await;
            terminal.draw(|f| ui::render(f, &mut *app_locked))?;
        }

        // Poll for events with timeout
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let mut app_locked = app.lock().await;
                    app_locked.handle_key_event(key.code);
                }
            }
        }

        // Check if should quit
        {
            let app_locked = app.lock().await;
            if app_locked.should_quit {
                break;
            }
        }
    }
    Ok(())
}
