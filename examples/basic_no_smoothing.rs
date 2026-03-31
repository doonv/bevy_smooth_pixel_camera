//! Demonstrates simple usage of `bevy_smooth_pixel_camera`, without camera smoothing.

use bevy::prelude::*;
use bevy_smooth_pixel_camera::prelude::*;

/// Marker component for the bevy icon so we can move it in [`update`]
#[derive(Component)]
struct BevyIcon;

fn main() {
    App::new()
        .add_plugins((
            // Set the ImagePlugin to have nearest neighbor sampling
            // This prevents our sprites from becoming blurry
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            // Add the smooth pixel camera plugin
            PixelCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(PixelCamera {
        viewport_size: ViewportScalingMode::PixelSize(32.0),
        smoothing: false,
        ..default()
    });

    commands.spawn(Sprite::from_image(asset_server.load("checkerboard.png")));

    commands.spawn((
        Sprite::from_image(asset_server.load("bevy_pixel_dark.png")),
        Transform::from_xyz(0.0, 0.0, 1.0),
        BevyIcon,
    ));
}

/// Moves the camera and icon over time to show how movement in the world is pixelated but movement of the camera is not.
fn update(
    // Make sure to use PixelCamera and not Camera, as Camera can return the viewport camera.
    // If you want to exclude the viewport camera from a query, you can use Without<ViewportCamera>
    mut camera: Single<&mut Transform, (With<PixelCamera>, Without<BevyIcon>)>,
    mut icon: Single<&mut Transform, (With<BevyIcon>, Without<PixelCamera>)>,
    time: Res<Time>,
) {
    camera.translation.x = (time.elapsed_secs() / 2.0).sin() * 10.0;

    icon.translation.y = time.elapsed_secs().sin() * 4.5;
}
