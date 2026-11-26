use crate::grid::Grid;

/// Color struct in RGB
#[derive(Debug, Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    /// Transforms the rgb color to hsv
    pub fn hsv(&self) -> (f32, f32, f32) {
        let r = self.0 as f32 / 255.0;
        let g = self.1 as f32 / 255.0;
        let b = self.2 as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta).rem_euclid(6.0))
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;

        (h, s, v)
    }
}

/// Cell structure acts like a pixel for ASCII
#[derive(Debug, Clone, Copy)]
pub struct Cell {
    /// Symbol is the ASCII character to show, NONE to show nothing
    pub symbol: Option<char>,
    /// Foregroud color
    pub fg: Color,
    /// Background color, NONE for "transparent"
    pub bg: Option<Color>,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            symbol: None,
            fg: Color(u8::MAX, u8::MAX, u8::MAX),
            bg: None,
        }
    }
}

/// Represents the canvas where everything is rendered
/// It keeps a vector of all the cells, and the settings of the render
pub type Canvas = Grid<Cell>;

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-4;

    fn assert_hsv_approx_eq(actual: (f32, f32, f32), expected: (f32, f32, f32)) {
        assert!(
            (actual.0 - expected.0).abs() < EPSILON
                && (actual.1 - expected.1).abs() < EPSILON
                && (actual.2 - expected.2).abs() < EPSILON,
            "expected {:?}, got {:?}",
            expected,
            actual
        );
    }

    #[test]
    fn test_hsv_greyscale() {
        assert_hsv_approx_eq(Color(0, 0, 0).hsv(), (0., 0., 0.));
        assert_hsv_approx_eq(Color(255, 255, 255).hsv(), (0., 0., 1.));
    }

    #[test]
    fn test_hsv_hues() {
        assert_hsv_approx_eq(Color(255, 0, 0).hsv(), (0., 1., 1.));
        assert_hsv_approx_eq(Color(0, 255, 0).hsv(), (120., 1., 1.));
        assert_hsv_approx_eq(Color(0, 0, 255).hsv(), (240., 1., 1.));
        // Magenta checks the negative hue wrap
        assert_hsv_approx_eq(Color(255, 0, 255).hsv(), (300., 1., 1.));
    }
}
