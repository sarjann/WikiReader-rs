use crate::app::{App, AppResult, State};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    app.before_key_event(&key_event);
    match app.state {
        State::Search => match key_event.code {
            KeyCode::Esc => {
                app.set_state(State::Normal);
            }

            KeyCode::Char(c) => {
                app.search.push(c);
                app.search();
            }
            KeyCode::Backspace => {
                app.search.pop();
            }

            KeyCode::Enter => {
                app.list_state.select(Some(0));
                app.set_state(State::Browse);
            }
            _ => {}
        },
        State::Command => match key_event.code {
            KeyCode::Esc => {
                app.set_state(State::Normal);
                app.command.clear();
            }

            KeyCode::Char(c) => {
                app.command.push(c);
            }
            KeyCode::Backspace => {
                app.command.pop();
            }

            KeyCode::Enter => {
                app.set_state(State::Normal);
                app.execute_command();
            }
            _ => {}
        },
        State::Normal => match key_event.code {
            KeyCode::Esc => {
                app.set_state(State::Normal);
            }

            // Exit application on `Ctrl-C`
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.quit();
                }
            }

            // Command mode
            KeyCode::Char(':') => {
                app.set_state(State::Command);
                app.command.push(':')
            }

            // Search Mode
            KeyCode::Char('/') => {
                app.set_state(State::Search);
                app.search.clear();
            }

            KeyCode::Left | KeyCode::Char('h') => {
                app.left();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                app.right();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.down(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.up(1);
            }

            // Help
            KeyCode::Char('?') => {
                app.set_state(State::Help);
            }

            _ => {}
        },
        State::Browse => match key_event.code {
            KeyCode::Esc => {
                app.set_state(State::Normal);
            }
            // Command mode
            KeyCode::Char(':') => {
                app.set_state(State::Command);
                app.command.push(':')
            }
            // Search Mode
            KeyCode::Char('/') => {
                app.set_state(State::Search);
                app.search.clear();
            }
            // Navigation
            KeyCode::Down | KeyCode::Char('j') => {
                app.down(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.up(1);
            }
            // gg (go to top)
            KeyCode::Char('g') => match app.last_key {
                Some(KeyCode::Char('g')) => {
                    app.list_state.select(Some(0));
                }
                _ => {}
            },
            // G (go to bottom)
            KeyCode::Char('G') => {
                app.list_state.select(Some(app.search_results.len() - 1));
            }
            KeyCode::Enter => {
                app.get_page();
            }
            _ => {}
        },
        State::Read => match key_event.code {
            KeyCode::Esc => {
                app.set_state(State::Normal);
            }
            // Command mode
            KeyCode::Char(':') => {
                app.set_state(State::Command);
                app.command.push(':')
            }
            // Search Mode
            KeyCode::Char('/') => {
                app.set_state(State::Search);
                app.search.clear();
            }
            // Navigation
            KeyCode::Down | KeyCode::Char('j') => {
                app.down(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.up(1);
            }

            KeyCode::Char('u') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.up(10);
                }
            }

            KeyCode::Char('d') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.down(10);
                }
            }

            // gg (go to top)
            KeyCode::Char('g') => match app.last_key {
                Some(KeyCode::Char('g')) => {
                    app.scroll = 0;
                }
                _ => {}
            },
            // G (go to bottom)
            KeyCode::Char('G') => {
                // TODO
            }
            _ => {}
        },
        State::Help => match key_event.code {
            KeyCode::Esc => {
                app.set_state(State::Normal);
            }
            _ => {}
        },
        // _ => match key_event.code {
        //     KeyCode::Esc => {
        //         app.set_state(State::Normal);
        //     }
        //     _ => {}
        // },
    }
    app.after_key_event(&key_event);
    Ok(())
}
