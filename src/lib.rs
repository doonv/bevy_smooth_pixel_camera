//! # bevy_smooth_pixel_camera
//!
//! A bevy plugin that adds a simple smooth pixel camera.
//!
//! The smoothing is based on this video from aarthificial which explains how it works pretty nicely: <https://youtu.be/jguyR4yJb1M>
//!
//! This method allows for smooth camera movement while retaining the pixel perfection of low resolution rendering.
//!
//! ## Usage
//!
//! 1. Add the `bevy_smooth_pixel_camera` crate to your project.
//! ```sh
//! cargo add bevy_smooth_pixel_camera
//! ```
//! 2. Add the `PixelCameraPlugin` and set the `ImagePlugin` to `default_nearest`.
//! ```
//! use bevy::prelude::*;
//! use bevy_smooth_pixel_camera::prelude::*;
//!
//! App::new().add_plugins((
//!     DefaultPlugins.set(ImagePlugin::default_nearest()),
//!     PixelCameraPlugin
//! ));
//! ```
//! 3. Add a pixel pefect camera to your scene.
//! ```
//! use bevy::prelude::*;
//! use bevy_smooth_pixel_camera::prelude::*;
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn((
//!         Camera2dBundle::default(),
//!         PixelCamera::from_scaling(4)
//!     ));
//! }
//! ```
//! 4. That should be it! Make sure you move your camera via the `PixelCamera.subpixel_pos` property instead of the `Transform` component.
//!
//! ## Bevy Compatibility
//!
//! | bevy   | bevy_smooth_pixel_camera |
//! | ------ | ------------------------ |
//! | 0.12.0 | 0.1.0                    |

use bevy::{prelude::*, render::render_resource::*, window::WindowResolution};

pub mod components;
pub mod prelude;
pub mod systems;

/// The [`PixelCameraPlugin`] handles initialization and updates of the [`PixelCamera`](components::PixelCamera).
///
/// It also disables [`Msaa`].
pub struct PixelCameraPlugin;
impl Plugin for PixelCameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa::Off).add_systems(
            Update,
            (
                systems::init_camera,
                systems::update_viewport_size,
                systems::smooth_camera,
            ),
        );
    }
}

/// Given a scaling factor and a window resolution, this function
/// calculates the texture size of the viewport.
pub fn get_viewport_size(
    window_resolution: &WindowResolution,
    scaling: u8,
    smoothing: bool,
) -> Extent3d {
    // We need to make the viewport slightly larger when smoothing is enabled to
    // accommodate for the movement of the viewport.
    let size_extension = if smoothing { 2 } else { 0 };

    Extent3d {
        width: (window_resolution.width() / scaling as f32).ceil() as u32 + size_extension,
        height: (window_resolution.height() / scaling as f32).ceil() as u32 + size_extension,
        depth_or_array_layers: 1,
    }
}
