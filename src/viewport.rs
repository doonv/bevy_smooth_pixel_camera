//! Viewport Scaling and Stretching.

use bevy::render::camera::ClearColorConfig;
use bevy::render::render_resource::Extent3d;
use bevy::window::WindowResolution;

/// The way the viewport scales to fit the window.
#[doc(alias = "stretching")]
pub enum FitMode {
    /// The viewport will be stretched to the size of the window.
    Stretch,
    /// The viewport will be cropped into to fill the window.
    #[doc(alias = "fill")]
    Crop,
    /// The viewport will scale as large as possible without cropping and keeping aspect ratio.
    ///
    /// The unused space will be filled with the color.
    Fit(ClearColorConfig),
}

/// Different methods of calculating the viewport's size
pub enum ViewportSize {
    /// Each pixel's size is fixed.
    /// The viewport scales with the window.
    #[doc(alias = "WindowSize")]
    PixelFixed(u32),
    /// The viewport's size is fixed.
    /// If the window and viewport sizes do not match, the viewport will stretch.
    Fixed {
        /// The width of the viewport in logical pixels.
        width: u32,
        /// The height of the viewport in logical pixels.
        height: u32,
        /// The way the viewport scales to fit the window.
        fit: FitMode,
    },
    /// Keep the viewport's width fixed. The height
    /// will be adjusted to maintain aspect ratio.
    FixedWidth(u32),
    /// Keep the viewport's height fixed. The width
    /// will be adjusted to maintain aspect ratio.
    FixedHeight(u32),
    /// Keeping the aspect ratio while the axes can't be smaller than given minimum.
    AutoMin {
        /// The minimum width of the viewport in logical pixels.
        min_width: u32,
        /// The minimum height of the viewport in logical pixels.
        min_height: u32,
    },
    /// Keeping the aspect ratio while the axes can't be bigger than given maximum.
    AutoMax {
        /// The maximum width of the viewport in logical pixels.
        max_width: u32,
        /// The maximum height of the viewport in logical pixels.
        max_height: u32,
    },
    /// Use your own function for converting a window resolution to viewport size.
    Custom {
        /// The function used for converting a window resolution to viewport size.
        func: fn(&WindowResolution) -> (u32, u32),
        /// The way the viewport scales to fit the window.
        fit: FitMode,
    },
}

impl Default for ViewportSize {
    fn default() -> Self {
        Self::PixelFixed(4)
    }
}

impl ViewportSize {
    /// Calculates the size of the viewport based on the [`ViewportSize`] and the [`WindowResolution`].
    pub fn calculate(&self, window_resolution: &WindowResolution) -> Extent3d {
        let window_width = window_resolution.width();
        let window_height = window_resolution.height();

        match *self {
            ViewportSize::PixelFixed(scaling) => Extent3d {
                width: (window_width / scaling as f32).ceil() as u32,
                height: (window_height / scaling as f32).ceil() as u32,
                depth_or_array_layers: 1,
            },
            ViewportSize::Fixed { width, height, .. } => Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            ViewportSize::FixedWidth(width) => Extent3d {
                width,
                height: window_height as u32 * width / window_width as u32,
                depth_or_array_layers: 1,
            },
            ViewportSize::FixedHeight(height) => Extent3d {
                width: window_width as u32 * height / window_height as u32,
                height,
                depth_or_array_layers: 1,
            },
            ViewportSize::AutoMin {
                min_width,
                min_height,
            } => {
                // Compare Pixels of current width and minimal height and Pixels of minimal width with current height.
                // Then use bigger (min_height when true) as what it refers to (height when true) and calculate rest so it can't get under minimum.
                let (width, height) =
                    if window_width as u32 * min_height > min_width * window_height as u32 {
                        (
                            window_width as u32 * min_height / window_height as u32,
                            min_height,
                        )
                    } else {
                        (
                            min_width,
                            window_height as u32 * min_width / window_width as u32,
                        )
                    };

                Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                }
            }
            ViewportSize::AutoMax {
                max_width,
                max_height,
            } => {
                // Compare Pixels of current width and minimal height and Pixels of minimal width with current height.
                // Then use bigger (min_height when true) as what it refers to (height when true) and calculate rest so it can't get under minimum.
                let (width, height) =
                    if window_width as u32 * max_height < max_width * window_height as u32 {
                        (
                            window_width as u32 * max_height / window_height as u32,
                            max_height,
                        )
                    } else {
                        (
                            max_width,
                            window_height as u32 * max_width / window_width as u32,
                        )
                    };

                Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                }
            }
            ViewportSize::Custom { func, .. } => {
                let (width, height) = func(window_resolution);

                Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                }
            }
        }
    }
    /// Returns the clear color for this [`ViewportSize`] if the current variant 
    /// has a [`FitMode::Fit`], otherwise returns [`ClearColorConfig::None`].
    pub fn clear_color(&self) -> ClearColorConfig {
        if let ViewportSize::Fixed {
            fit: FitMode::Fit(config),
            ..
        }
        | ViewportSize::Custom {
            fit: FitMode::Fit(config),
            ..
        } = self
        {
            config.clone()
        } else {
            ClearColorConfig::None
        }
    }
}
