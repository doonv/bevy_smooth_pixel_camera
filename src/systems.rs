use crate::CAMERA_POSITION_OFFSET;
use crate::components::{
    HighResolution, PixelCamera, ViewportCamera, ViewportEntities, ViewportImage,
};
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowRef};

/// Resolves the logical size of a [`RenderTarget`].
pub(crate) fn resolve_target_size(
    target: &RenderTarget,
    window_query: &Query<&Window>,
    primary_window: Option<Entity>,
    images: &Assets<Image>,
) -> Result<Vec2> {
    match target {
        RenderTarget::Window(window_ref) => {
            let entity = match window_ref {
                WindowRef::Primary => primary_window.ok_or("primary window doesn't exist")?,
                &WindowRef::Entity(e) => e,
            };
            let window = window_query.get(entity)?;

            Ok(Vec2::new(window.width(), window.height()))
        }
        RenderTarget::Image(handle) => Ok(images
            .get(&handle.handle)
            .ok_or("image not found")?
            .size_f32()),
        target => Err(format!("Unsupported result type {:?}", target))?,
    }
}

pub(crate) fn update_viewport_size(
    pixel_cameras: Query<(&PixelCamera, &ViewportEntities, &RenderTarget), Without<HighResolution>>,
    mut viewport_cameras: Query<(&mut Projection, &RenderTarget), With<ViewportCamera>>,
    windows: Query<&Window>,
    primary_window: Option<Single<Entity, With<PrimaryWindow>>>,
    mut images: ResMut<Assets<Image>>,
) -> Result<()> {
    for (pixel_camera, viewport, pixel_target) in &pixel_cameras {
        let (viewport_projection, viewport_target) = viewport_cameras.get_mut(viewport.camera)?;

        let window_size = resolve_target_size(
            viewport_target,
            &windows,
            primary_window.as_deref().copied(),
            &images,
        )?;

        let (new_tex_size, new_scaling) = pixel_camera
            .viewport_size
            .get_configuration(window_size, pixel_camera.smoothing);

        // Update image size
        if let RenderTarget::Image(image) = pixel_target
            && let Some(img) = images.get_mut(&image.handle)
            && img.texture_descriptor.size != new_tex_size
        {
            img.resize(new_tex_size);
        }

        // Update projection
        if let Projection::Orthographic(ortho) = viewport_projection.into_inner() {
            ortho.scaling_mode = new_scaling;
        }
    }
    Ok(())
}

/// Snaps the [`PixelCamera`] to the pixel grid while smoothing the positioning of the viewport.
///
/// We snap the [`GlobalTransform`] directly instead of the [`Transform`] so that we don't have to use a separate
/// variable for the position of the camera.
///
/// This system runs after [`TransformSystems::Propagate`] to manually
/// decouple the snapped world position from the smooth viewport position.
pub(crate) fn snap_camera_position(
    mut world_cameras: Query<
        (&mut GlobalTransform, &ViewportEntities, &PixelCamera),
        Without<HighResolution>,
    >,
    mut viewport_sprite_transform: Query<
        (&mut Transform, &mut GlobalTransform),
        (With<ViewportImage>, Without<PixelCamera>),
    >,
) -> Result<()> {
    for (mut camera_global, viewport, pixel_camera) in &mut world_cameras {
        let smooth_transform = camera_global.compute_transform();

        // Project into local space to align snapping with the rotated pixel grid.
        let local_translation = smooth_transform.rotation.inverse() * smooth_transform.translation
            - Vec2::splat(0.5).extend(0.0);

        let snapped_local_translation =
            (local_translation.xy().round() + CAMERA_POSITION_OFFSET).extend(local_translation.z);

        // Recompose the snapped world position.
        let snapped_transform = smooth_transform
            .with_translation(smooth_transform.rotation * snapped_local_translation);

        let subpixel_offset = if pixel_camera.smoothing {
            snapped_local_translation - local_translation
        } else {
            Vec3::ZERO
        };

        let (mut sprite_local, mut sprite_global_transform) =
            viewport_sprite_transform.get_mut(viewport.sprite)?;
        sprite_local.translation = sprite_local.translation.with_xy(subpixel_offset.xy());

        // Manually update the child GlobalTransform since propagation is finished.
        *sprite_global_transform = camera_global.mul_transform(*sprite_local);

        // Overwrite the GlobalTransform with the snapped position for rendering.
        *camera_global = GlobalTransform::from(snapped_transform);
    }
    Ok(())
}

pub(crate) fn sync_camera_fields(
    pixel_cameras: Query<(Ref<PixelCamera>, &ViewportEntities), Without<HighResolution>>,
    mut viewport_cameras: Query<(&mut RenderLayers, &mut Camera)>,
) -> Result<()> {
    for (pixel_camera, viewport) in pixel_cameras {
        if !pixel_camera.is_changed() {
            continue;
        }

        let (mut viewport_camera_layers, mut viewport_camera) =
            viewport_cameras.get_mut(viewport.camera)?;

        *viewport_camera_layers = pixel_camera.viewport_layers.clone();
        viewport_camera.order = pixel_camera.viewport_order;
        if let Some(clear_color) = pixel_camera.viewport_size.clear_color() {
            viewport_camera.clear_color = clear_color;
        }
    }

    Ok(())
}

/// This mostly just for [`crate::viewport::ViewportScalingMode::Custom`].
pub(crate) fn update_high_resolution_viewport_size(
    mut pixel_cameras: Query<(&mut Projection, &PixelCamera, &RenderTarget), With<HighResolution>>,
    windows: Query<&Window>,
    primary_window: Option<Single<Entity, With<PrimaryWindow>>>,
    images: Res<Assets<Image>>,
) -> Result<()> {
    for (projection, pixel_camera, render_target) in &mut pixel_cameras {
        let window_size = resolve_target_size(
            render_target,
            &windows,
            primary_window.as_deref().copied(),
            &images,
        )?;

        let (_, new_scaling) = pixel_camera
            .viewport_size
            .get_configuration(window_size, pixel_camera.smoothing);

        if let Projection::Orthographic(orthographic_projection) = projection.into_inner() {
            orthographic_projection.scaling_mode = new_scaling;
        }
    }
    Ok(())
}
