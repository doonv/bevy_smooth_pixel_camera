# bevy_smooth_pixel_camera

[![crates.io](https://img.shields.io/crates/v/bevy_smooth_pixel_camera?style=flat-square)](https://crates.io/crates/bevy_smooth_pixel_camera)
[![docs.rs](https://img.shields.io/docsrs/bevy_smooth_pixel_camera?style=flat-square)](https://docs.rs/bevy_smooth_pixel_camera)
![GitHub branch check runs](https://img.shields.io/github/check-runs/doonv/bevy_smooth_pixel_camera/main?style=flat-square)
[![Following released Bevy versions](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue?style=flat-square)](https://bevy.org/learn/quick-start/plugin-development/#main-branch-tracking)

<!-- WHEN UPDATING README WITH cargo rdme:
Use --heading-base-level 0.
Comment out the intralink definitions at the bottom for `default_nearest` and smoothing.
 -->
<!-- cargo-rdme start -->

A bevy plugin that adds a simple smooth pixel camera.

It works by rendering the main camera to a small viewport which is then rendered by a second camera spawned by the plugin.
This allows for hybrid rendering of both a pixelated world and high resolution assets on top.

This plugin has a [smoothing] feature, which makes the camera's movement appear smooth while keeping the world itself locked
to a pixel grid. It works by moving the canvas in the opposite direction of the world camera's subpixel position. See the
[`how_smoothing_works`] example for a demonstration of how it works behind the scenes.

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
    
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PixelCameraPlugin,
        ))
        .run();
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

## Features

`picking` **(default)** - Enables [picking] through the viewport.

## Bevy Compatibility

| bevy   | bevy_smooth_pixel_camera |
| ------ | ------------------------ |
| 0.18.* | 0.4.x - [`main`]         |
| 0.13.* | 0.3.0                    |
| 0.12.* | 0.1.0 - 0.2.1            |

[smoothing_off]: https://raw.githubusercontent.com/doonv/bevy_smooth_pixel_camera/main/assets/smoothing_off.avif
[smoothing_on]: https://raw.githubusercontent.com/doonv/bevy_smooth_pixel_camera/main/assets/smoothing_on.avif
[`how_smoothing_works`]: https://github.com/doonv/bevy_smooth_pixel_camera/blob/main/examples/how_smoothing_works.rs
[`main`]: https://github.com/doonv/bevy_smooth_pixel_camera

<!-- [`default_nearest`]: ImagePlugin::default_nearest
[smoothing]: components::PixelCamera::smoothing
[picking]: bevy::picking -->

<!-- cargo-rdme end -->

[`PixelCameraPlugin`]: https://docs.rs/bevy_smooth_pixel_camera/latest/bevy_smooth_pixel_camera/struct.PixelCameraPlugin.html
[`ImagePlugin`]: https://docs.rs/bevy_image/latest/bevy_image/image/struct.ImagePlugin.html
[`default_nearest`]: https://docs.rs/bevy_image/latest/bevy_image/struct.ImagePlugin.html#method.default_nearest
[smoothing]: https://docs.rs/bevy_smooth_pixel_camera/latest/bevy_smooth_pixel_camera/components/struct.PixelCamera.html#structfield.smoothing
[picking]: https://docs.rs/bevy/latest/bevy/picking/index.html
