//! Interactive ratatui surfaces.

mod render;
mod state;

pub use state::{DoneCommand, PaneId, UiState};

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
    let (status_tx, status_rx) = mpsc::channel::<WorkerOutcome>();
    let reporter: SharedReporter = Arc::new(ChannelReporter { tx });

    let initial = match verb {
        Verb::Auto => WorkKind::Auto { refresh_first },
        _ => WorkKind::Check,
    };
    spawn_work(
        reporter.clone(),
        options,
        initial,
        status_tx.clone(),
    );

    event_loop(
        verb,
        version,
        rx,
        status_rx,
        reporter,
        options,
        status_tx,
    )
}

#[derive(Clone)]
enum WorkKind {
    Auto { refresh_first: bool },
    Check,
    ApplyIgnores(Vec<String>),
    ApplyResolutions(Vec<(String, String)>),
}

type WorkerOutcome = (Result<RunStatus, Error>, bool);

fn work_counts_as_auto_ignore(kind: &WorkKind, options: crate::cmd::Options) -> bool {
    matches!(kind, WorkKind::Auto { .. } | WorkKind::ApplyResolutions(_)) && options.auto_ignore()
}

fn spawn_work(
    reporter: SharedReporter,
    options: crate::cmd::Options,
    kind: WorkKind,
    status_tx: Sender<WorkerOutcome>,
) {
    let is_policy = work_counts_as_auto_ignore(&kind, options);
    thread::spawn(move || {
        let run_reporter = reporter.clone();
        let result = (|| {
            let mut gnarl = Gnarl::with_reporter(options, reporter.clone())?;
            match kind {
                WorkKind::Auto { refresh_first } => gnarl.auto(refresh_first),
                WorkKind::Check => gnarl.check(),
                WorkKind::ApplyIgnores(ids) => gnarl.apply_ignores_then_check(&ids),
                WorkKind::ApplyResolutions(entries) => {
                    gnarl.apply_resolutions_then_auto(&entries)
                }
            }
        })();
        if let Err(ref err) = result {
            run_reporter.emit(UiEvent::Error {
                message: err.to_string(),
            });
        }
        let _ = status_tx.send((result, is_policy));
    });
}

#[cfg(test)]
pub(crate) fn finish_interactive(
    ui: Result<(), Error>,
    worker: Result<RunStatus, Error>,
) -> Result<RunStatus, Error> {
    match (ui, worker) {
        (Ok(()), Ok(status)) => Ok(status),
        (Err(e), _) | (_, Err(e)) => Err(e),
    }
}

fn event_loop(
    verb: Verb,
    version: &str,
    rx: Receiver<UiEvent>,
    status_rx: Receiver<WorkerOutcome>,
    reporter: SharedReporter,
    options: crate::cmd::Options,
    status_tx: Sender<WorkerOutcome>,
) -> Result<RunStatus, Error> {
    let mut guard = TerminalGuard::enter()?;
    let mut state = UiState::new(verb);
    state
        .activity
        .push(format!("[INFO] gnarl {version}"));
    let started = Instant::now();
    let mut clipboard_status: Option<String> = None;
    let mut worker_running = true;
    let mut session_status = RunStatus::ok();

    loop {
        let running = !state.done && state.error.is_none();
        while let Ok(event) = rx.try_recv() {
            state.apply(event);
        }
        while let Ok((result, is_policy)) = status_rx.try_recv() {
            worker_running = false;
            match result {
                Ok(status) => {
                    session_status =
                        crate::status::merge_session_policy(session_status, status, is_policy);
                }
                Err(err) => {
                    if state.error.is_none() {
                        state.error = Some(err.to_string());
                    }
                }
            }
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
                KeyCode::Esc if state.maximized => {
                    state.maximized = false;
                }
                code if should_quit(code, state.maximized) => {
                    break;
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
                KeyCode::Char(c) => match state.done_command(c, worker_running) {
                    Some(DoneCommand::CopyResolutions) => {
                        clipboard_status = Some(copy_text(state.resolutions_clipboard()));
                    }
                    Some(DoneCommand::CopyIgnores) => {
                        clipboard_status = Some(copy_text(state.ignores_clipboard()));
                    }
                    Some(DoneCommand::ApplyIgnores) => {
                        let ids = state.ignore_apply_ids.clone();
                        state.begin_ignore_refresh();
                        worker_running = true;
                        spawn_work(
                            reporter.clone(),
                            options,
                            WorkKind::ApplyIgnores(ids),
                            status_tx.clone(),
                        );
                    }
                    Some(DoneCommand::ApplyResolutions) => {
                        let entries = state.resolution_entries.clone();
                        state.begin_resolution_auto();
                        worker_running = true;
                        spawn_work(
                            reporter.clone(),
                            options,
                            WorkKind::ApplyResolutions(entries),
                            status_tx.clone(),
                        );
                    }
                    None => {}
                },
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
    Ok(session_status)
}

fn should_quit(code: KeyCode, maximized: bool) -> bool {
    match code {
        KeyCode::Esc if maximized => false,
        KeyCode::Esc | KeyCode::Char('q') => true,
        _ => false,
    }
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

    #[test]
    fn quit_keys_always_leave_except_esc_when_maximized() {
        assert!(should_quit(KeyCode::Char('q'), false));
        assert!(should_quit(KeyCode::Char('q'), true));
        assert!(should_quit(KeyCode::Esc, false));
        assert!(!should_quit(KeyCode::Esc, true));
        assert!(!should_quit(KeyCode::Enter, false));
    }

    #[test]
    fn sequential_workers_can_complete_two_cycles() {
        let (status_tx, status_rx) = mpsc::channel::<WorkerOutcome>();
        let tx1 = status_tx.clone();
        thread::spawn(move || {
            tx1.send((Ok(RunStatus::ok()), false)).unwrap();
        });
        let (first, first_policy) = status_rx.recv().unwrap();
        let mut session = crate::status::merge_session_policy(
            RunStatus::ok(),
            first.unwrap(),
            first_policy,
        );
        thread::spawn(move || {
            status_tx
                .send((
                    Ok(RunStatus::from_max_ignore_severity(Some(
                        crate::audit::Severity::Critical,
                    ))),
                    true,
                ))
                .unwrap();
        });
        let (second, second_policy) = status_rx.recv().unwrap();
        session = crate::status::merge_session_policy(session, second.unwrap(), second_policy);
        assert_eq!(session.policy_exit(), 14);
    }

    #[test]
    fn finish_interactive_returns_session_status_after_quit() {
        let status = crate::RunStatus::from_max_ignore_severity(Some(
            crate::audit::Severity::High,
        ));
        let out = finish_interactive(Ok(()), Ok(status)).unwrap();
        assert_eq!(out.policy_exit(), 13);
    }
}
