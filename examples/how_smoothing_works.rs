//! Demonstrates how [`bevy_smooth_pixel_camera`]'s camera smoothing works behind the scenes.

use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::basic::LIME;
use bevy::prelude::*;
use bevy_smooth_pixel_camera::components::ViewportCamera;
use bevy_smooth_pixel_camera::prelude::*;

#[derive(Component)]
struct BevyIcon;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PixelCameraPlugin,
        ))
        .add_systems(Startup, (setup, change_viewport_camera_zoom).chain())
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(PixelCamera::from_size(ViewportScalingMode::PixelSize(32.0)));

    commands.spawn(Sprite::from_image(asset_server.load("checkerboard.png")));

    commands.spawn((
        Sprite::from_image(asset_server.load("bevy_pixel_dark.png")),
        Transform::from_xyz(0.0, 0.0, 1.0),
        BevyIcon,
    ));
}

/// This whole function is kinda jank, as we are intentionally breaking the plugin's
/// logic to give you a look behind the scenes.
fn change_viewport_camera_zoom(
    viewport_camera: Single<(&mut Projection, &mut Camera), With<ViewportCamera>>,
    mut config: ResMut<GizmoConfigStore>,
) {
    // Make gizmos appear in high resolution
    config
        .config_mut::<DefaultGizmoConfigGroup>()
        .0
        .render_layers = RenderLayers::layer(1);

    let (mut projection, mut camera) = viewport_camera.into_inner();

    // Change the clear color to pure black for contrast.
    camera.clear_color = ClearColorConfig::Custom(Color::BLACK);

    // The `scale` field isn't actually touched by `bevy_smooth_pixel_camera`, only the `scaling_mode` is modified,
    // so this isn't immediately overridden by the plugin.
    if let Projection::Orthographic(projection) = projection.as_mut() {
        projection.scale = 1.2;
    }
}

fn update(
    mut camera: Single<&mut Transform, (With<PixelCamera>, Without<BevyIcon>)>,
    mut icon: Single<&mut Transform, (With<BevyIcon>, Without<PixelCamera>)>,
    time: Res<Time>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
) {
    // Render a rectangle representing what would usually be in the viewport
    gizmos.rect_2d(
        Isometry2d::from_translation(camera.translation.xy() + Vec2::splat(2.0 / 32.0)),
        (window.physical_size().as_vec2() / 32.0).ceil() + Vec2::splat(2.0 / 32.0),
        LIME,
    );

    camera.translation.x = (time.elapsed_secs() / 2.0).sin() * 10.0;

    icon.translation.y = time.elapsed_secs().sin() * 4.5;
}
