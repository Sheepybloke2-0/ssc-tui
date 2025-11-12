use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

/// Main render function
pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content (satellite list)
            Constraint::Length(3),  // Status bar
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_satellites(f, app, chunks[1]);
    render_status(f, app, chunks[2]);

    // Render overlay on top if showing
    if app.show_detail_overlay {
        render_detail_overlay(f, app);
    }
}

/// Render the header
fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let title_text = if app.loading {
        "NASA SSC Satellite Browser - Loading..."
    } else {
        "NASA SSC Satellite Browser"
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title_text)
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan));

    let text = vec![Line::from(Span::styled(
        format!("Press 'h' for help | {} satellites loaded", app.satellites.len()),
        Style::default().fg(Color::Gray),
    ))];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

/// Render the satellites list
fn render_satellites(f: &mut Frame, app: &mut App, area: Rect) {
    if app.loading {
        let block = Block::default()
            .title("Satellites")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let loading_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Loading satellites from NASA SSC API...",
                Style::default().fg(Color::Yellow),
            )),
        ];

        let paragraph = Paragraph::new(loading_text)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
        return;
    }

    if app.satellites.is_empty() {
        let block = Block::default()
            .title("Satellites")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No satellites loaded",
                Style::default().fg(Color::Red),
            )),
            Line::from(""),
            Line::from("Press 'r' to refresh"),
        ];

        let paragraph = Paragraph::new(empty_text)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
        return;
    }

    let title = format!(
        "Satellites ({}) - Sort: {}",
        app.satellites.len(),
        app.sort_order.as_str()
    );

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    // Create list items for satellites
    let items: Vec<ListItem> = app
        .satellites
        .iter()
        .map(|sat| {
            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    &sat.name,
                    Style::default().fg(Color::Cyan),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

/// Render satellite detail overlay
fn render_detail_overlay(f: &mut Frame, app: &App) {
    let satellite = match app.get_selected_satellite() {
        Some(sat) => sat,
        None => return,
    };

    // Create centered overlay
    let area = centered_rect(80, 60, f.area());

    // Clear the area
    f.render_widget(Clear, area);

    // Create the overlay block
    let block = Block::default()
        .title(format!(" Satellite Details: {} ", satellite.name))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .border_style(Style::default().fg(Color::Yellow));

    // Format the satellite details
    let start_date = satellite
        .start_time_parsed()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let end_date = satellite
        .end_time_parsed()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let status = if let Ok(end_time) = satellite.end_time_parsed() {
        if end_time > chrono::Utc::now() {
            Span::styled("Active", Style::default().fg(Color::Green))
        } else {
            Span::styled("Historical", Style::default().fg(Color::Gray))
        }
    } else {
        Span::styled("Unknown", Style::default().fg(Color::Gray))
    };

    let mut content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(&satellite.id),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(&satellite.name),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            status,
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Resolution: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(format!("{} ({}s)", satellite.resolution_str(), satellite.resolution)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Mission Start: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(&start_date),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Mission End: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(&end_date),
        ]),
    ];

    if let Some(resource_id) = &satellite.resource_id {
        content.push(Line::from(""));
        content.push(Line::from(vec![
            Span::styled("SPASE ID: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(resource_id),
        ]));
    }

    if !satellite.group_id.is_empty() {
        content.push(Line::from(""));
        content.push(Line::from(vec![
            Span::styled("Groups: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(satellite.group_id.join(", ")),
        ]));
    }

    content.push(Line::from(""));
    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "Press Enter/Esc/q to close",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
    )));

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centered_rect_80_60() {
        let screen = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(80, 60, screen);

        // Width: 80% of 100 = 80, centered means starting at 10
        assert_eq!(centered.x, 10);
        assert_eq!(centered.width, 80);

        // Height: 60% of 100 = 60, centered means starting at 20
        assert_eq!(centered.y, 20);
        assert_eq!(centered.height, 60);
    }

    #[test]
    fn test_centered_rect_50_50() {
        let screen = Rect::new(0, 0, 200, 100);
        let centered = centered_rect(50, 50, screen);

        // Width: 50% of 200 = 100, centered means starting at 50
        assert_eq!(centered.x, 50);
        assert_eq!(centered.width, 100);

        // Height: 50% of 100 = 50, centered means starting at 25
        assert_eq!(centered.y, 25);
        assert_eq!(centered.height, 50);
    }

    #[test]
    fn test_centered_rect_100_100() {
        let screen = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(100, 100, screen);

        // 100% should fill entire screen
        assert_eq!(centered.x, 0);
        assert_eq!(centered.y, 0);
        assert_eq!(centered.width, 100);
        assert_eq!(centered.height, 100);
    }

    #[test]
    fn test_centered_rect_small_screen() {
        let screen = Rect::new(0, 0, 50, 30);
        let centered = centered_rect(80, 60, screen);

        // Width: 80% of 50 = 40, centered means starting at 5
        assert_eq!(centered.x, 5);
        assert_eq!(centered.width, 40);

        // Height: 60% of 30 = 18, centered means starting at 6
        assert_eq!(centered.y, 6);
        assert_eq!(centered.height, 18);
    }

    #[test]
    fn test_centered_rect_offset_origin() {
        // Test with a rect that doesn't start at (0, 0)
        let screen = Rect::new(10, 20, 100, 100);
        let centered = centered_rect(50, 50, screen);

        // The centered rect should be within the bounds of screen
        assert!(centered.x >= screen.x);
        assert!(centered.y >= screen.y);
        assert!(centered.x + centered.width <= screen.x + screen.width);
        assert!(centered.y + centered.height <= screen.y + screen.height);
    }

    #[test]
    fn test_centered_rect_proportions() {
        let screen = Rect::new(0, 0, 120, 80);
        let centered = centered_rect(60, 40, screen);

        // Verify the centered rect has the correct proportions
        assert_eq!(centered.width, 72); // 60% of 120
        assert_eq!(centered.height, 32); // 40% of 80

        // Verify it's centered
        assert_eq!(centered.x, 24); // (120 - 72) / 2
        assert_eq!(centered.y, 24); // (80 - 32) / 2
    }
}
