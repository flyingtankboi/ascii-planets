//! ToneMap stage: turns unbounded luminance values back to RGB, only linear transformations
//! of the color space

use glam::Vec3;

use crate::stages::shading::ShadeFragment;
use crate::grid::Grid;

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

pub(crate) fn tonemap(shade_grid: Grid<ShadeFragment>, settings: ToneMapSettings) -> Grid<ToneFragment> {
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
