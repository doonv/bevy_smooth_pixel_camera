use bevy::prelude::*;
use bevy::render::camera::{RenderTarget, ScalingMode};
use bevy::render::render_resource::*;
use bevy::render::view::RenderLayers;
use bevy::window::{PrimaryWindow, WindowRef};

use crate::components::*;
use crate::prelude::ViewportSize;
use crate::viewport::FitMode;

pub(crate) fn init_camera(
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
            viewport_size,
            viewport_layer,
            smoothing,
            ..
        },
        mut camera,
        world_layer,
        entity,
    ) in &mut query
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

        let mut size = viewport_size.calculate(&window.resolution);
        if *smoothing {
            size.width += 2;
            size.height += 2;
        }

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

        let viewport_sprite = commands
            .spawn((
                SpriteBundle {
                    texture: image_handle,
                    transform: Transform::from_scale(Vec3::splat(1.0)),
                    ..default()
                },
                *viewport_layer,
                PixelViewport,
            ))
            .id();

        let viewport_camera = commands
            .spawn((
                Camera2dBundle {
                    camera: Camera {
                        order: *viewport_order,
                        clear_color: viewport_size.clear_color(),
                        ..default()
                    },
                    projection: OrthographicProjection {
                        far: 1000.,
                        near: -1000.,
                        scaling_mode: ScalingMode::Fixed {
                            width: (size.width - 2) as f32,
                            height: (size.height - 2) as f32,
                        },
                        ..default()
                    },

                    ..default()
                },
                ViewportCamera,
                *viewport_layer,
            ))
            .id();

        commands.entity(entity).insert(PixelViewportReferences {
            sprite: viewport_sprite,
            camera: viewport_camera,
        });
    }
}

pub(crate) fn update_viewport_size(
    primary_cameras: Query<
        (Entity, &PixelCamera, &Camera, &PixelViewportReferences),
        Without<ViewportCamera>,
    >,
    mut viewport_cameras: Query<(&mut OrthographicProjection, &mut Camera), With<ViewportCamera>>,
    windows: Query<Ref<Window>>,
    primary_window: Query<Ref<Window>, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (
        entity,
        PixelCamera {
            viewport_size,
            smoothing,
            ..
        },
        camera,
        viewport,
    ) in &primary_cameras
    {
        let Ok((mut viewport_projection, mut viewport_camera)) =
            viewport_cameras.get_mut(viewport.camera)
        else {
            error!("PixelCamera {entity:?}'s viewport camera no longer exists.");
            continue;
        };
        let (mut new_size, aspect_ratio) = match &viewport_camera.target {
            RenderTarget::Window(window_ref) => {
                let window = match window_ref {
                    WindowRef::Primary => {
                        if let Ok(window) = primary_window.get_single() {
                            window
                        } else {
                            error!("The primary window that the PixelCamera is pointing to doesn't exist.");
                            continue;
                        }
                    }
                    &WindowRef::Entity(entity) => {
                        if let Ok(window) = windows.get(entity) {
                            window
                        } else {
                            error!("Window {entity:?} that the PixelCamera is pointing to doesn't exist.");
                            continue;
                        }
                    }
                };
                if !window.is_changed() {
                    continue;
                }

                let new_size = viewport_size.calculate(&window.resolution);
                let aspect_ratio = window.width() / window.height();

                (new_size, aspect_ratio)
            }
            RenderTarget::Image(image) => {
                let image = images
                    .get(image)
                    .expect("RenderTarget::Image doesn't exist");
                let size = image.size();

                let new_size = Extent3d {
                    width: size.x,
                    height: size.y,
                    ..default()
                };
                let aspect_ratio = size.x as f32 / size.y as f32;

                (new_size, aspect_ratio)
            }
            RenderTarget::TextureView(_) => {
                error_once!(
                    "RenderTarget::TextureView is not yet supported for `bevy_smooth_pixel_camera`"
                );
                return;
            }
        };

        viewport_projection.scaling_mode = if let ViewportSize::Fixed { fit, .. }
        | ViewportSize::Custom { fit, .. } = viewport_size
        {
            match fit {
                FitMode::Fit(clear_color) => {
                    viewport_camera.clear_color = clear_color.clone();
                    if aspect_ratio > new_size.width as f32 / new_size.height as f32 {
                        ScalingMode::Fixed {
                            width: new_size.height as f32 * (aspect_ratio),
                            height: new_size.height as f32,
                        }
                    } else {
                        ScalingMode::Fixed {
                            width: new_size.width as f32,
                            height: new_size.width as f32 / (aspect_ratio),
                        }
                    }
                }
                FitMode::Crop => {
                    let axis = new_size.height.min(new_size.width);
                    if aspect_ratio > 1.0 {
                        ScalingMode::Fixed {
                            width: axis as f32,
                            height: axis as f32 / (aspect_ratio),
                        }
                    } else {
                        ScalingMode::Fixed {
                            width: axis as f32 * (aspect_ratio),
                            height: axis as f32,
                        }
                    }
                }
                FitMode::Stretch => ScalingMode::Fixed {
                    width: new_size.width as f32,
                    height: new_size.height as f32,
                },
            }
        } else {
            ScalingMode::Fixed {
                width: new_size.width as f32,
                height: new_size.height as f32,
            }
        };

        if *smoothing {
            new_size.width += 2;
            new_size.height += 2;
        }
        if let RenderTarget::Image(image_handle) = &camera.target {
            if let Some(image) = images.get_mut(image_handle) {
                image.resize(new_size);
            } else {
                error!("Pixel camera render target image doesn't exist!");
            }
        }
    }
}

/// Set the camera transform the rounded down version of the subpixel position
pub(crate) fn set_camera_position(mut cameras: Query<(&PixelCamera, &mut Transform)>) {
    for (PixelCamera { subpixel_pos, .. }, mut transform) in &mut cameras {
        transform.translation.x = subpixel_pos.x.trunc();
        transform.translation.y = subpixel_pos.y.trunc();
    }
}

/// Smooth the camera's subpixel position
#[allow(clippy::type_complexity)]
pub(crate) fn smooth_camera(
    mut cameras: Query<(&PixelCamera, &PixelViewportReferences)>,
    mut viewports: Query<
        (&mut Sprite, &Handle<Image>),
        (With<PixelViewport>, Without<PixelViewportReferences>),
    >,
    images: Res<Assets<Image>>,
) {
    for (
        PixelCamera {
            subpixel_pos,
            smoothing,
            ..
        },
        viewport,
    ) in &mut cameras
    {
        if !smoothing {
            continue;
        }
        let (mut sprite, handle) = viewports.get_mut(viewport.sprite).unwrap();
        let Some(image) = images.get(handle) else {
            error!(
                "Pixel camera viewport ({:?}) image doesn't exist",
                viewport.sprite
            );
            continue;
        };

        // In order to get smooth camera movement while retaining pixel perfection,
        // we can move the viewport's transform by the remainder of the subpixel.
        //
        // The smoothing is based on this video: https://youtu.be/jguyR4yJb1M?t=98
        let remainder = Vec2 {
            x: subpixel_pos.x % 1.0,
            // The y axis on sprite.rect is inverted, so we need to invert our y to counteract this.
            y: -subpixel_pos.y % 1.0,
        };

        sprite.rect = Some(Rect {
            min: Vec2::ONE + remainder,
            max: image.size_f32() - Vec2::ONE + remainder,
        })
    }
}
