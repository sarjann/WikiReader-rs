use ratatui::{
    layout::Alignment,
    prelude::{Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, List, ListItem},
    Frame,
};

use crate::app::App;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(frame.size());

    let top_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(main_layout[0]);


    let middle_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(100),
        ])
        .split(main_layout[1]);

    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(main_layout[2]);

    // Top
    frame.render_widget(
        Paragraph::new(format!("{}", app.state))
            .block(Block::new().borders(Borders::ALL))
            .alignment(Alignment::Center),
        top_layout[0],
    );

    frame.render_widget(
        Paragraph::new(format!("{}", app.search))
            .block(Block::new().borders(Borders::ALL))
            .alignment(Alignment::Left),
        top_layout[1],
    );

    // // Format with new line for each app result
    // let mut search_results = String::new();
    // for result in &app.search_results {
    //     search_results.push_str(&format!("{}\n", result));
    // }
    //
    // frame.render_widget(
    //     Paragraph::new(format!("{}", search_results))
    //         .block(Block::new().borders(Borders::ALL))
    //         .alignment(Alignment::Left),
    //     middle_layout[0],
    // );
    // let list: Vec<ListItem> = List::new(app.search_results.into());
    let list = List::new(app.search_results.iter().map(|i| ListItem::new(i.as_str())).collect::<Vec<ListItem>>())
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("* ");

    // frame.render_widget(list, middle_layout[0]);
    frame.render_stateful_widget(list, middle_layout[0], &mut app.list_state);

    // Bottom
    frame.render_widget(
        Paragraph::new(format!("{}", app.command))
            .block(Block::new().borders(Borders::ALL))
            .alignment(Alignment::Left),
        bottom_layout[0],
    );
}
