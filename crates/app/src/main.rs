//! Terminal based ascii planet renderer
use std::time::Duration;

use crossterm::event::{self, KeyCode};
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, Frame};

/// General Result handler.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;


/// Main call to run the ratatui app.
fn main() -> Result<()> {
    ratatui::run(run)
}

/// Run the application loop.
/// Exits when the user presses 'q'.
fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(render)?;
        if should_quit()? {
            break;
        }
    }
    Ok(())
}

/// Render the application
fn render(frame: &mut Frame) {
    let greeting = Paragraph::new("Hello World! (press 'q' to quit)");
    frame.render_widget(greeting, frame.area());
}

/// Check if the user has pressed 'q' every 250ms.
fn should_quit() -> Result<bool> {
    if event::poll(Duration::from_millis(250))? {
        let q_pressed = event::read()?
            .as_key_press_event()
            .is_some_and(|key| key.code == KeyCode::Char('q'));
        return Ok(q_pressed);
    }
    Ok(false)
}
