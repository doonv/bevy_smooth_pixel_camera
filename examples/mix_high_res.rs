//! Demonstrates how to combine high resolution assets with a pixelated world.
//!
//! This example also uses [`PanCamera`] for camera movement.

use bevy::camera::visibility::RenderLayers;
use bevy::camera_controller::pan_camera::{PanCamera, PanCameraPlugin};
use bevy::prelude::*;
use bevy_smooth_pixel_camera::prelude::*;

#[derive(Component)]
struct BevyIcon;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PixelCameraPlugin,
            PanCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        PixelCamera::from_size(ViewportScalingMode::PixelSize(32.0)),
        PanCamera {
            zoom_speed: 0.01,
            pan_speed: 20.0,
            ..default()
        },
    ));

    commands.spawn(Sprite::from_image(asset_server.load("checkerboard.png")));
    commands.spawn((
        Sprite::from_image(asset_server.load("bevy_pixel_dark.png")),
        Transform::from_xyz(0.0, 0.0, 1.0),
        BevyIcon,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("bevy_high_res_bird_dark.png")),
        // The viewport camera and image are on render layer 1 by default,
        // while the pixelated world is rendered on layer 0 by default.
        //
        // So, in order to render our high resolution sprite in high resolution,
        // we need to put it on the same layer as the viewport image,
        // to be rendered the viewport camera in high resolution.
        RenderLayers::layer(1),
        // High res images share coordinates with the pixelated world, so
        // we have to scale down this image to compensate for it's high intrinsic size.
        // (256x256) -> (8x8)
        Transform::from_scale(Vec3::splat(1.0 / 32.0)),
    ));
}

fn update(mut icon: Single<&mut Transform, With<BevyIcon>>, time: Res<Time>) {
    icon.translation.y = time.elapsed_secs().sin() * 4.5;
}
