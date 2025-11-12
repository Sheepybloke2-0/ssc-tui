use crossterm::event::KeyCode;

/// Main application state
pub struct App {
    /// Current selected tab
    pub current_tab: usize,
    /// Available tabs
    pub tabs: Vec<&'static str>,
    /// Query results
    pub query_results: Vec<String>,
    /// Current status message
    pub status_message: String,
    /// Whether the app should exit
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_tab: 0,
            tabs: vec!["Query", "Satellites", "Analysis", "Help"],
            query_results: vec![],
            status_message: "Ready - Press 'h' for help".to_string(),
            should_quit: false,
        }
    }

    /// Handle keyboard events
    pub fn handle_key_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('h') => self.show_help(),
            KeyCode::Tab => self.next_tab(),
            KeyCode::BackTab => self.previous_tab(),
            KeyCode::Char('r') => self.refresh_data(),
            KeyCode::Left => self.previous_tab(),
            KeyCode::Right => self.next_tab(),
            _ => {}
        }
    }

    fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % self.tabs.len();
    }

    fn previous_tab(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        } else {
            self.current_tab = self.tabs.len() - 1;
        }
    }

    fn show_help(&mut self) {
        self.status_message = "Keys: q=quit, h=help, Tab/Arrow=navigate, r=refresh".to_string();
    }

    fn refresh_data(&mut self) {
        self.status_message = "Refreshing data...".to_string();
        // TODO: Implement actual data refresh
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
