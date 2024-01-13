# bevy_smooth_pixel_camera

[![crates.io](https://img.shields.io/crates/v/bevy_smooth_pixel_camera)](https://crates.io/crates/bevy_smooth_pixel_camera)
[![docs.rs](https://docs.rs/bevy_smooth_pixel_camera/badge.svg)](https://docs.rs/bevy_smooth_pixel_camera)

A bevy plugin that adds a simple smooth pixel camera.

The smoothing is based on this video from aarthificial which explains how it works pretty nicely: <https://youtu.be/jguyR4yJb1M>

This method allows for smooth camera movement while retaining the pixel perfection of low resolution rendering.

## Usage

1. Add the `bevy_smooth_pixel_camera` crate to your project.

    ```sh
    cargo add bevy_smooth_pixel_camera
    ```

2. Add the `PixelCameraPlugin` and set the `ImagePlugin` to `default_nearest`.

    ```rust,no_run
    use bevy::prelude::*;
    use bevy_smooth_pixel_camera::prelude::*;

    App::new().add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        PixelCameraPlugin
    )).run();
    ```

3. Add a pixel pefect camera to your scene.

    ```rust
    use bevy::prelude::*;
    use bevy_smooth_pixel_camera::prelude::*;

    fn setup(mut commands: Commands) {
        commands.spawn((
            Camera2dBundle::default(),
            PixelCamera::from_size(ViewportSize::PixelFixed(4))
        ));
    }
    ```

4. That should be it! Make sure you move your camera via the `PixelCamera.subpixel_pos` property instead of the `Transform` component.

## Bevy Compatibility

| bevy   | bevy_smooth_pixel_camera |
| ------ | ------------------------ |
| 0.12.* | 0.1.0 - 0.2.1            |
