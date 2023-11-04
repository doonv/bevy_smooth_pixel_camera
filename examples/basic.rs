use bevy::prelude::*;
use bevy_smooth_pixel_camera::prelude::*;

#[derive(Component)]
struct BevyIcon;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PixelCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), PixelCamera::from_scaling(32)));

    commands.spawn(SpriteBundle {
        texture: asset_server.load("checkerboard.png"),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("bevy_pixel_dark.png"),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        },
        BevyIcon,
    ));
}
fn update(
    mut camera: Query<&mut PixelCamera>,
    mut bevy: Query<&mut Transform, With<BevyIcon>>,
    time: Res<Time>,
) {
    let mut camera = camera.single_mut();

    camera.subpixel_pos.x = (time.elapsed_seconds() / 2.0).sin() * 10.0;
    info!("{:?}", camera.subpixel_pos);

    let mut bevy_transform = bevy.single_mut();

    bevy_transform.translation.y = time.elapsed_seconds().sin() * 4.5;
}
