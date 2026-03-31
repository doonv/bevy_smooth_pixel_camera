//! A bevy plugin that adds a simple smooth pixel camera.
//!
//! It works by rendering the main camera to a small viewport which is then rendered by a second camera spawned by the plugin.
//! This allows for hybrid rendering of both a pixelated world and high resolution assets on top.
//!
//! This plugin has a [smoothing] feature, which makes the camera's movement appear smooth while keeping the world itself locked
//! to a pixel grid. It works by moving the canvas in the opposite direction of the world camera's subpixel position. See the
//! [`how_smoothing_works`] example for a demonstration of how it works behind the scenes.
//!
//! | Smoothing OFF                                                                   | Smoothing ON                                                                                    |
//! | :-----------------------------------------------------------------------------: | :---------------------------------------------------------------------------------------------: |
//! | ![The camera is locked to the pixel grid, causing jagged motion][smoothing_off] | ![The camera moves smoothly while the world itself stays locked to a pixel grid.][smoothing_on] |
//!
//! ## Usage
//!
//! 1. Add the `bevy_smooth_pixel_camera` crate to your project.
//!
//!     ```sh
//!     cargo add bevy_smooth_pixel_camera
//!     ```
//!
//! 2. Add the [`PixelCameraPlugin`] and set [`ImagePlugin`] to [`default_nearest`].
//!
//!     ```no_run
//!     use bevy::prelude::*;
//!     use bevy_smooth_pixel_camera::prelude::*;
//!     
//!     App::new().add_plugins((
//!         DefaultPlugins.set(ImagePlugin::default_nearest()),
//!         PixelCameraPlugin
//!     )).run();
//!     ```
//!
//! 3. Add a [`PixelCamera`](crate::components::PixelCamera) to your world.
//!
//!     ```
//!     use bevy::prelude::*;
//!     use bevy_smooth_pixel_camera::prelude::*;
//!
//!     fn setup(mut commands: Commands) {
//!         commands.spawn(PixelCamera::from_size(ViewportScalingMode::PixelSize(4.0)));
//!     }
//!     ```
//!
//! 4. That's it!
//!
//! ## Bevy Compatibility
//!
//! | bevy   | bevy_smooth_pixel_camera |
//! | ------ | ------------------------ |
//! | 0.18.* | 0.4.0 - [`main`]         |
//! | 0.13.* | 0.3.0                    |
//! | 0.12.* | 0.1.0 - 0.2.1            |
//!
//! [smoothing_off]: https://raw.githubusercontent.com/doonv/bevy_smooth_pixel_camera/main/assets/smoothing_off.avif
//! [smoothing_on]: https://raw.githubusercontent.com/doonv/bevy_smooth_pixel_camera/main/assets/smoothing_on.avif
//! [`how_smoothing_works`]: https://github.com/doonv/bevy_smooth_pixel_camera/blob/main/examples/how_smoothing_works.rs
//! [`main`]: https://github.com/doonv/bevy_smooth_pixel_camera
//!
//! [`default_nearest`]: ImagePlugin::default_nearest
//! [smoothing]: components::PixelCamera::smoothing

use bevy::prelude::*;

pub mod components;
pub mod prelude;
mod systems;
pub mod viewport;

/// A [`SystemSet`] for [`PixelCameraPlugin`]'s systems.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum CameraSystems {
    /// The system that updates the [`PixelCamera`](components::PixelCamera)'s position after every frame.
    UpdatePosition,
    /// Other systems that update the properties of the camera.
    Update,
}

/// A tiny offset applied to the [`PixelCamera`](components::PixelCamera)'s final position every frame, this prevents the GPU from getting confused on
/// certain pixel sizes and not rendering some pixels, causing artifacts.
///
/// This is exposed to you in the case it messes up the rendering of high resolution assets.
pub const CAMERA_POSITION_OFFSET: Vec2 = Vec2::splat(0.01);

/// Updates the [`PixelCamera`](components::PixelCamera), allowing for [smoothing](components::PixelCamera::smoothing) and viewport resizing with the window.
pub struct PixelCameraPlugin;
impl Plugin for PixelCameraPlugin {
    fn build(&self, app: &mut App) {
        use systems::*;

        app.add_systems(
            PostUpdate,
            (
                snap_camera_position
                    .in_set(CameraSystems::UpdatePosition)
                    .after(TransformSystems::Propagate),
                (
                    update_viewport_size,
                    sync_camera_fields,
                    update_high_resolution_viewport_size,
                )
                    .in_set(CameraSystems::Update),
            ),
        );
    }
}
