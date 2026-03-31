# bevy_smooth_pixel_camera

[![crates.io](https://img.shields.io/crates/v/bevy_smooth_pixel_camera?style=flat-square)](https://crates.io/crates/bevy_smooth_pixel_camera)
[![docs.rs](https://img.shields.io/docsrs/bevy_smooth_pixel_camera?style=flat-square)](https://docs.rs/bevy_smooth_pixel_camera)
![GitHub branch check runs](https://img.shields.io/github/check-runs/doonv/bevy_smooth_pixel_camera/main)

<!-- WHEN UPDATING README WITH cargo rdme:
Use --heading-base-level 0.
Comment out the intralink definitions at the bottom for `default_nearest` and `smoothing`.
And change `smoothing_on` and `smoothing_off` to be local links.
 -->
<!-- cargo-rdme start -->

A bevy plugin that adds a simple smooth pixel camera.

It works by rendering the main camera to a small viewport which is then rendered by a second camera spawned by the plugin.
This allows for hybrid rendering of both a pixelated world and high resolution assets on top.

This plugin has a [smoothing] feature, which makes the camera's movement appear smooth while keeping the world itself locked
to a pixel grid. It works by moving the canvas in the opposite direction of the world camera's subpixel position. See the
`how_smoothing_works` example for a demonstration of how it works behind the scenes.

| Smoothing OFF                                                                   | Smoothing ON                                                                                    |
| :-----------------------------------------------------------------------------: | :---------------------------------------------------------------------------------------------: |
| ![The camera is locked to the pixel grid, causing jagged motion][smoothing_off] | ![The camera moves smoothly while the world itself stays locked to a pixel grid.][smoothing_on] |

## Usage

1. Add the `bevy_smooth_pixel_camera` crate to your project.

    ```sh
    cargo add bevy_smooth_pixel_camera
    ```

2. Add the [`PixelCameraPlugin`] and set [`ImagePlugin`] to [`default_nearest`].

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
| 0.18.* | 0.4.0 - `main`           |
| 0.13.* | 0.3.0                    |
| 0.12.* | 0.1.0 - 0.2.1            |

<!-- [`default_nearest`]: ImagePlugin::default_nearest -->
[smoothing_off]: ./assets/smoothing_off.avif
[smoothing_on]: ./assets/smoothing_on.avif
<!-- [smoothing]: components::PixelCamera::smoothing -->

<!-- cargo-rdme end -->

[`PixelCameraPlugin`]: https://docs.rs/bevy_smooth_pixel_camera/latest/bevy_smooth_pixel_camera/struct.PixelCameraPlugin.html
[`ImagePlugin`]: https://docs.rs/bevy_image/latest/bevy_image/image/struct.ImagePlugin.html
[`default_nearest`]: https://docs.rs/bevy_image/latest/bevy_image/struct.ImagePlugin.html#method.default_nearest
[smoothing]: https://docs.rs/bevy_smooth_pixel_camera/latest/bevy_smooth_pixel_camera/components/struct.PixelCamera.html#structfield.smoothing
