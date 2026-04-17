//! Demonstrates how to toggle the [`HighResolution`] component.
//!
//! This example also uses [`PanCamera`] for camera movement.

use bevy::camera_controller::pan_camera::{PanCamera, PanCameraPlugin};
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_smooth_pixel_camera::prelude::*;

#[derive(Component)]
struct BevyIcon;

#[derive(Component)]
struct HighResolutionStatus;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PanCameraPlugin,
            PixelCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update,
                toggle_high_res.run_if(input_just_pressed(KeyCode::Space)),
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        PixelCamera::from_size(ViewportScalingMode::PixelSize(32.0)),
        PanCamera {
            zoom_speed: 0.01,
            pan_speed: 15.0,
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
        Text::new("Press 'Space' to toggle HighResolution "),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
        children![(TextSpan::new("on"), HighResolutionStatus)],
    ));
}

fn toggle_high_res(
    pixel_camera: Single<(Entity, Has<HighResolution>), With<PixelCamera>>,
    mut commands: Commands,
    mut text: Single<&mut TextSpan, With<HighResolutionStatus>>,
) {
    let (pixel_camera, is_high_res) = *pixel_camera;
    if is_high_res {
        commands.entity(pixel_camera).remove::<HighResolution>();
        text.replace_range(.., "on");
    } else {
        commands.entity(pixel_camera).insert(HighResolution);
        text.replace_range(.., "off");
    }
}

fn update(mut icon: Single<&mut Transform, With<BevyIcon>>, time: Res<Time>) {
    icon.translation.y = time.elapsed_secs().sin() * 4.5;
}
