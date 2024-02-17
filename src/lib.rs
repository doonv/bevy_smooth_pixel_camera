#![doc = include_str!("../README.md")]

use bevy::prelude::*;

pub mod components;
pub mod prelude;
mod systems;
pub mod viewport;

/// A [`SystemSet`] for [`PixelCameraPlugin`]'s systems.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum CameraSystems {
    /// The systems that initialize the [`PixelCamera`](components::PixelCamera)
    /// component when it's added to an entity.
    Initialization,
    /// The systems that update the pixel camera's position after every frame.
    Update,
}

/// The [`PixelCameraPlugin`] handles initialization and updates of the [`PixelCamera`](components::PixelCamera).
///
/// It also disables [`Msaa`].
pub struct PixelCameraPlugin;
impl Plugin for PixelCameraPlugin {
    fn build(&self, app: &mut App) {
        use systems::*;

        app.insert_resource(Msaa::Off).add_systems(
            PostUpdate,
            (
                init_camera.in_set(CameraSystems::Initialization),
                (update_viewport_size, smooth_camera, set_camera_position)
                    .in_set(CameraSystems::Update),
            ),
        );
    }
}
