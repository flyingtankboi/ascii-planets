//! ToneMap stage: turns unbounded luminance values back to RGB, only linear transformations
//! of the color space

use glam::Vec3;

use crate::grid::Grid;
use crate::stages::shading::ShadeFragment;

/// The different bounding operators to get to [0,1]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Operator {
    /// Clamps values at a maximum number before scaling to [0,1]
    Clamp { clamp: f32 },
    /// Applies Reinhard clamping
    #[default]
    Reinhard,
    /// Applies Reinhard clamping where the value white_luminance will map to 1
    ReinhardExtended { white_luminance: f32 },
}

impl Operator {
    /// Evaluate the correct operator function depending on enum value
    pub(crate) fn eval(&self, x: f32) -> f32 {
        match *self {
            Operator::Clamp { clamp } => clamping(x, clamp),
            Operator::Reinhard => reinhard(x),
            Operator::ReinhardExtended { white_luminance } => reinhard_extended(x, white_luminance),
        }
    }
}

// Enum for settings of how to apply the clamping
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ApplyTo {
    PerChannel,
    #[default]
    Luminance,
}

impl ApplyTo {
    /// Apply the bounding operator depending on the strategy
    pub(crate) fn apply(&self, color: Vec3, op: &Operator) -> Vec3 {
        match *self {
            ApplyTo::PerChannel => Vec3::new(op.eval(color.x), op.eval(color.y), op.eval(color.z)),
            ApplyTo::Luminance => {
                let luminance = color.dot(Vec3::new(0.2126, 0.7152, 0.0722));
                if luminance <= 0.0 {
                    return Vec3::ZERO;
                }
                let mapped_luminance = op.eval(luminance);
                color * (mapped_luminance / luminance)
            }
        }
    }
}

/// Settings for the tonemap stage.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ToneMapSettings {
    /// Exposure value in stops. +1 stops means doubling the luminance, -1 means to halve it
    /// 2^exposure_stops
    pub exposure_stops: f32,
    /// Clamping operator to transform unbounded luminance to [0,1]
    pub operator: Operator,
    /// Should it apply per RGB channel or to the luminance
    pub apply_to: ApplyTo,
}

/// Output of the toning stage for one cell.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct ToneFragment {
    /// Linear radiance, bounded.
    pub color: Vec3,
}

pub(crate) fn tonemap(
    shade_grid: Grid<ShadeFragment>,
    settings: ToneMapSettings,
) -> Grid<ToneFragment> {
    shade_grid.map(|shade| {
        let mut color = if shade.hit { shade.color } else { Vec3::ZERO };
        // Exposure
        color *= settings.exposure_stops.exp2();
        // Operator
        color = settings.apply_to.apply(color, &settings.operator);
        // Security clamp
        color = color.map(|x| x.clamp(0., 1.));
        ToneFragment { color }
    })
}

/// Clamps value between 0 and clamp then scales it to 0-1
/// A clamp value of zero gives zero
fn clamping(value: f32, clamp: f32) -> f32 {
    if clamp <= 0. {
        0.
    } else {
        value.clamp(0., clamp) / clamp
    }
}

/// Reinhard clamping
fn reinhard(value: f32) -> f32 {
    value / (value + 1.)
}

/// Reinhard clamping with a white value that maps to 1
fn reinhard_extended(value: f32, white_luminance: f32) -> f32 {
    if white_luminance <= 0. {
        0.
    } else {
        value * (1. + value / (white_luminance.powf(2.))) / (value + 1.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn assert_vec3_approx_eq(actual: Vec3, expected: Vec3) {
        assert!(
            (actual - expected).length() < EPSILON,
            "expected {:?}, got {:?}",
            expected,
            actual
        );
    }

    fn tonemap_one(color: Vec3, hit: bool, settings: ToneMapSettings) -> Vec3 {
        let grid = Grid::new(1, 1, ShadeFragment { hit, color });
        tonemap(grid, settings)[(0, 0)].color
    }

    #[test]
    fn test_clamping() {
        assert_eq!(clamping(5., 0.), 0.);
        assert_eq!(clamping(1., 2.), 0.5);
        assert_eq!(clamping(5., 2.), 1.);
    }

    #[test]
    fn test_reinhard() {
        assert_eq!(reinhard(0.), 0.);
        assert_eq!(reinhard(1.), 0.5);
        // The white luminance maps exactly to 1
        assert!((reinhard_extended(4., 4.) - 1.).abs() < EPSILON);
        assert_eq!(reinhard_extended(4., 0.), 0.);
    }

    #[test]
    fn test_luminance_keeps_channel_ratios() {
        let color = Vec3::new(2., 1., 0.5);
        let mapped = ApplyTo::Luminance.apply(color, &Operator::Reinhard);
        assert_vec3_approx_eq(mapped / mapped.x, color / color.x);
        assert_eq!(
            ApplyTo::Luminance.apply(Vec3::ZERO, &Operator::Reinhard),
            Vec3::ZERO
        );
    }

    #[test]
    fn test_per_channel() {
        let mapped = ApplyTo::PerChannel.apply(Vec3::new(0., 1., 3.), &Operator::Reinhard);
        assert_vec3_approx_eq(mapped, Vec3::new(0., 0.5, 0.75));
    }

    #[test]
    fn test_miss_is_black() {
        let color = tonemap_one(Vec3::ONE, false, ToneMapSettings::default());
        assert_eq!(color, Vec3::ZERO);
    }

    #[test]
    fn test_exposure_doubles_per_stop() {
        let settings = ToneMapSettings {
            exposure_stops: 1.,
            operator: Operator::Clamp { clamp: 10. },
            apply_to: ApplyTo::PerChannel,
        };
        let color = tonemap_one(Vec3::splat(1.), true, settings);
        assert_vec3_approx_eq(color, Vec3::splat(0.2));
    }

    #[test]
    fn test_output_is_bounded() {
        let settings = ToneMapSettings {
            exposure_stops: 10.,
            operator: Operator::Clamp { clamp: 1. },
            apply_to: ApplyTo::Luminance,
        };
        let color = tonemap_one(Vec3::new(100., -5., 0.), true, settings);
        assert!(color.cmpge(Vec3::ZERO).all() && color.cmple(Vec3::ONE).all());
    }
}
