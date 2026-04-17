//! Demonstrates usage of [picking](bevy::picking) with [`bevy_smooth_pixel_camera`]
//!
//! This is a simple scene with a bevy logo you can rotate by dragging it.

use bevy::prelude::*;
use bevy_smooth_pixel_camera::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PixelCameraPlugin,
        ))
        .insert_resource(SpritePickingSettings {
            // Setting BoundingBox is necessary for picking to work at the moment.
            // See https://github.com/bevyengine/bevy/issues/23750
            picking_mode: SpritePickingMode::BoundingBox,
            ..default()
        })
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(PixelCamera::from_size(ViewportScalingMode::PixelSize(32.0)));

    commands.spawn(Sprite::from_image(asset_server.load("checkerboard.png")));

    commands
        .spawn((
            Sprite::from_image(asset_server.load("bevy_pixel_dark.png")),
            Pickable::default(),
        ))
        .observe(on_drag_rotate);
}

fn on_drag_rotate(drag: On<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
    if let Ok(mut transform) = transforms.get_mut(drag.entity) {
        transform.rotate_y(drag.delta.x * 0.2);
        transform.rotate_x(drag.delta.y * 0.2);
    }
}
