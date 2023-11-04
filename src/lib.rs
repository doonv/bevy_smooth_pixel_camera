//! # bevy_smooth_pixel_camera
//!
//! A bevy plugin that adds a simple smooth pixel camera.
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
//! use bevy_smooth_pixel_camera::PixelCameraPlugin;
//!
//! App::new().add_plugins((
//!     DefaultPlugins.set(ImagePlugin::default_nearest()),
//!     PixelCameraPlugin
//! ));
//! ```
//! 3. Add a pixel pefect camera to your scene.
//! ```
//! use bevy::prelude::*;
//! use bevy_smooth_pixel_camera::PixelCamera;
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn((
//!         Camera2dBundle::default(),
//!         PixelCamera::from_scaling(4)
//!     ));
//! }
//! ```
//! 4. That should be it!

use bevy::{prelude::*, render::render_resource::*, window::WindowResolution};

pub mod components;
pub mod prelude;
pub mod systems;

/// The [`PixelCameraPlugin`] handles initialization and updates of the [`PixelCamera`].
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

pub fn get_viewport_size(window_resolution: &WindowResolution, scaling: u8) -> Extent3d {
    Extent3d {
        width: (window_resolution.width() / scaling as f32).ceil() as u32 + 2,
        height: (window_resolution.height() / scaling as f32).ceil() as u32 + 2,
        depth_or_array_layers: 1,
    }
}
