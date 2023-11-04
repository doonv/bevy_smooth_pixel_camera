use bevy::{
    prelude::*,
    render::{camera::RenderTarget, render_resource::*, view::RenderLayers},
    window::WindowResolution,
};

/// The pixelated camera component.
///
/// Add this component to a [`Camera2dBundle`] in order to turn it into a
/// pixelated camera.
///
/// **Warning:** In order to move the camera please use the `subpixel_pos`
/// attribute instead of the [`Transform`] component (the transform is a rounded version of subpixel_pos)
#[derive(Component)]
pub struct PixelCamera {
    /// The level of upscaling to use for pixels.
    ///
    /// For example: A scaling of `4` which cause every world pixel to be 4x4 in size on the screen.
    pub scaling: u8,
    /// The subpixel position of the [`PixelCamera`], use this instead of the camera's [`Transform`].
    pub subpixel_pos: Vec2,
    /// The order in which the viewport camera renders.
    /// Cameras with a higher order are rendered later, and thus on top of lower order cameras.
    ///
    /// Because we want the world camera to render before the viewport camera, set this value to a positive number.
    pub viewport_order: isize,
    /// The rendering layer the viewport is on.
    pub viewport_layer: RenderLayers,
}

#[derive(Component)]
struct PixelViewport(Entity);
#[derive(Component)]
struct PixelViewportMarker;

impl Default for PixelCamera {
    fn default() -> Self {
        Self {
            viewport_order: 1,
            scaling: 2,
            viewport_layer: RenderLayers::layer(1),
            subpixel_pos: Vec2::ZERO,
        }
    }
}

impl PixelCamera {
    /// Creates a new pixel camera with the `scaling` of choice.
    pub fn from_scaling(scaling: u8) -> Self {
        Self {
            scaling,
            ..default()
        }
    }
}

/// The [`PixelCameraPlugin`] handles initialization and updates of the [`PixelCamera`].
///
/// It also disables [`Msaa`].
pub struct PixelCameraPlugin;
impl Plugin for PixelCameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa::Off)
            .add_systems(Update, (init_camera, update_viewport_size, smooth_camera));
    }
}

fn get_viewport_size(window_resolution: &WindowResolution, scaling: u8) -> Extent3d {
    Extent3d {
        width: (window_resolution.width() / scaling as f32).ceil() as u32 + 2,
        height: (window_resolution.height() / scaling as f32).ceil() as u32 + 2,
        depth_or_array_layers: 1,
    }
}

fn init_camera(
    mut query: Query<
        (&PixelCamera, &mut Camera, Option<&RenderLayers>, Entity),
        Added<PixelCamera>,
    >,
    window_query: Query<&Window>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    let window = window_query.single();

    for (
        PixelCamera {
            viewport_order,
            scaling,
            viewport_layer,
            ..
        },
        mut camera,
        world_layer,
        entity,
    ) in query.iter_mut()
    {
        if let Some(world_layer) = world_layer {
            if world_layer.intersects(viewport_layer) {
                error!("The render layers of the world intersect with the render layers of the viewport camera");
                return;
            }
        } else if viewport_layer.intersects(&RenderLayers::layer(0)) {
            error!("The render layers of the viewport camera intersect with the default render layer of the world");
            return;
        } else if *viewport_layer == RenderLayers::none() {
            error!("The viewport camera has no render layers and will be rendered on the world");
            return;
        }

        if &camera.order >= viewport_order {
            error!("The camera is configured to render later or at the same time as of the viewport camera. (camera.order >= viewport_camera.order)");
            return;
        }

        let size = get_viewport_size(&window.resolution, *scaling);

        // This is the texture that will be rendered to.
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };

        // fill image.data with zeroes
        image.resize(size);

        let image_handle = images.add(image);

        camera.target = RenderTarget::Image(image_handle.clone());

        let viewport_entity = commands
            .spawn((
                SpriteBundle {
                    texture: image_handle.clone(),
                    transform: Transform::from_scale(Vec2::splat(*scaling as f32).extend(1.0)),
                    ..default()
                },
                *viewport_layer,
                PixelViewportMarker,
            ))
            .id();

        commands.spawn((
            Camera2dBundle {
                camera: Camera {
                    order: *viewport_order,
                    ..default()
                },
                ..default()
            },
            *viewport_layer,
        ));

        commands
            .entity(entity)
            .insert(PixelViewport(viewport_entity));
    }
}

fn update_viewport_size(
    mut query: Query<(&PixelCamera, &mut Camera)>,
    window_query: Query<&Window, Changed<Window>>,
    mut images: ResMut<Assets<Image>>,
) {
    let window = if let Ok(window) = window_query.get_single() {
        window
    } else {
        return;
    };

    for (PixelCamera { scaling, .. }, mut camera) in query.iter_mut() {
        if let RenderTarget::Image(image_handle) = &mut camera.target {
            let image = images.get_mut(image_handle);

            if let Some(image) = image {
                let new_size = get_viewport_size(&window.resolution, *scaling);

                image.resize(new_size);
            } else {
                error!("Pixel camera render target image doesn't exist!");
            }
        }
    }
}

fn smooth_camera(
    mut query: Query<(&PixelCamera, &mut Transform, &PixelViewport)>,
    mut viewports: Query<&mut Transform, (With<PixelViewportMarker>, Without<PixelViewport>)>,
) {
    for (
        PixelCamera {
            scaling,
            subpixel_pos,
            ..
        },
        mut camera_transform,
        viewport,
    ) in query.iter_mut()
    {
        let mut viewport_transform = viewports.get_mut(viewport.0).unwrap();
        let scaling_f32 = *scaling as f32;

        // Set the camera transform the rounded down version of the subpixel position
        camera_transform.translation.x = subpixel_pos.x.floor();
        camera_transform.translation.y = subpixel_pos.y.floor();

        // In order to get smooth camera movement while retaining pixel perfection,
        // we can move the viewport's transform by the remainder of the subpixel.
        //
        // The smoothing is based on this video: https://youtu.be/jguyR4yJb1M?t=98
        let remainder_x = subpixel_pos.x % 1.;
        let remainder_y = subpixel_pos.y % 1.;

        viewport_transform.translation.x = -remainder_x * scaling_f32;
        viewport_transform.translation.y = -remainder_y * scaling_f32;
    }
}
