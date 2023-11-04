# bevy_smooth_pixel_camera

A bevy plugin that adds a simple smooth pixel camera.

## Usage

1. Add the `bevy_smooth_pixel_camera` crate to your project.
```
cargo add bevy_smooth_pixel_camera
```
2. Add the `PixelCameraPlugin` and set the `ImagePlugin` to `default_nearest`.
```rs
app.add_plugins((
    DefaultPlugins.set(ImagePlugin::default_nearest()),
    PixelCameraPlugin
));
```
3. Add a pixel pefect camera to your scene.
```rs
fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        PixelCamera::from_scaling(4)
    ));
}
```
4. That should be it!