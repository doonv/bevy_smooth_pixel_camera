#![doc = include_str!("../README.md")]

use bevy::prelude::*;

pub mod components;
pub mod prelude;
mod systems;
pub mod viewport;

/// The [`PixelCameraPlugin`] handles initialization and updates of the [`PixelCamera`](components::PixelCamera).
///
/// It also disables [`Msaa`].
pub struct PixelCameraPlugin;
impl Plugin for PixelCameraPlugin {
    fn build(&self, app: &mut App) {
        use crate::systems::*;

        app.insert_resource(Msaa::Off).add_systems(
            Update,
            (
                init_camera,
                update_viewport_size,
                smooth_camera,
                set_camera_position,
                fix_11240,
            ),
        );
    }
}
