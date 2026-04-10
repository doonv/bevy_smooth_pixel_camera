//! The components of [`bevy_smooth_pixel_camera`](crate).

use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::window::PrimaryWindow;
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
        fn inner(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) -> Result<()> {
            let this = world.get_entity(entity)?;

            let (pixel_camera, camera, render_target) =
                this.get_components::<(&PixelCamera, &Camera, &RenderTarget)>()?;

            let camera_layers = this.get::<RenderLayers>();
            if let Err(e) = validate_layers(camera_layers, &pixel_camera.viewport_layers) {
                error!(
                    r#"While validating RenderLayers for PixelCamera {entity}: {e}
    RenderLayers: {camera_layers:?}
    PixelCamera::viewport_layers: {:?}"#,
                    &pixel_camera.viewport_layers
                );
            }

            if camera.order >= pixel_camera.viewport_order {
                return Err("The camera is configured to render later or at the same time as of the viewport camera. (camera.order >= viewport_camera.order)".into());
            }
            let window_size = match render_target {
                RenderTarget::Window(window_ref) => {
                    let primary = world
                        .try_query_filtered::<Entity, With<PrimaryWindow>>()
                        .expect("all components should be registered into the world")
                        .single(&world);

                    let mut windows = world
                        .try_query_filtered::<&Window, ()>()
                        .expect("all components should be registered into the world");
                    let window = window_ref
                        .normalize(primary.ok())
                        .and_then(|w| windows.get(&world, w.entity()).ok())
                        .ok_or("RenderTarget::Window points to a window that doesn't exist.")?;

                    Vec2::new(window.width(), window.height())
                }
                RenderTarget::None { size } => size.as_vec2(),
                target => Err(format!("Render target {target:?} is not supported."))?,
            };
            let (size, scaling_mode) = pixel_camera
                .viewport_size
                .get_configuration(window_size, pixel_camera.smoothing);

            // This is the texture that will be rendered to.
            let mut viewport_image = Image::new_target_texture(
                size.width,
                size.height,
                TextureFormat::Rgba8UnormSrgb,
                None,
            );
            viewport_image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
            viewport_image.asset_usage = RenderAssetUsages::RENDER_WORLD;

            let render_target = render_target.clone();

            let viewport_image_handle = world
                .get_resource_mut::<Assets<Image>>()
                .ok_or("resource Assets<Image> should exist, bevy_smooth_pixel_camera expects AssetPlugin to be present")?
                .add(viewport_image);
            let pixel_camera = world
                .get::<PixelCamera>(entity)
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
                render_target,
                pixel_camera.viewport_layers.clone(),
                ViewportCamera,
            );
            let viewport_sprite = (
                Sprite::from_image(viewport_image_handle.clone()),
                pixel_camera.viewport_layers.clone(),
                ViewportImage,
            );
            let mut viewport_camera_id = Entity::PLACEHOLDER;
            let mut viewport_sprite_id = Entity::PLACEHOLDER;
            world
                .commands()
                .entity(entity)
                .insert(RenderTarget::from(viewport_image_handle))
                .with_children(|cmd| {
                    viewport_camera_id = cmd.spawn(viewport_camera).id();
                    viewport_sprite_id = cmd.spawn(viewport_sprite).id();
                })
                .insert(ViewportEntities {
                    camera: viewport_camera_id,
                    sprite: viewport_sprite_id,
                });

            Ok(())
        }

        match inner(world, context) {
            Ok(()) => (),
            Err(err) => {
                if cfg!(test) {
                    panic!("PixelCamera::on_add: {:?}", err);
                } else {
                    error!("While deinitializing PixelCamera {}: {err}", context.entity)
                }
            }
        }
    }

    fn on_remove(world: DeferredWorld, context: HookContext) {
        fn inner(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) -> Result<()> {
            let &ViewportEntities {
                camera,
                sprite: image,
            } = world
                .get(entity)
                .ok_or("PixelCameraViewportEntities should exist")?;

            // Swap RenderTarget
            let world_camera_entity = entity;
            world
                .commands()
                .queue(move |world: &mut World| -> Result<()> {
                    let viewport_entities = world
                        .get::<ViewportEntities>(entity)
                        .ok_or("PixelCameraViewportEntities should exist")?;
                    let [mut world_camera, mut viewport_camera] =
                        world.get_entity_mut([world_camera_entity, viewport_entities.camera])?;
                    let world_target = world_camera
                        .get_mut::<RenderTarget>()
                        .ok_or("no RenderTarget")?;
                    let viewport_target = viewport_camera
                        .get_mut::<RenderTarget>()
                        .ok_or("no RenderTarget")?;
                    mem::swap(world_target.into_inner(), viewport_target.into_inner());

                    Ok(())
                });

            world.commands().entity(camera).despawn();
            world.commands().entity(image).despawn();

            world.commands().entity(entity).remove::<ViewportEntities>();

            Ok(())
        }

        match inner(world, context) {
            Ok(()) => (),
            Err(err) => {
                if cfg!(test) {
                    panic!("PixelCamera::on_remove: {:?}", err);
                } else {
                    error!("While deinitializing PixelCamera {}: {err}", context.entity)
                }
            }
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
#[component(on_add = Self::swap, on_remove = Self::swap)]
pub struct HighResolution;
impl HighResolution {
    fn swap(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        world
            .commands()
            .queue(move |world: &mut World| -> Result<()> {
                let &ViewportEntities { camera, .. } = world
                    .get(entity)
                    .ok_or("PixelCameraViewportEntities should exist")?;
                let [mut world_camera, mut viewport_camera] =
                    world.get_entity_mut([entity, camera])?;
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
}

// Maybe replace with https://github.com/bevyengine/bevy/issues/21086 when added?
// But even then, Translation propagation is important.
#[derive(Component)]
pub(crate) struct ViewportEntities {
    pub(crate) camera: Entity,
    pub(crate) sprite: Entity,
}

fn validate_layers(
    world_layers: Option<&RenderLayers>,
    viewport_layers: &RenderLayers,
) -> Result<(), &'static str> {
    if let Some(world_layers) = world_layers {
        if world_layers.intersects(viewport_layers) {
            return Err(
                "The render layers of the world (PixelCamera) intersect with the render layers of the viewport camera.",
            );
        }
    } else if viewport_layers.intersects(&RenderLayers::default()) {
        return Err(
            "The render layers of the viewport camera intersect with the default render layer of the world.",
        );
    } else if viewport_layers == &RenderLayers::none() {
        return Err("The viewport camera has no render layers and will not be rendered.");
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::assert_matches;

    #[test]
    fn test_pixel_camera_hooks_components() {
        let mut app = test_app();
        let world = app.world_mut();

        let target = RenderTarget::None {
            size: UVec2 { x: 84, y: 905 },
        };
        let projection = Projection::Orthographic(OrthographicProjection {
            far: -421.6,
            ..OrthographicProjection::default_3d()
        });
        let mut camera = world.spawn((target, projection, PixelCamera::default()));

        assert_eq!(camera.get::<Children>().unwrap().len(), 2);
        assert!(camera.get::<ViewportEntities>().is_some());
        assert!(!matches!(
            camera.get::<RenderTarget>(),
            Some(RenderTarget::None {
                size: UVec2 { x: 84, y: 905 }
            })
        ));
        assert_matches!(
            camera.get::<Projection>(),
            Some(Projection::Orthographic(OrthographicProjection {
                far: -421.6,
                ..
            }))
        );
        camera.remove::<PixelCamera>();

        assert_matches!(
            camera.get::<RenderTarget>(),
            Some(RenderTarget::None {
                size: UVec2 { x: 84, y: 905 }
            })
        );
        assert_matches!(
            camera.get::<Projection>(),
            Some(Projection::Orthographic(OrthographicProjection {
                far: -421.6,
                ..
            }))
        );
        assert!(camera.get::<PixelCamera>().is_none());
        assert!(camera.get::<ViewportEntities>().is_none());
        assert!(camera.get::<Children>().is_none());
    }

    #[test]
    fn test_pixel_camera_hooks_children() {
        let mut app = test_app();
        let world = app.world_mut();

        let mut camera = world.spawn(children![Name::new("test")]);
        camera.insert(PixelCamera::default());

        assert_eq!(camera.get::<Children>().unwrap().len(), 3);

        camera.remove::<PixelCamera>();

        let children = camera.get::<Children>().unwrap();
        assert_eq!(children.len(), 1);
        let child = children[0];
        assert_eq!(world.get::<Name>(child).unwrap().as_str(), "test");
    }

    #[test]
    fn test_high_resolution() {
        let mut app = test_app();
        let world = app.world_mut();

        let target = RenderTarget::None {
            size: UVec2 { x: 831, y: 124 },
        };
        let projection = Projection::Orthographic(OrthographicProjection {
            far: 54.1e12,
            ..OrthographicProjection::default_3d()
        });
        let mut camera = world.spawn((target, projection, children![Name::new("test")]));

        assert_eq!(camera.get::<Children>().unwrap().len(), 1);

        camera.insert(PixelCamera::default());

        assert_eq!(camera.get::<Children>().unwrap().len(), 3);

        assert!(!matches!(
            camera.get::<RenderTarget>(),
            Some(RenderTarget::None {
                size: UVec2 { x: 831, y: 124 }
            })
        ));
        assert_matches!(
            camera.get::<Projection>(),
            Some(Projection::Orthographic(OrthographicProjection {
                far: 54.1e12,
                ..
            }))
        );

        camera.insert(HighResolution);

        assert_matches!(
            camera.get::<RenderTarget>(),
            Some(RenderTarget::None {
                size: UVec2 { x: 831, y: 124 }
            })
        );
        assert!(!matches!(
            camera.get::<Projection>(),
            Some(Projection::Orthographic(OrthographicProjection {
                far: 54.1e12,
                ..
            }))
        ));

        assert_eq!(camera.get::<Children>().unwrap().len(), 3);
    }

    fn test_app() -> App {
        let mut app = App::new();

        app.add_plugins((
            TaskPoolPlugin::default(),
            AssetPlugin::default(),
            ImagePlugin::default(),
            WindowPlugin::default(),
        ));

        app
    }
}
