//! Everything from every example, all at once.

use bevy::camera::visibility::RenderLayers;
use bevy::camera_controller::pan_camera::{PanCamera, PanCameraPlugin};
use bevy::color::palettes::css::LIME;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_smooth_pixel_camera::components::ViewportCamera;
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
        .insert_resource(SpritePickingSettings {
            // Setting BoundingBox is necessary for picking to work at the moment.
            // See https://github.com/bevyengine/bevy/issues/23750
            picking_mode: SpritePickingMode::BoundingBox,
            ..default()
        })
        .add_systems(Startup, (setup, change_viewport_camera_zoom).chain())
        .add_systems(
            Update,
            (
                update,
                draw_border,
                toggle_high_res.run_if(input_just_pressed(KeyCode::Space)),
                toggle_smoothing.run_if(input_just_pressed(KeyCode::KeyM)),
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
    commands
        .spawn((
            Sprite::from_image(asset_server.load("bevy_pixel_dark.png")),
            Transform::from_xyz(0.0, 0.0, 1.0),
            BevyIcon,
            Pickable::default(), // picking
        ))
        .observe(on_drag_rotate);

    // mix_high_res
    commands
        .spawn((
            Sprite::from_image(asset_server.load("bevy_high_res_bird_dark.png")),
            RenderLayers::layer(1),
            Transform::from_scale(Vec3::splat(1.0 / 32.0))
                .with_translation(Vec3::new(0.0, 0.0, 2.0)),
            Pickable::default(), // picking
        ))
        .observe(on_drag_rotate);

    commands.spawn((
        Text::new("Press 'Space' to toggle HighResolution "),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
        children![
            (TextSpan::new("on"), HighResolutionStatus),
            TextSpan::new(".\nPress 'M' to toggle smoothing.")
        ],
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

fn toggle_smoothing(mut pixel_camera: Single<&mut PixelCamera>) {
    pixel_camera.smoothing = !pixel_camera.smoothing;
}

fn change_viewport_camera_zoom(
    viewport_camera: Single<(&mut Projection, &mut Camera), With<ViewportCamera>>,
    mut config: ResMut<GizmoConfigStore>,
) {
    config
        .config_mut::<DefaultGizmoConfigGroup>()
        .0
        .render_layers = RenderLayers::layer(1);

    let (mut projection, mut camera) = viewport_camera.into_inner();

    camera.clear_color = ClearColorConfig::Custom(Color::BLACK);

    if let Projection::Orthographic(projection) = projection.as_mut() {
        projection.scale = 1.2;
    }
}

fn draw_border(
    window: Single<&Window>,
    mut gizmos: Gizmos,
    camera: Single<&Transform, With<PixelCamera>>,
) {
    gizmos.rect_2d(
        Isometry2d::new(
            camera.translation.xy() + Vec2::splat(2.0 / 32.0),
            Rot2::radians(camera.rotation.to_euler(EulerRot::XYZ).2),
        ),
        (window.physical_size().as_vec2() / 32.0).ceil() * camera.scale.xy()
            + Vec2::splat(2.0 / 32.0),
        LIME,
    );
}

fn update(mut icon: Single<&mut Transform, With<BevyIcon>>, time: Res<Time>) {
    icon.translation.y = time.elapsed_secs().sin() * 4.5;
}

fn on_drag_rotate(drag: On<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
    if let Ok(mut transform) = transforms.get_mut(drag.entity) {
        transform.rotate_y(drag.delta.x * 0.10);
        transform.rotate_x(drag.delta.y * 0.10);
    }
}
