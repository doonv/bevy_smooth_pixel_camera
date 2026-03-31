//! The components of [`bevy_smooth_pixel_camera`](crate).

use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureDimension, TextureFormat, TextureUsages};
use bevy::window::PrimaryWindow;
use smallvec::SmallVec;
use std::mem;

use crate::viewport::ViewportScalingMode;

/// The pixelated camera component. Adding this component makes this camera render
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
#[component(on_add, on_remove)]
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
impl PixelCamera {
    /// Creates a new pixel camera with the `viewport_size` of choice and default configuration.
    #[must_use]
    pub fn from_size(viewport_size: ViewportScalingMode) -> Self {
        Self {
            viewport_size,
            ..default()
        }
    }

    fn on_add(world: DeferredWorld, context: HookContext) {
        fn inner(mut world: DeferredWorld, context: HookContext) -> Result<()> {
            let entity = world.get_entity(context.entity)?;

            let (pixel_camera, camera, render_target) =
                entity.get_components::<(&PixelCamera, &Camera, &RenderTarget)>()?;

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
            let mut image = Image::new_fill(
                size,
                TextureDimension::D2,
                &[0; 4],
                TextureFormat::Bgra8UnormSrgb,
                RenderAssetUsages::default(),
            );
            image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;

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
                    clear_color: pixel_camera.viewport_size.clear_color().unwrap_or_default(),
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

    fn on_remove(world: DeferredWorld, context: HookContext) {
        fn inner(mut world: DeferredWorld, context: HookContext) -> Result<()> {
            let viewport_entities: SmallVec<[Entity; 2]> = world
                .get::<Children>(context.entity)
                .iter()
                .copied()
                .flatten()
                .copied()
                .filter(|&e| {
                    world.get::<ViewportCamera>(e).is_some()
                        || world.get::<ViewportImage>(e).is_some()
                })
                .collect::<SmallVec<[_; 2]>>();

            swap(&mut world, &context);
            for entity in viewport_entities {
                world.commands().entity(entity).despawn();
            }

            Ok(())
        }

        match inner(world, context) {
            Ok(()) => (),
            Err(err) => error!("While deinitializing PixelCamera {}: {err}", context.entity),
        }
    }
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

/// Marker component for a [`PixelCamera`]'s viewport camera.
///
/// Used when excluding the viewport camera when querying for cameras.
#[derive(Component, Debug)]
pub struct ViewportCamera;

/// Marker component for a [`PixelCamera`]'s viewport image.
#[derive(Component, Debug)]
pub struct ViewportImage;

/// Makes this [`PixelCamera`] high resolution, it will have the same scaling, but
/// the pixels will no longer be locked to the grid. This is useful for when you need to quickly
/// check how the game looks when not locked to the grid.
#[derive(Component, Debug)]
#[require(PixelCamera)]
#[component(on_add = swap_hook, on_remove = swap_hook)]
pub struct HighResolution;
fn swap_hook(mut world: DeferredWorld, context: HookContext) {
    swap(&mut world, &context);
}
fn swap(world: &mut DeferredWorld, context: &HookContext) {
    let world_camera_entity = context.entity;
    world
        .commands()
        .queue(move |world: &mut World| -> Result<()> {
            let children = world
                .get::<Children>(world_camera_entity)
                .ok_or("no children")?;
            let [mut world_camera, mut viewport_camera] =
                world.get_entity_mut([world_camera_entity, children[0]])?;
            let world_target = world_camera
                .get_mut::<RenderTarget>()
                .ok_or("no RenderTarget")?;
            let viewport_target = viewport_camera
                .get_mut::<RenderTarget>()
                .ok_or("no RenderTarget")?;
            mem::swap(world_target.into_inner(), viewport_target.into_inner());

            let world_proj = world_camera
                .get_mut::<Projection>()
                .ok_or("no Projection")?;
            let viewport_proj = viewport_camera
                .get_mut::<Projection>()
                .ok_or("no Projection")?;
            mem::swap(world_proj.into_inner(), viewport_proj.into_inner());

            Ok(())
        });
}

fn validate_layers(camera: EntityRef, pixel_camera: &PixelCamera) -> Result<(), &'static str> {
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
