use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};

use crate::app::App;

/// Main render function
pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header with tabs
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Status bar
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_content(f, app, chunks[1]);
    render_status(f, app, chunks[2]);
}

/// Render the header with tabs
fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = app
        .tabs
        .iter()
        .map(|t| Line::from(*t))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("NASA SSC TUI"))
        .select(app.current_tab)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

/// Render the main content area based on selected tab
fn render_content(f: &mut Frame, app: &App, area: Rect) {
    match app.current_tab {
        0 => render_query_tab(f, app, area),
        1 => render_satellites_tab(f, app, area),
        2 => render_analysis_tab(f, app, area),
        3 => render_help_tab(f, app, area),
        _ => {}
    }
}

/// Render the query tab
fn render_query_tab(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Query input panel
    let query_block = Block::default()
        .title("Query Parameters")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let query_text = vec![
        Line::from(""),
        Line::from("Satellite: <select>"),
        Line::from("Start Time: <input>"),
        Line::from("End Time: <input>"),
        Line::from("Coordinate System: <select>"),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Execute Query",
            Style::default().fg(Color::Green),
        )),
    ];

    let query_paragraph = Paragraph::new(query_text).block(query_block);
    f.render_widget(query_paragraph, chunks[0]);

    // Results panel
    let results_block = Block::default()
        .title("Results")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let items: Vec<ListItem> = app
        .query_results
        .iter()
        .map(|r| ListItem::new(r.as_str()))
        .collect();

    let results_list = List::new(items)
        .block(results_block)
        .style(Style::default().fg(Color::White));

    f.render_widget(results_list, chunks[1]);
}

/// Render the satellites tab
fn render_satellites_tab(f: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title("Available Satellites")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "ISS - International Space Station",
            Style::default().fg(Color::Cyan),
        )),
        Line::from("  Status: Active | Resolution: 1min"),
        Line::from(""),
        Line::from(Span::styled(
            "ACE - Advanced Composition Explorer",
            Style::default().fg(Color::Cyan),
        )),
        Line::from("  Status: Active | Resolution: 5min"),
        Line::from(""),
        Line::from(Span::styled(
            "Geotail",
            Style::default().fg(Color::Cyan),
        )),
        Line::from("  Status: Active | Resolution: 1min"),
        Line::from(""),
        Line::from(Span::styled(
            "TODO: Load from NASA SSC API",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
        )),
    ];

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}

/// Render the analysis tab
fn render_analysis_tab(f: &mut Frame, _app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Analysis options
    let options_block = Block::default()
        .title("Analysis Tools")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let options_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "1. Trajectory Visualization",
            Style::default().fg(Color::Green),
        )),
        Line::from("   Plot satellite orbits in 3D space"),
        Line::from(""),
        Line::from(Span::styled(
            "2. Conjunction Analysis",
            Style::default().fg(Color::Green),
        )),
        Line::from("   Find close approaches between satellites"),
        Line::from(""),
        Line::from(Span::styled(
            "3. Ground Track",
            Style::default().fg(Color::Green),
        )),
        Line::from("   Display satellite ground coverage"),
        Line::from(""),
        Line::from(Span::styled(
            "TODO: Implement analysis functions",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
        )),
    ];

    let options_paragraph = Paragraph::new(options_text).block(options_block);
    f.render_widget(options_paragraph, chunks[0]);

    // Analysis results
    let results_block = Block::default()
        .title("Analysis Results")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let results_text = vec![
        Line::from(""),
        Line::from("Select an analysis tool to begin..."),
    ];

    let results_paragraph = Paragraph::new(results_text).block(results_block);
    f.render_widget(results_paragraph, chunks[1]);
}

/// Render the help tab
fn render_help_tab(f: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title("Help & Keyboard Shortcuts")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from("  Tab / Shift+Tab  - Switch between tabs"),
        Line::from("  ← / →           - Switch between tabs"),
        Line::from("  h               - Show this help"),
        Line::from("  q               - Quit application"),
        Line::from(""),
        Line::from(Span::styled(
            "Query Tab",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from("  Enter           - Execute query"),
        Line::from("  r               - Refresh data"),
        Line::from(""),
        Line::from(Span::styled(
            "About NASA SSC",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from("  The Satellite Situation Center (SSC) provides"),
        Line::from("  spacecraft orbit data and visualizations."),
        Line::from(""),
        Line::from(Span::styled(
            "API Endpoint:",
            Style::default().fg(Color::Cyan),
        )),
        Line::from("  https://sscweb.gsfc.nasa.gov/WS/sscr/2"),
    ];

    let paragraph = Paragraph::new(help_text).block(block);
    f.render_widget(paragraph, area);
}

/// Render the status bar
fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let status_text = vec![Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Green)),
        Span::raw(&app.status_message),
    ])];

    let paragraph = Paragraph::new(status_text).block(block);
    f.render_widget(paragraph, area);
}
