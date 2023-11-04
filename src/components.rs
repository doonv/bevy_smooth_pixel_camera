use bevy::{prelude::*, render::view::RenderLayers};

/// The pixelated camera component.
///
/// Add this component to a [`Camera2dBundle`] in order to turn it into a
/// pixelated camera.
///
/// **Warning:** In order to move the camera please use the `subpixel_pos`
/// attribute instead of the [`Transform`] component (the transform is a truncated version of subpixel_pos)
#[derive(Component)]
pub struct PixelCamera {
    /// The level of upscaling to use for pixels.
    ///
    /// For example: A scaling of `4` which cause every world pixel to be 4x4 in size on the screen.
    pub scaling: u8,
    /// The subpixel position of the [`PixelCamera`], use this instead of the camera's [`Transform`].
    pub subpixel_pos: Vec2,
    /// The order in which the viewport camera renders.
    /// Cameras with a higher order are rendered later, and thus on top of lower order cameras.
    ///
    /// Because we want the world camera to render before the viewport camera, set this value to a positive number.
    pub viewport_order: isize,
    /// The rendering layer the viewport is on.
    pub viewport_layer: RenderLayers,
    /// Whether camera position smoothing is enabled for this camera.
    pub smoothing: bool,
}

impl Default for PixelCamera {
    fn default() -> Self {
        Self {
            viewport_order: 1,
            scaling: 2,
            viewport_layer: RenderLayers::layer(1),
            subpixel_pos: Vec2::ZERO,
            smoothing: true,
        }
    }
}

impl PixelCamera {
    /// Creates a new pixel camera with the `scaling` of choice and default configuration.
    pub fn from_scaling(scaling: u8) -> Self {
        Self {
            scaling,
            ..default()
        }
    }
}

// TODO: Replace these components when we get entity relationships or something like that
#[derive(Component)]
pub struct PixelViewport(pub Entity);
#[derive(Component)]
pub struct PixelViewportMarker;
