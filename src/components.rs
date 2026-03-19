//! The components of [`bevy_smooth_pixel_camera`](crate).

use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::render::render_resource::{
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::window::PrimaryWindow;

use crate::viewport::ViewportScalingMode;

/// A pixelated camera component. Adding this component makes this camera render
/// to a small viewport before rendering to the window (or whatever its [`RenderTarget`] was set to).
///
/// ## Implementation details
///
/// Upon adding this component to an entity, 2 children will be added to it, a [`ViewportCamera`] and a [`ViewportImage`],
/// this camera's [`RenderTarget`] will be redirected to the [`ViewportImage`]. Depending on the configured [`viewport_size`],
/// the viewport would be some amount smaller than the window size, which creates the pixelated effect. The [`ViewportCamera`]
/// will point at the small image with zoom and scale it up to the size of the whole window.
///
/// The [`ViewportCamera`] and image are on their own separate [render layer](RenderLayers), (configured with [`viewport_layers`])
/// so that they don't interfere with the world. Any entities added to this layer will be rendered at their full resolution instead
/// of being pixelated.
///
/// [`viewport_size`]: Self::viewport_size
/// [`viewport_layers`]: Self::viewport_layers
#[derive(Component)]
#[require(Camera2d, Msaa::Off)]
#[component(on_add = init_camera)]
pub struct PixelCamera {
    /// The size of the viewport.
    pub viewport_size: ViewportScalingMode,
    /// The order in which the viewport camera renders.
    /// Cameras with a higher order are rendered later, and thus on top of lower order cameras.
    ///
    /// Because we want the world camera to render before the viewport camera,
    /// set this value to a number higher the than the world camera's order.
    pub viewport_order: isize,
    /// The rendering layer the viewport is on.
    ///
    /// Any entities on this layer will be rendered in high resolution instead of pixelated.
    pub viewport_layers: RenderLayers,
    /// Whether camera position smoothing is enabled for this camera.
    pub smoothing: bool,
}

fn init_camera(world: DeferredWorld, context: HookContext) {
    fn inner(mut world: DeferredWorld, context: HookContext) -> Result<()> {
        let entity = world
            .get_entity(context.entity)?;

        let (pixel_camera, camera, render_target) = entity.get_components::<(&PixelCamera, &Camera, &RenderTarget)>()?;

        validate_layers(entity, pixel_camera)?;

        if camera.order >= pixel_camera.viewport_order {
            return Err("The camera is configured to render later or at the same time as of the viewport camera. (camera.order >= viewport_camera.order)".into());
        }

        let primary = world
            .try_query_filtered::<Entity, With<PrimaryWindow>>()
            .expect("all components should be registered into the world")
            .single(&world);

        let mut windows = world
            .try_query_filtered::<&Window, ()>()
            .expect("all components should be registered into the world");
        let window_size = match render_target {
            RenderTarget::Window(window_ref) => {
                match window_ref
                    .normalize(primary.ok())
                    .and_then(|w| windows.get(&world, w.entity()).ok())
                {
                    Some(window) => Vec2::new(window.width(), window.height()),
                    None => {
                        return Err("".into());
                        // return error!(
                        //     "{}'s RenderTarget::Window points to a window that doesn't exist.",
                        //     context.entity
                        // );
                    }
                }
            }
            RenderTarget::None { size } => size.as_vec2(),
            target => {
                return Err(format!("Render target {target:?} is not supported.").into());
            }
        };
        let (size, scaling_mode) = pixel_camera
            .viewport_size
            .get_configuration(window_size, pixel_camera.smoothing);

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

        let render_target = render_target.clone();

        let image_handle = world
            .get_resource_mut::<Assets<Image>>()
            .expect("resource Assets<Image> should exist")
            .add(image);
        let pixel_camera = world
            .get::<PixelCamera>(context.entity)
            .expect("PixelCamera must exist in it's own on_add hook");

        let viewport_camera = (
            Camera {
                order: pixel_camera.viewport_order,
                clear_color: pixel_camera.viewport_size.clear_color(),
                ..default()
            },
            Projection::Orthographic(OrthographicProjection {
                scaling_mode,
                ..OrthographicProjection::default_2d()
            }),
            Camera2d,
            ViewportCamera,
            render_target,
            pixel_camera.viewport_layers.clone(),
        );
        let viewport_image = (
            Sprite::from_image(image_handle.clone()),
            Transform::from_xyz(0.0, 0.0, -5.0),
            pixel_camera.viewport_layers.clone(),
            ViewportImage,
        );
        world
            .commands()
            .entity(context.entity)
            .insert(RenderTarget::from(image_handle))
            .with_child(viewport_camera)
            .with_child(viewport_image);

        Ok(())
    }
    
    match inner(world, context) {
        Ok(()) => (),
        Err(err) => error!("While initializing PixelCamera {}: {err}", context.entity),
    }
}
pub(crate) fn validate_layers(
    camera: EntityRef<'_>,
    pixel_camera: &PixelCamera,
) -> Result<(), &'static str> {
    if let Some(world_layers) = camera.get::<RenderLayers>() {
        if world_layers.intersects(&pixel_camera.viewport_layers) {
            return Err(
                "The render layers of the world intersect with the render layers of the viewport camera",
            );
        }
    } else if pixel_camera
        .viewport_layers
        .intersects(&RenderLayers::default())
    {
        return Err(
            "The render layers of the viewport camera intersect with the default render layer of the world",
        );
    } else if pixel_camera.viewport_layers == RenderLayers::none() {
        return Err("The viewport camera has no render layers and will not be rendered");
    }

    Ok(())
}

impl Default for PixelCamera {
    fn default() -> Self {
        Self {
            viewport_size: ViewportScalingMode::default(),
            viewport_order: 1,
            viewport_layers: RenderLayers::layer(1),
            smoothing: true,
        }
    }
}

impl PixelCamera {
    /// Creates a new pixel camera with the `size` of choice and default configuration.
    pub fn from_size(viewport_size: ViewportScalingMode) -> Self {
        Self {
            viewport_size,
            ..default()
        }
    }
}

/// Marker component for a [`PixelCamera`]'s viewport camera.
///
/// Used when excluding the viewport camera when querying for cameras.
#[derive(Component)]
pub struct ViewportCamera;

/// Marker component for a [`PixelCamera`]'s viewport image.
#[derive(Component)]
pub struct ViewportImage;
