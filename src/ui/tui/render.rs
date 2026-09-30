use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::state::{PaneId, UiState, visible_window};
use crate::ui::Phase;

/// Header (1) + footer (1) chrome rows used when sizing pane viewports.
pub const CHROME_ROWS: u16 = 1 + 1;

pub fn draw(frame: &mut Frame, state: &UiState, clipboard_status: Option<&str>) {
    let area = frame.area();
    if let Some(ref err) = state.error {
        draw_error_modal(frame, area, err);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(area);

    frame.render_widget(Paragraph::new(header_line(state)), chunks[0]);
    if state.maximized {
        render_pane(frame, chunks[1], state, state.focus);
    } else {
        draw_grid(frame, chunks[1], state);
    }
    frame.render_widget(
        Paragraph::new(Line::from(state.keyboard_hints(clipboard_status))),
        chunks[2],
    );
}

fn phase_label(phase: Phase) -> &'static str {
    match phase {
        Phase::Install => "install",
        Phase::Dedupe => "dedupe",
        Phase::Audit => "audit",
        Phase::Fix => "fix",
        Phase::Hygiene => "hygiene",
        Phase::Report => "report",
        Phase::Done => "done",
    }
}

fn header_line(state: &UiState) -> Line<'static> {
    let verb = match state.verb {
        crate::cmd::Verb::Auto => "auto",
        crate::cmd::Verb::Check => "check",
        _ => "gnarl",
    };
    let secs = state.elapsed.as_secs();
    let focus = match state.focus {
        PaneId::Activity => "activity",
        PaneId::Fixed => "fixed",
        PaneId::State => "insights",
        PaneId::Next => "tasks",
    };
    Line::from(vec![
        Span::styled(
            format!(" gnarl · {verb} "),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            "· {} · {:02}:{:02} · focus:{focus}{} ",
            phase_label(state.phase),
            secs / 60,
            secs % 60,
            if state.maximized { " · expanded" } else { "" }
        )),
    ])
}

fn draw_grid(frame: &mut Frame, area: Rect, state: &UiState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    render_pane(frame, top[0], state, PaneId::Activity);
    render_pane(frame, top[1], state, PaneId::Fixed);
    render_pane(frame, bottom[0], state, PaneId::State);
    render_pane(frame, bottom[1], state, PaneId::Next);
}

fn render_pane(frame: &mut Frame, area: Rect, state: &UiState, pane: PaneId) {
    let lines = state.pane_lines(pane);
    let inner_height = area.height.saturating_sub(2) as usize;
    let scroll = state.scroll_from_bottom[pane.index()];
    let window = visible_window(&lines, inner_height, scroll);
    let content: Vec<Line> = window.iter().map(|l| Line::from(l.clone())).collect();

    let focused = state.focus == pane;
    let base_title = pane.title(state);
    let title = if focused {
        format!("▶{}◀", base_title.trim())
    } else {
        base_title
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if focused {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        });

    frame.render_widget(
        Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_error_modal(frame: &mut Frame, area: Rect, message: &str) {
    let popup = centered_rect(70, 40, area);
    frame.render_widget(Clear, popup);
    let text = vec![
        Line::from(Span::styled(
            " error ",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(message.to_owned()),
        Line::from(""),
        Line::from("[q] quit"),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" ERROR "))
            .wrap(Wrap { trim: false }),
        popup,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
