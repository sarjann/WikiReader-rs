use ratatui::{
    layout::Alignment,
    prelude::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, Padding, Paragraph},
    Frame,
};

use crate::app::{App, State};

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
        .constraints([Constraint::Percentage(100)])
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

    match app.state {
        State::Read => {
            let revision = &app.page.as_ref().unwrap().revision;
            if revision.is_none() {
                return;
            }

            let text = &revision.as_ref().unwrap().text;
            if text.is_none() {
                return;
            }

            let text_str_opt = &text.as_ref().unwrap().value;
            let text_str;
            if text_str_opt.is_none() {
                text_str = String::from("");
            } else {
                text_str = text_str_opt.as_ref().unwrap().to_string();
            }

            let detail = Paragraph::new(text_str);

            frame.render_widget(
                detail
                    .block(Block::default().borders(Borders::ALL))
                    .alignment(Alignment::Left)
                    .scroll((app.scroll, 0)),
                middle_layout[0],
            )
        }
        _ => {
            let list = List::new(
                app.search_results
                    .iter()
                    .map(|result| ListItem::new(result.title.as_str()))
                    .collect::<Vec<ListItem>>(),
            )
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("* ");

            frame.render_stateful_widget(list, middle_layout[0], &mut app.list_state);
        }
    }

    // Bottom
    frame.render_widget(
        Paragraph::new(format!("{}", app.command))
            .block(Block::new().borders(Borders::ALL))
            .alignment(Alignment::Left),
        bottom_layout[0],
    );
}
