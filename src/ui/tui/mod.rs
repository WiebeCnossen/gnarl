//! Interactive ratatui surfaces.

mod render;
mod state;

pub use state::{PaneId, UiState};

use std::io::{self, Stdout};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use super::{Reporter, SharedReporter, UiEvent};
use crate::Error;
use crate::RunStatus;
use crate::cmd::Verb;
use crate::gnarl::Gnarl;

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    fn enter() -> Result<Self, Error> {
        enable_raw_mode().map_err(|e| Error::String(e.to_string()))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(|e| Error::String(e.to_string()))?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).map_err(|e| Error::String(e.to_string()))?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

struct ChannelReporter {
    tx: Sender<UiEvent>,
}

impl Reporter for ChannelReporter {
    fn emit(&self, event: UiEvent) {
        let _ = self.tx.send(event);
    }
}

/// Run `auto` or `check` on a worker thread with a live/compact TUI on the main thread.
pub fn run_interactive(
    verb: Verb,
    options: crate::cmd::Options,
    version: &str,
    refresh_first: bool,
) -> Result<RunStatus, Error> {
    let (tx, rx) = mpsc::channel::<UiEvent>();
    let reporter: SharedReporter = Arc::new(ChannelReporter { tx });

    let worker = thread::spawn(move || -> Result<RunStatus, Error> {
        let run_reporter = reporter.clone();
        let result = (|| {
            let mut gnarl = Gnarl::with_reporter(options, reporter.clone())?;
            match verb {
                Verb::Auto => gnarl.auto(refresh_first),
                Verb::Check => gnarl.check(),
                _ => Ok(RunStatus::ok()),
            }
        })();
        if let Err(ref err) = result {
            run_reporter.emit(UiEvent::Error {
                message: err.to_string(),
            });
        }
        result
    });

    let ui_result = event_loop(verb, version, rx);
    let worker_result = worker
        .join()
        .unwrap_or_else(|_| Err("worker thread panicked".into()));

    finish_interactive(ui_result, worker_result)
}

pub(crate) fn finish_interactive(
    ui: Result<(), Error>,
    worker: Result<RunStatus, Error>,
) -> Result<RunStatus, Error> {
    match (ui, worker) {
        (Ok(()), Ok(status)) => Ok(status),
        (Err(e), _) | (_, Err(e)) => Err(e),
    }
}

fn event_loop(verb: Verb, version: &str, rx: Receiver<UiEvent>) -> Result<(), Error> {
    let mut guard = TerminalGuard::enter()?;
    let mut state = UiState::new(verb);
    state
        .activity
        .push(format!("[INFO] gnarl {version}"));
    let started = Instant::now();
    let mut clipboard_status: Option<String> = None;

    loop {
        let running = !state.done && state.error.is_none();
        while let Ok(event) = rx.try_recv() {
            state.apply(event);
        }
        if running {
            // Keep ticking while work runs; take a final stamp the moment we finish.
            state.elapsed = started.elapsed();
        }
        let size = guard
            .terminal
            .size()
            .map_err(|e| Error::String(e.to_string()))?;
        // Header + footer chrome; grid uses half-height panes unless maximized.
        let content_h = size.height.saturating_sub(render::CHROME_ROWS);
        let pane_viewport_height = if state.maximized {
            content_h.saturating_sub(2).max(1) as usize
        } else {
            ((content_h) / 2).saturating_sub(2).max(1) as usize
        };

        guard
            .terminal
            .draw(|frame| render::draw(frame, &state, clipboard_status.as_deref()))
            .map_err(|e| Error::String(e.to_string()))?;

        if event::poll(Duration::from_millis(50)).map_err(|e| Error::String(e.to_string()))?
            && let Event::Key(key) = event::read().map_err(|e| Error::String(e.to_string()))?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Esc => {
                    if state.maximized {
                        state.maximized = false;
                    } else if state.error.is_some() || state.done {
                        break;
                    }
                }
                KeyCode::Char('q') => {
                    if state.error.is_some() || state.done {
                        break;
                    }
                }
                KeyCode::Enter => {
                    state.maximized = true;
                }
                KeyCode::BackTab => state.cycle_focus_backward(),
                KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    state.cycle_focus_backward();
                }
                KeyCode::Tab => state.cycle_focus(),
                KeyCode::Up | KeyCode::Char('k') => {
                    state.scroll_focused(1, pane_viewport_height);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.scroll_focused(-1, pane_viewport_height);
                }
                KeyCode::PageUp => {
                    state.scroll_focused(pane_viewport_height as isize, pane_viewport_height);
                }
                KeyCode::PageDown => {
                    state.scroll_focused(-(pane_viewport_height as isize), pane_viewport_height);
                }
                KeyCode::Home => {
                    let len = state.pane_lines(state.focus).len();
                    let max = len.saturating_sub(pane_viewport_height);
                    state.scroll_from_bottom[state.focus.index()] = max;
                }
                KeyCode::End => {
                    state.scroll_from_bottom[state.focus.index()] = 0;
                }
                KeyCode::Char('r') if !state.suggested_resolutions.is_empty() => {
                    clipboard_status = Some(copy_text(state.resolutions_clipboard()));
                }
                KeyCode::Char('i') if state.ignore_yaml.is_some() => {
                    clipboard_status = Some(copy_text(state.ignores_clipboard()));
                }
                _ => {}
            }
        }

        if state.worker_finished && !state.done && state.error.is_none() {
            state.done = true;
        }
    }

    if let Some(message) = state.error {
        return Err(message.into());
    }
    Ok(())
}

fn copy_text(payload: Option<String>) -> String {
    let Some(text) = payload else {
        return "empty".into();
    };
    match arboard::Clipboard::new().and_then(|mut c| c.set_text(text)) {
        Ok(()) => "copied".into(),
        Err(_) => "failed".into(),
    }
}

/// Channel reporter used by tests to prove the UI thread can progress while a worker blocks.
#[cfg(test)]
pub(crate) fn channel_reporter_pair() -> (SharedReporter, Receiver<UiEvent>) {
    let (tx, rx) = mpsc::channel();
    (Arc::new(ChannelReporter { tx }), rx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{ActivityKind, Phase};
    use std::time::Duration;

    #[test]
    fn worker_events_arrive_while_sleeping() {
        let (reporter, rx) = channel_reporter_pair();
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(30));
            reporter.emit(UiEvent::Activity {
                kind: ActivityKind::Info,
                message: "after sleep".into(),
            });
            reporter.emit(UiEvent::Phase(Phase::Done));
        });

        let mut saw = false;
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if let Ok(UiEvent::Activity { message, .. }) = rx.try_recv()
                && message == "after sleep"
            {
                saw = true;
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        handle.join().unwrap();
        assert!(saw);
    }

    #[test]
    fn terminal_guard_restores() {
        use std::io::IsTerminal;
        if !std::io::stdout().is_terminal() {
            return;
        }
        let guard = TerminalGuard::enter();
        drop(guard);
    }

    #[test]
    fn finish_interactive_keeps_worker_policy_exit() {
        let status = crate::RunStatus::from_max_ignore_severity(Some(
            crate::audit::Severity::Critical,
        ));
        let out = finish_interactive(Ok(()), Ok(status)).unwrap();
        assert_eq!(out.policy_exit(), 14);
    }

    #[test]
    fn finish_interactive_prefers_error() {
        let status = crate::RunStatus::from_max_ignore_severity(Some(
            crate::audit::Severity::Critical,
        ));
        let err = finish_interactive(Err("ui failed".into()), Ok(status)).unwrap_err();
        assert_eq!(err.to_string(), "ui failed");
    }
}
