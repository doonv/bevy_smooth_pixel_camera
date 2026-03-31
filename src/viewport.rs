//! Viewport Scaling and Stretching.

use bevy::camera::{ClearColorConfig, ScalingMode};
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;

/// The way the viewport scales to fit the window.
#[doc(alias = "Stretching")]
pub enum FitMode {
    /// Stretch viewport will to the size of the window.
    Stretch,
    /// Scale the viewport to cover the entire window; edges may be cropped.
    #[doc(alias = "Crop")]
    Fill,
    /// Scale the viewport as large as possible without cropping.
    ///
    /// Any empty space (letterboxing) is cleared using the [`ClearColorConfig`].
    Fit(ClearColorConfig),
}

/// Different methods of calculating the viewport's size
pub enum ViewportScalingMode {
    /// Each pixel's size is fixed.
    /// The viewport scales with the window.
    #[doc(alias = "WindowSize")]
    PixelSize(f32),
    /// Fixed viewport size.
    Fixed {
        /// The width of the viewport in logical pixels.
        width: f32,
        /// The height of the viewport in logical pixels.
        height: f32,
        /// The way the viewport scales to fit the window.
        fit: FitMode,
    },
    /// Keep the viewport's width fixed. The height
    /// will be adjusted to maintain aspect ratio.
    FixedWidth(f32),
    /// Keep the viewport's height fixed. The width
    /// will be adjusted to maintain aspect ratio.
    FixedHeight(f32),
    /// Keeping the aspect ratio while the axes can't be smaller than given minimum.
    AutoMin {
        /// The minimum width of the viewport in logical pixels.
        min_width: f32,
        /// The minimum height of the viewport in logical pixels.
        min_height: f32,
    },
    /// Keeping the aspect ratio while the axes can't be bigger than given maximum.
    AutoMax {
        /// The maximum width of the viewport in logical pixels.
        max_width: f32,
        /// The maximum height of the viewport in logical pixels.
        max_height: f32,
    },
    /// Define your own function for converting a window resolution to viewport size.
    Custom {
        /// The function used for converting a window resolution to viewport size.
        func: fn(Vec2) -> Vec2,
        /// The way the viewport scales to fit the window.
        fit: FitMode,
    },
}

impl Default for ViewportScalingMode {
    fn default() -> Self {
        Self::PixelSize(4.0)
    }
}

impl ViewportScalingMode {
    /// Calculates the size of the viewport based on the [`ViewportScalingMode`] and the provided window size.
    #[must_use]
    pub fn calculate(&self, window_size: Vec2) -> Vec2 {
        match *self {
            Self::PixelSize(scaling) => Vec2::new(window_size.x / scaling, window_size.y / scaling),
            Self::Fixed {
                width,
                height,
                fit: _,
            } => Vec2::new(width, height),
            Self::FixedWidth(width) => Vec2::new(width, window_size.y * width / window_size.x),
            Self::FixedHeight(height) => Vec2::new(window_size.x * height / window_size.y, height),
            Self::AutoMin {
                min_width,
                min_height,
            } => {
                if window_size.x * min_height > min_width * window_size.y {
                    Vec2::new(window_size.x * min_height / window_size.y, min_height)
                } else {
                    Vec2::new(min_width, window_size.y * min_width / window_size.x)
                }
            }
            Self::AutoMax {
                max_width,
                max_height,
            } => {
                if window_size.x * max_height < max_width * window_size.y {
                    Vec2::new(window_size.x * max_height / window_size.y, max_height)
                } else {
                    Vec2::new(max_width, window_size.y * max_width / window_size.x)
                }
            }
            Self::Custom { func, fit: _ } => func(window_size),
        }
    }

    /// Returns the clear color for this [`ViewportScalingMode`] if the current variant
    /// has a [`FitMode::Fit`].
    #[must_use]
    pub const fn clear_color(&self) -> Option<ClearColorConfig> {
        if let Self::Fixed {
            fit: FitMode::Fit(config),
            ..
        }
        | Self::Custom {
            fit: FitMode::Fit(config),
            ..
        } = self
        {
            Some(*config)
        } else {
            None
        }
    }

    /// Returns the internal texture size and the camera projection scaling for a given window size.
    #[must_use]
    pub(crate) fn get_configuration(
        &self,
        window_size: Vec2,
        smoothing: bool,
    ) -> (Extent3d, ScalingMode) {
        let base = self.calculate(window_size);
        let aspect = window_size.x / window_size.y;

        let (view_w, view_h) = match self {
            Self::Fixed { fit, .. } | Self::Custom { fit, .. } => match fit {
                FitMode::Fit(_) => {
                    if aspect > base.x / base.y {
                        (base.y * aspect, base.y)
                    } else {
                        (base.x, base.x / aspect)
                    }
                }
                FitMode::Fill => {
                    let axis = base.x.min(base.y);
                    if aspect > 1.0 {
                        (axis * aspect, axis)
                    } else {
                        (axis, axis / aspect)
                    }
                }
                FitMode::Stretch => (base.x, base.y),
            },
            _ => (base.x, base.y),
        };

        let mut tex_w = base.x.ceil() as u32;
        let mut tex_h = base.y.ceil() as u32;

        if smoothing {
            tex_w += 1;
            tex_h += 1;
        }

        (
            Extent3d {
                width: tex_w,
                height: tex_h,
                ..default()
            },
            ScalingMode::Fixed {
                width: view_w,
                height: view_h,
            },
        )
    }
}
