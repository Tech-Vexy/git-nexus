use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

use crate::RepoStatus;

pub fn run_tui(repos: Vec<RepoStatus>) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut selected = 0;

    loop {
        terminal.draw(|f| ui(f, &repos, selected))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected < repos.len().saturating_sub(1) {
                        selected += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                KeyCode::Home => selected = 0,
                KeyCode::End => selected = repos.len().saturating_sub(1),
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, repos: &[RepoStatus], selected: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(vec![Line::from(vec![
        Span::styled("git-nexus", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(
            format!("{} repositories", repos.len()),
            Style::default().fg(Color::Green),
        ),
    ])])
    .block(Block::default().borders(Borders::ALL).title("Git Nexus"));
    f.render_widget(header, chunks[0]);

    // Repository list
    let items: Vec<ListItem> = repos
        .iter()
        .enumerate()
        .map(|(i, repo)| {
            let status_color = if repo.is_clean {
                Color::Green
            } else {
                Color::Red
            };

            let status_text = if repo.is_clean { "CLEAN" } else { "DIRTY" };

            let mut spans = vec![
                Span::styled("📁 ", Style::default()),
                Span::styled(
                    repo.path.display().to_string(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
            ];

            if let Some(ref branch) = repo.branch {
                spans.push(Span::styled(
                    format!("({})", branch),
                    Style::default().fg(Color::Blue),
                ));
                spans.push(Span::raw(" "));
            }

            spans.push(Span::styled(
                format!("[{}]", status_text),
                Style::default().fg(status_color),
            ));

            if repo.ahead > 0 {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    format!("↑{}", repo.ahead),
                    Style::default().fg(Color::Yellow),
                ));
            }

            if repo.behind > 0 {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    format!("↓{}", repo.behind),
                    Style::default().fg(Color::Red),
                ));
            }

            let style = if i == selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(spans)).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Repositories"));
    f.render_widget(list, chunks[1]);

    // Footer with help
    let help = Paragraph::new("↑/k: Up | ↓/j: Down | Home/End: Jump | q/Esc: Quit")
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(help, chunks[2]);
}
