//! Keyboard input handling: turns terminal events into actions and orbit directions
use std::collections::HashMap;
use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{
    self, DisableFocusChange, EnableFocusChange, Event, KeyCode, KeyEventKind,
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::{execute, terminal};
use scene::camera::OrbitDirection;

/// Without key releases, a key counts as held for this long after its last press or repeat
const HOLD_DURATION: Duration = Duration::from_millis(100);

/// What the user asked for during one frame
#[derive(Debug, Default)]
pub struct FrameInput {
    /// `q` was pressed
    pub should_quit: bool,
    /// Tab was pressed
    pub next_target: bool,
    /// Directions held during the frame
    pub direction: OrbitDirection,
}

/// Keeps track of the keyboard between frames
pub struct Input {
    /// Whether the terminal reports key releases (kitty keyboard protocol)
    reports_release: bool,
    /// Held keys and when they were last seen
    held: HashMap<KeyCode, Instant>,
}

impl Input {
    /// Detects key release support and enables it when available, along with focus events.
    /// Must be called once ratatui has set up the terminal.
    pub fn new() -> io::Result<Self> {
        let reports_release = terminal::supports_keyboard_enhancement()?;
        if reports_release {
            execute!(
                io::stdout(),
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                )
            )?;
        }
        execute!(io::stdout(), EnableFocusChange)?;
        Ok(Input {
            reports_release,
            held: HashMap::new(),
        })
    }

    /// Waits for and reads every event until `deadline`, then gives what was asked this frame
    pub fn read_until(&mut self, deadline: Instant) -> io::Result<FrameInput> {
        let mut frame = FrameInput::default();
        loop {
            let time_left = deadline.saturating_duration_since(Instant::now());
            if !event::poll(time_left)? {
                break;
            } else {
                self.handle_event(event::read()?, &mut frame);
            }
        }

        frame.direction = self.direction();
        Ok(frame)
    }

    /// Updates the held keys and the frame's actions from one event
    fn handle_event(&mut self, event: Event, frame: &mut FrameInput) {
        match event {
            Event::Key(key) => match key.kind {
                KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Tab => frame.next_target = true,
                        KeyCode::Char('q') => frame.should_quit = true,
                        _ => (),
                    };
                    self.held.insert(key.code, Instant::now());
                }
                KeyEventKind::Repeat => {
                    self.held.insert(key.code, Instant::now());
                }
                KeyEventKind::Release => {
                    self.held.remove(&key.code);
                }
            },
            Event::FocusLost => self.held.clear(),
            _ => (),
        };
    }

    /// Whether `key` is currently held
    fn is_held(&self, key: KeyCode) -> bool {
        if self.reports_release {
            self.held.contains_key(&key)
        } else {
            self.held
                .get(&key)
                .is_some_and(|&instant| (instant.elapsed()) < HOLD_DURATION)
        }
    }

    /// Builds the orbit direction from the held keys
    fn direction(&self) -> OrbitDirection {
        OrbitDirection {
            up: self.is_held(KeyCode::Up),
            down: self.is_held(KeyCode::Down),
            left: self.is_held(KeyCode::Left),
            right: self.is_held(KeyCode::Right),
            closer: self.is_held(KeyCode::Char('=')),
            further: self.is_held(KeyCode::Char('-')),
        }
    }
}

impl Drop for Input {
    /// Restores the terminal's keyboard modes, runs even when `run` exits early with an error
    fn drop(&mut self) {
        // Errors are ignored, there is nothing left to do if restoring fails
        if self.reports_release {
            let _ = execute!(io::stdout(), PopKeyboardEnhancementFlags);
        }
        let _ = execute!(io::stdout(), DisableFocusChange);
    }
}
