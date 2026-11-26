//! Ratatui widget rendering a scene in the terminal
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use render::{canvas::Color as RenderColor, renderer::render, settings::Settings};
use scene::Scene;

/// Widget rendering a [`Scene`] to fit the area it is drawn in
pub struct SceneWidget<'a> {
    /// Scene to render
    pub scene: &'a Scene,
    /// Graphical settings of the render
    /// see [`Settings`]
    pub settings: &'a Settings,
}

impl Widget for SceneWidget<'_> {
    /// Renders the scene to a canvas the size of `area`, then copies it into the buffer
    fn render(self, area: Rect, buf: &mut Buffer) {
        let canvas = render(
            self.scene,
            self.settings,
            area.width as usize,
            area.height as usize,
        );

        for (x, y, cell) in canvas.iter() {
            // No symbol means nothing to draw
            let Some(symbol) = cell.symbol else {
                continue;
            };
            let position = (area.left() + x as u16, area.top() + y as u16);
            if let Some(ui_cell) = buf.cell_mut(position) {
                let mut style = Style::default().fg(to_ratatui_color(cell.fg));
                if let Some(bg) = cell.bg {
                    style = style.bg(to_ratatui_color(bg));
                }
                ui_cell.set_char(symbol).set_style(style);
            }
        }
    }
}

/// Converts a render color to a ratatui RGB color
fn to_ratatui_color(color: RenderColor) -> Color {
    Color::Rgb(color.0, color.1, color.2)
}
