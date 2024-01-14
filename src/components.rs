//! The components of [`bevy_smooth_pixel_camera`](crate).

use bevy::prelude::*;
use bevy::render::view::RenderLayers;

use crate::viewport::ViewportSize;

/// The pixelated camera component.
///
/// Add this component to a [`Camera2dBundle`] in order to turn it into a
/// pixelated camera.
///
/// **Warning:** In order to move the camera please use the `subpixel_pos`
/// attribute instead of the [`Transform`] component (the transform is a truncated version of subpixel_pos (for pixel perfect snapping))
#[derive(Component)]
pub struct PixelCamera {
    /// The size of the viewport.
    ///
    /// See [`ViewportSize`] for details.
    pub viewport_size: ViewportSize,
    /// The subpixel position of the [`PixelCamera`], use this instead of the camera's [`Transform`].
    pub subpixel_pos: Vec2,
    /// The order in which the viewport camera renders.
    /// Cameras with a higher order are rendered later, and thus on top of lower order cameras.
    ///
    /// Because we want the world camera to render before the viewport camera,
    /// set this value to a number higher the than the world camera's order.
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
            viewport_size: ViewportSize::PixelFixed(4),
            viewport_layer: RenderLayers::layer(1),
            subpixel_pos: Vec2::ZERO,
            smoothing: true,
        }
    }
}

impl PixelCamera {
    /// Creates a new pixel camera with the `size` of choice and default configuration.
    pub fn from_size(viewport_size: ViewportSize) -> Self {
        Self {
            viewport_size,
            ..default()
        }
    }
    /// Creates a new pixel camera with the `scaling` of choice and default configuration.'
    #[deprecated(since = "0.2.0", note = "`from_size` should be used instead")]
    pub fn from_scaling(scaling: u8) -> Self {
        Self {
            viewport_size: ViewportSize::PixelFixed(scaling.into()),
            ..default()
        }
    }
}

// TODO: Replace these components when we get entity relationships or something like that
#[derive(Component)]
pub(crate) struct PixelViewportReferences {
    pub camera: Entity,
    pub sprite: Entity,
}
#[derive(Component)]
pub(crate) struct PixelViewport;
#[derive(Component)]
pub(crate) struct ViewportCamera;
