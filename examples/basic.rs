//! A simple example of this crate having smooth camera movement while maintaining pixel perfection.

use bevy::prelude::*;
use bevy_smooth_pixel_camera::prelude::*;

/// Marker component for the bevy icon so we can move it in `update`
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
    // Spawn a 2d camera with the PixelCamera bundle in order to
    // turn it into a smooth pixel perfect camera.
    commands.spawn((
        Camera2d,
        Msaa::Off,
        PixelCamera::from_size(ViewportSize::PixelFixed(32)),
    ));

    // Spawn a checkerboard background
    commands.spawn((
        Sprite::from_image(asset_server.load("checkerboard.png")),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    // Spawn a bevy icon sprite and mark it with the `BevyIcon` component
    commands.spawn((
        Sprite::from_image(asset_server.load("bevy_pixel_dark.png")),
        Transform::from_xyz(0.0, 0.0, 1.0),
        BevyIcon,
    ));
}

fn update(
    mut camera: Query<&mut PixelCamera>,
    mut bevy: Query<&mut Transform, With<BevyIcon>>,
    time: Res<Time>,
) {
    // Get the camera and move it horizontally over time
    let mut camera = camera.single_mut();

    camera.subpixel_pos.x = (time.elapsed_secs() / 2.0).sin() * 10.0;

    // Get the bevy icon and move it vertically over time
    let mut bevy_transform = bevy.single_mut();

    bevy_transform.translation.y = time.elapsed_secs().sin() * 4.5;
}
