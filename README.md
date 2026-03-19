# bevy_smooth_pixel_camera

[![crates.io](https://img.shields.io/crates/v/bevy_smooth_pixel_camera)](https://crates.io/crates/bevy_smooth_pixel_camera)
[![docs.rs](https://docs.rs/bevy_smooth_pixel_camera/badge.svg)](https://docs.rs/bevy_smooth_pixel_camera)

<!-- The intralinks need to be manually fixed every time you update the readme with cargo rdme because it doesn't support intralinks very well, if at all. -->

A bevy plugin that adds a simple smooth pixel camera.

| Smoothing OFF                     | Smoothing ON                      |
| :-------------------------------: | :-------------------------------: |
| ![](https://placehold.co/384x216) | ![](https://placehold.co/384x216) |

<!-- cargo-rdme start -->

This method allows for smooth camera movement while retaining the pixel perfection of low resolution rendering.

## Usage

1. Add the `bevy_smooth_pixel_camera` crate to your project.

    ```sh
    cargo add bevy_smooth_pixel_camera
    ```

2. Add the [`PixelCameraPlugin`](https://docs.rs/bevy_smooth_pixel_camera/latest/bevy_smooth_pixel_camera/struct.PixelCameraPlugin.html) and set the [`ImagePlugin`](https://docs.rs/bevy_image/latest/bevy_image/image/struct.ImagePlugin.html) to [`default_nearest`](https://docs.rs/bevy_image/latest/bevy_image/struct.ImagePlugin.html#method.default_nearest).

    ```rust
    use bevy::prelude::*;
    use bevy_smooth_pixel_camera::prelude::*;
    
    App::new().add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        PixelCameraPlugin
    )).run();
    ```

3. Add a [`PixelCamera`](https://docs.rs/bevy_smooth_pixel_camera/latest/bevy_smooth_pixel_camera/components/struct.PixelCamera.html) to your world.

    ```rust
    use bevy::prelude::*;
    use bevy_smooth_pixel_camera::prelude::*;

    fn setup(mut commands: Commands) {
        commands.spawn(PixelCamera::from_size(ViewportScalingMode::PixelSize(4.0)));
    }
    ```

4. That's it!

## Bevy Compatibility

| bevy   | bevy_smooth_pixel_camera |
| ------ | ------------------------ |
| 0.18.* | 0.4.0 - main             |
| 0.13.* | 0.3.0                    |
| 0.12.* | 0.1.0 - 0.2.1            |

<!-- cargo-rdme end -->
