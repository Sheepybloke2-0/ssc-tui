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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_order_next() {
        assert_eq!(SortOrder::NameAZ.next(), SortOrder::NameZA);
        assert_eq!(SortOrder::NameZA.next(), SortOrder::StartDate);
        assert_eq!(SortOrder::StartDate.next(), SortOrder::NameAZ);
    }

    #[test]
    fn test_sort_order_as_str() {
        assert_eq!(SortOrder::NameAZ.as_str(), "Name (A-Z)");
        assert_eq!(SortOrder::NameZA.as_str(), "Name (Z-A)");
        assert_eq!(SortOrder::StartDate.as_str(), "Start Date");
    }

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert_eq!(app.should_quit, false);
        assert_eq!(app.loading, true);
        assert_eq!(app.show_detail_overlay, false);
        assert_eq!(app.sort_order, SortOrder::NameAZ);
        assert_eq!(app.satellites.len(), 0);
        assert_eq!(app.list_state.selected(), Some(0));
    }

    #[test]
    fn test_app_default() {
        let app = App::default();
        assert_eq!(app.should_quit, false);
        assert_eq!(app.loading, true);
    }

    #[test]
    fn test_navigation_empty_list() {
        let mut app = App::new();

        // Navigation should not panic on empty list
        app.next_satellite();
        assert_eq!(app.list_state.selected(), Some(0));

        app.previous_satellite();
        assert_eq!(app.list_state.selected(), Some(0));
    }

    #[test]
    fn test_navigation_single_item() {
        let mut app = App::new();
        app.satellites = vec![create_test_satellite("sat1", "Satellite 1")];

        // Next wraps to beginning
        app.next_satellite();
        assert_eq!(app.list_state.selected(), Some(0));

        // Previous wraps to end
        app.previous_satellite();
        assert_eq!(app.list_state.selected(), Some(0));
    }

    #[test]
    fn test_navigation_multiple_items() {
        let mut app = App::new();
        app.satellites = vec![
            create_test_satellite("sat1", "Satellite 1"),
            create_test_satellite("sat2", "Satellite 2"),
            create_test_satellite("sat3", "Satellite 3"),
        ];
        app.list_state.select(Some(0));

        // Move forward
        app.next_satellite();
        assert_eq!(app.list_state.selected(), Some(1));

        app.next_satellite();
        assert_eq!(app.list_state.selected(), Some(2));

        // Wrap to beginning
        app.next_satellite();
        assert_eq!(app.list_state.selected(), Some(0));

        // Move backward
        app.previous_satellite();
        assert_eq!(app.list_state.selected(), Some(2));

        app.previous_satellite();
        assert_eq!(app.list_state.selected(), Some(1));
    }

    #[test]
    fn test_sort_satellites_name_az() {
        let mut app = App::new();
        app.satellites = vec![
            create_test_satellite("sat3", "Zebra"),
            create_test_satellite("sat1", "Alpha"),
            create_test_satellite("sat2", "Beta"),
        ];
        app.sort_order = SortOrder::NameAZ;

        app.sort_satellites();

        assert_eq!(app.satellites[0].name, "Alpha");
        assert_eq!(app.satellites[1].name, "Beta");
        assert_eq!(app.satellites[2].name, "Zebra");
    }

    #[test]
    fn test_sort_satellites_name_za() {
        let mut app = App::new();
        app.satellites = vec![
            create_test_satellite("sat1", "Alpha"),
            create_test_satellite("sat3", "Zebra"),
            create_test_satellite("sat2", "Beta"),
        ];
        app.sort_order = SortOrder::NameZA;

        app.sort_satellites();

        assert_eq!(app.satellites[0].name, "Zebra");
        assert_eq!(app.satellites[1].name, "Beta");
        assert_eq!(app.satellites[2].name, "Alpha");
    }

    #[test]
    fn test_sort_satellites_case_insensitive() {
        let mut app = App::new();
        app.satellites = vec![
            create_test_satellite("sat1", "alpha"),
            create_test_satellite("sat2", "BETA"),
            create_test_satellite("sat3", "Zebra"),
        ];
        app.sort_order = SortOrder::NameAZ;

        app.sort_satellites();

        assert_eq!(app.satellites[0].name, "alpha");
        assert_eq!(app.satellites[1].name, "BETA");
        assert_eq!(app.satellites[2].name, "Zebra");
    }

    #[test]
    fn test_sort_satellites_by_start_date() {
        let mut app = App::new();
        app.satellites = vec![
            create_test_satellite_with_date("sat2", "Sat2", "2020-01-01T00:00:00Z"),
            create_test_satellite_with_date("sat1", "Sat1", "2010-01-01T00:00:00Z"),
            create_test_satellite_with_date("sat3", "Sat3", "2025-01-01T00:00:00Z"),
        ];
        app.sort_order = SortOrder::StartDate;

        app.sort_satellites();

        assert_eq!(app.satellites[0].name, "Sat1");
        assert_eq!(app.satellites[1].name, "Sat2");
        assert_eq!(app.satellites[2].name, "Sat3");
    }

    #[test]
    fn test_cycle_sort_order() {
        let mut app = App::new();
        app.satellites = vec![
            create_test_satellite("sat1", "Beta"),
            create_test_satellite("sat2", "Alpha"),
        ];

        assert_eq!(app.sort_order, SortOrder::NameAZ);

        app.cycle_sort_order();
        assert_eq!(app.sort_order, SortOrder::NameZA);
        assert_eq!(app.list_state.selected(), Some(0)); // Selection reset
        assert_eq!(app.satellites[0].name, "Beta"); // Reverse sorted

        app.cycle_sort_order();
        assert_eq!(app.sort_order, SortOrder::StartDate);

        app.cycle_sort_order();
        assert_eq!(app.sort_order, SortOrder::NameAZ);
    }

    #[test]
    fn test_get_selected_satellite() {
        let mut app = App::new();
        app.satellites = vec![
            create_test_satellite("sat1", "Satellite 1"),
            create_test_satellite("sat2", "Satellite 2"),
            create_test_satellite("sat3", "Satellite 3"),
        ];

        app.list_state.select(Some(0));
        assert_eq!(app.get_selected_satellite().unwrap().name, "Satellite 1");

        app.list_state.select(Some(1));
        assert_eq!(app.get_selected_satellite().unwrap().name, "Satellite 2");

        app.list_state.select(Some(2));
        assert_eq!(app.get_selected_satellite().unwrap().name, "Satellite 3");
    }

    #[test]
    fn test_get_selected_satellite_none() {
        let mut app = App::new();
        app.satellites = vec![create_test_satellite("sat1", "Satellite 1")];

        app.list_state.select(None);
        assert!(app.get_selected_satellite().is_none());
    }

    #[test]
    fn test_get_selected_satellite_out_of_bounds() {
        let mut app = App::new();
        app.satellites = vec![create_test_satellite("sat1", "Satellite 1")];

        app.list_state.select(Some(10));
        assert!(app.get_selected_satellite().is_none());
    }

    #[test]
    fn test_handle_key_event_quit() {
        let mut app = App::new();
        assert_eq!(app.should_quit, false);

        app.handle_key_event(KeyCode::Char('q'));
        assert_eq!(app.should_quit, true);
    }

    #[test]
    fn test_handle_key_event_sort() {
        let mut app = App::new();
        app.satellites = vec![
            create_test_satellite("sat1", "Beta"),
            create_test_satellite("sat2", "Alpha"),
        ];

        assert_eq!(app.sort_order, SortOrder::NameAZ);

        app.handle_key_event(KeyCode::Char('s'));
        assert_eq!(app.sort_order, SortOrder::NameZA);
    }

    #[test]
    fn test_handle_key_event_navigation() {
        let mut app = App::new();
        app.satellites = vec![
            create_test_satellite("sat1", "Sat1"),
            create_test_satellite("sat2", "Sat2"),
        ];
        app.list_state.select(Some(0));

        app.handle_key_event(KeyCode::Down);
        assert_eq!(app.list_state.selected(), Some(1));

        app.handle_key_event(KeyCode::Up);
        assert_eq!(app.list_state.selected(), Some(0));

        app.handle_key_event(KeyCode::Char('j'));
        assert_eq!(app.list_state.selected(), Some(1));

        app.handle_key_event(KeyCode::Char('k'));
        assert_eq!(app.list_state.selected(), Some(0));
    }

    #[test]
    fn test_handle_key_event_overlay() {
        let mut app = App::new();
        assert_eq!(app.show_detail_overlay, false);

        app.handle_key_event(KeyCode::Enter);
        assert_eq!(app.show_detail_overlay, true);

        app.handle_key_event(KeyCode::Esc);
        assert_eq!(app.show_detail_overlay, false);

        app.handle_key_event(KeyCode::Enter);
        assert_eq!(app.show_detail_overlay, true);

        app.handle_key_event(KeyCode::Char('q'));
        assert_eq!(app.show_detail_overlay, false);
    }

    #[test]
    fn test_overlay_blocks_other_keys() {
        let mut app = App::new();
        app.satellites = vec![
            create_test_satellite("sat1", "Sat1"),
            create_test_satellite("sat2", "Sat2"),
        ];
        app.list_state.select(Some(0));
        app.show_detail_overlay = true;

        let initial_selection = app.list_state.selected();

        // These keys should be ignored when overlay is showing
        app.handle_key_event(KeyCode::Down);
        assert_eq!(app.list_state.selected(), initial_selection);

        app.handle_key_event(KeyCode::Char('s'));
        assert_eq!(app.sort_order, SortOrder::NameAZ); // Sort didn't change
    }

    // Helper functions for creating test data
    fn create_test_satellite(id: &str, name: &str) -> Observatory {
        Observatory {
            id: id.to_string(),
            name: name.to_string(),
            resolution: 60,
            start_time: "2020-01-01T00:00:00Z".to_string(),
            end_time: "2030-01-01T00:00:00Z".to_string(),
            resource_id: None,
            group_id: vec![],
        }
    }

    fn create_test_satellite_with_date(id: &str, name: &str, start_time: &str) -> Observatory {
        Observatory {
            id: id.to_string(),
            name: name.to_string(),
            resolution: 60,
            start_time: start_time.to_string(),
            end_time: "2030-01-01T00:00:00Z".to_string(),
            resource_id: None,
            group_id: vec![],
        }
    }
}
