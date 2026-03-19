//! A bevy plugin that adds a simple smooth pixel camera.
//!
//! This method allows for smooth camera movement while retaining the pixel perfection of low resolution rendering.
//!
//! ## Usage
//!
//! 1. Add the `bevy_smooth_pixel_camera` crate to your project.
//!
//!     ```sh
//!     cargo add bevy_smooth_pixel_camera
//!     ```
//!
//! 2. Add the [`PixelCameraPlugin`] and set the [`ImagePlugin`] to [`default_nearest`](ImagePlugin::default_nearest).
//!
//!     ```
//!     use bevy::prelude::*;
//!     use bevy_smooth_pixel_camera::prelude::*;
//!     
//!     App::new().add_plugins((
//!         DefaultPlugins.set(ImagePlugin::default_nearest()),
//!         PixelCameraPlugin
//!     ));
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
//! | 0.18.* | 0.4.0 - main             |
//! | 0.13.* | 0.3.0                    |
//! | 0.12.* | 0.1.0 - 0.2.1            |

use bevy::prelude::*;

pub mod components;
pub mod prelude;
mod systems;
pub mod viewport;

/// A [`SystemSet`] for [`PixelCameraPlugin`]'s systems.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum CameraSystems {
    /// The system that update the pixel camera's position after every frame.
    Update,
}

/// Updates the [`PixelCamera`](components::PixelCamera), enabling [smoothing](components::PixelCamera::smoothing) and viewport resizing with the window.
pub struct PixelCameraPlugin;
impl Plugin for PixelCameraPlugin {
    fn build(&self, app: &mut App) {
        use systems::*;

        app.add_systems(
            PostUpdate,
            ((
                snap_camera_position,
                update_viewport_size,
                sync_camera_fields,
            )
                .in_set(CameraSystems::Update)
                .after(TransformSystems::Propagate),),
        );
    }
}
