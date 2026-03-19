//! Demonstrates how to combine high resolution assets with a pixelated world.

use bevy::camera::visibility::RenderLayers;
use bevy::camera_controller::pan_camera::{PanCamera, PanCameraPlugin};
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
            PanCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a 2d camera with the PixelCamera bundle in order to
    // turn it into a smooth pixel perfect camera.
    commands.spawn((
        PixelCamera::from_size(ViewportScalingMode::PixelSize(32.0)),
        PanCamera {
            zoom_speed: 0.01,
            pan_speed: 20.0,
            ..default()
        },
    ));

    // Spawn a checkerboard background
    commands.spawn(Sprite::from_image(asset_server.load("checkerboard.png")));
    // Spawn a bevy icon sprite and mark it with the `BevyIcon` component
    commands.spawn((
        Sprite::from_image(asset_server.load("bevy_pixel_dark.png")),
        Transform::from_xyz(0.0, 0.0, 1.0),
        BevyIcon,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("bevy_high_res_bird_dark.png")),
        // The viewport camera and image are on render layer 1 with the default configuration,
        // while the pixelated world is rendered on layer 0 by default.
        //
        // So, in order to render our high resolution sprite in high res, we need to put it on the same layer as the viewport image,
        // to be rendered the viewport camera in high resolution.
        RenderLayers::layer(1),
        // High res images share coordinates with the pixelated world, so
        // we have to scale down this image to compensate.
        Transform::from_scale(Vec3::splat(1.0 / 32.0)),
    ));
}

fn update(
    mut icon: Single<&mut Transform, (With<BevyIcon>, Without<PixelCamera>)>,
    time: Res<Time>,
) {
    icon.translation.y = time.elapsed_secs().sin() * 4.5;
}
