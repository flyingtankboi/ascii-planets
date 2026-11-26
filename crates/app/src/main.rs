//! Terminal based ascii planet renderer
use clap::Parser;
use crossterm::event::{self, KeyCode};
use ratatui::{DefaultTerminal, Frame};
use std::time::Duration;

use render::settings::Settings;
use scene::Scene;

use crate::demo::DemoOption;
use crate::scene_widget::SceneWidget;

mod demo;
mod scene_widget;

/// General Result handler.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// A terminal renderer for custom planets for your ricing or personal pleasure
#[derive(Parser)]
struct Args {
    /// Which demo scene and setting to launch from
    #[arg(long, value_enum, default_value_t = DemoOption::Simple)]
    demo: DemoOption,
}

/// Main call to run the ratatui app.
fn main() -> Result<()> {
    let args = Args::parse();
    let (scene, settings) = args.demo.fetch();
    ratatui::run(|terminal| run(terminal, &scene, &settings))
}

/// Run the application loop.
/// Exits when the user presses 'q'.
fn run(terminal: &mut DefaultTerminal, scene: &Scene, settings: &Settings) -> Result<()> {
    loop {
        terminal.draw(|frame| render(frame, scene, settings))?;
        if should_quit()? {
            break;
        }
    }
    Ok(())
}

/// Render the application
fn render(frame: &mut Frame, scene: &Scene, settings: &Settings) {
    frame.render_widget(SceneWidget { scene, settings }, frame.area());
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
