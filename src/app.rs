use crossterm::event::KeyCode;
use crate::ssc::{Observatory, SscClient};
use anyhow::Result;
use ratatui::widgets::ListState;

/// Sort order for satellite list
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOrder {
    NameAZ,
    NameZA,
    StartDate,
}

impl SortOrder {
    pub fn next(&self) -> Self {
        match self {
            SortOrder::NameAZ => SortOrder::NameZA,
            SortOrder::NameZA => SortOrder::StartDate,
            SortOrder::StartDate => SortOrder::NameAZ,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            SortOrder::NameAZ => "Name (A-Z)",
            SortOrder::NameZA => "Name (Z-A)",
            SortOrder::StartDate => "Start Date",
        }
    }
}

/// Main application state
pub struct App {
    /// Current status message
    pub status_message: String,
    /// Whether the app should exit
    pub should_quit: bool,
    /// NASA SSC API client
    pub ssc_client: Option<SscClient>,
    /// List of satellites
    pub satellites: Vec<Observatory>,
    /// List state for scrolling
    pub list_state: ListState,
    /// Current sort order
    pub sort_order: SortOrder,
    /// Whether the detail overlay is showing
    pub show_detail_overlay: bool,
    /// Whether data is loading
    pub loading: bool,
}

impl App {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            status_message: "Loading satellites...".to_string(),
            should_quit: false,
            ssc_client: None,
            satellites: vec![],
            list_state,
            sort_order: SortOrder::NameAZ,
            show_detail_overlay: false,
            loading: true,
        }
    }

    /// Initialize the SSC client and fetch satellites
    pub async fn initialize(&mut self) -> Result<()> {
        let mut client = SscClient::new()?;
        let satellites = client.get_satellites().await?;

        self.ssc_client = Some(client);
        self.satellites = satellites;
        self.sort_satellites();
        self.loading = false;
        self.status_message = format!("Loaded {} satellites - Press 's' to change sort", self.satellites.len());

        Ok(())
    }

    /// Handle keyboard events
    pub fn handle_key_event(&mut self, key: KeyCode) {
        // Handle overlay-specific keys first
        if self.show_detail_overlay {
            match key {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                    self.show_detail_overlay = false;
                    return;
                }
                _ => return, // Ignore other keys when overlay is showing
            }
        }

        // Main key handling
        match key {
            KeyCode::Up | KeyCode::Char('k') => self.previous_satellite(),
            KeyCode::Down | KeyCode::Char('j') => self.next_satellite(),
            KeyCode::Enter => self.show_detail_overlay = true,
            KeyCode::Char('s') => self.cycle_sort_order(),
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('h') => self.show_help(),
            KeyCode::Char('r') => self.refresh_data(),
            _ => {}
        }
    }

    fn show_help(&mut self) {
        self.status_message = "Keys: q=quit, h=help, ↑↓/jk=navigate, Enter=details, s=sort, r=refresh".to_string();
    }

    fn refresh_data(&mut self) {
        self.status_message = "Refreshing data...".to_string();
        // TODO: Implement actual data refresh
    }

    /// Move to next satellite in list
    fn next_satellite(&mut self) {
        if self.satellites.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.satellites.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Move to previous satellite in list
    fn previous_satellite(&mut self) {
        if self.satellites.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.satellites.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Cycle through sort orders
    fn cycle_sort_order(&mut self) {
        self.sort_order = self.sort_order.next();
        self.sort_satellites();
        self.list_state.select(Some(0)); // Reset selection after sort
        self.status_message = format!(
            "Sorted by: {} | {} satellites",
            self.sort_order.as_str(),
            self.satellites.len()
        );
    }

    /// Sort satellites based on current sort order
    fn sort_satellites(&mut self) {
        match self.sort_order {
            SortOrder::NameAZ => {
                self.satellites.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            }
            SortOrder::NameZA => {
                self.satellites.sort_by(|a, b| b.name.to_lowercase().cmp(&a.name.to_lowercase()));
            }
            SortOrder::StartDate => {
                self.satellites.sort_by(|a, b| {
                    a.start_time_parsed()
                        .unwrap_or_else(|_| chrono::Utc::now())
                        .cmp(&b.start_time_parsed().unwrap_or_else(|_| chrono::Utc::now()))
                });
            }
        }
    }

    /// Get the currently selected satellite
    pub fn get_selected_satellite(&self) -> Option<&Observatory> {
        self.list_state.selected()
            .and_then(|i| self.satellites.get(i))
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
