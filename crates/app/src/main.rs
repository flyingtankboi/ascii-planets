//! Terminal based ascii planet renderer
use clap::Parser;
use crossterm::event::{self, KeyCode};
use ratatui::{DefaultTerminal, Frame};
use std::time::{Duration, Instant};

use render::settings::Settings;
use scene::Scene;

use crate::demo::DemoOption;
use crate::scene_widget::SceneWidget;
use crate::input::Input;

mod input;
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
    let (mut scene, settings) = args.demo.build();
    ratatui::run(|terminal| run(terminal, &mut scene, &settings))
}

/// Run the application loop.
/// Exits when the user presses 'q'.
fn run(terminal: &mut DefaultTerminal, scene: &mut Scene, settings: &Settings) -> Result<()> {
    let fps = 30;
    let mut input = Input::new()?;
    let frame_time = Duration::from_secs_f32(1. / fps as f32);
    let mut last = Instant::now();
    loop {
        let now = Instant::now();
        let dt = (now - last).as_secs_f32() * 1000.;
        last = now;

        let frame = input.read_until(now + frame_time)?;
        if frame.should_quit { break; }
        if frame.next_target { scene.next_camera_target(); }
        scene.step(dt, &frame.direction);

        terminal.draw(|f| render(f, scene, settings))?;
    }
    Ok(())
}

/// Render the application
fn render(frame: &mut Frame, scene: &Scene, settings: &Settings) {
    frame.render_widget(SceneWidget { scene, settings }, frame.area());
}
