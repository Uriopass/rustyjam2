use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseWheel;
use bevy::input::ElementState;
use bevy::prelude::*;
use bevy::render::camera::Camera2d;
use std::collections::HashSet;

#[derive(Component)]
pub(crate) struct Yoo;

pub(crate) fn gfx_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn().insert_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(10000.0, 10000.0)),
            ..Default::default()
        },
        transform: Transform::default()
            .with_scale(Vec3::new(1.0, 1.0, 1.0))
            .with_translation(Vec3::new(0.0, 0.0, 0.00)),
        texture: asset_server.load("fond.jpg"),
        ..SpriteBundle::default()
    });

    commands.spawn().insert_bundle(SpriteBundle {
        transform: Transform::default()
            .with_scale(Vec3::new(1.0, 1.0, 1.0))
            .with_translation(Vec3::new(-500.0, -800.0, 0.02)),
        texture: asset_server.load("enclot.png"),
        ..SpriteBundle::default()
    });
    commands.spawn().insert_bundle(SpriteBundle {
        transform: Transform::default()
            .with_scale(Vec3::new(1.0, 1.0, 1.0))
            .with_translation(Vec3::new(0.0, -800.0, 0.02)),
        texture: asset_server.load("enclot.png"),
        ..SpriteBundle::default()
    });
    commands.spawn().insert_bundle(SpriteBundle {
        transform: Transform::default()
            .with_scale(Vec3::new(1.0, 1.0, 1.0))
            .with_translation(Vec3::new(500.0, -800.0, 0.02)),
        texture: asset_server.load("enclot.png"),
        ..SpriteBundle::default()
    });
    commands.spawn().insert_bundle(SpriteBundle {
        transform: Transform::default()
            .with_scale(Vec3::new(2.0, 1.0, 1.0))
            .with_translation(Vec3::new(0.0, 350.0, 0.001)),
        texture: asset_server.load("forest_bg.png"),
        ..SpriteBundle::default()
    });

    let mut already: Vec<(f32, f32)> = vec![];

    for _ in 0..300 {
        let x = (-0.5 + fastrand::f32()) * 1500.0;
        let y = (-0.5 + 0.5 * fastrand::f32()) * 700.0;

        let mut is_ok = true;
        for (xx, yy) in &already {
            if (xx - x).abs().hypot((yy - y).abs()) < 20.0 {
                is_ok = false;
                break;
            }
        }
        if !is_ok {
            continue;
        }

        already.push((x, y));

        commands.spawn().insert_bundle(SpriteBundle {
            transform: Transform::default()
                .with_scale(Vec3::new(1.0, 1.0, 1.0))
                .with_translation(Vec3::new(x, 550.0 + y, 0.2 - y * 0.00001)),
            texture: asset_server.load("trunk.png"),
            ..SpriteBundle::default()
        });

        commands.spawn().insert_bundle(SpriteBundle {
            transform: Transform::default()
                .with_scale(Vec3::new(1.0, 1.0, 1.0))
                .with_translation(Vec3::new(x, 550.0 + y, 0.3 - y * 0.001)),
            texture: asset_server.load("leaves.png"),
            ..SpriteBundle::default()
        });
    }

    commands.insert_resource(MouseProj(Vec2::default()));
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub enum Action {
    CamRight,
    CamLeft,
    CamUp,
    CamDown,
    Zoom,
    Dezoom,
}

#[derive(Default)]
pub struct Inputs {
    just_pressed: HashSet<Action>,
    pressed: HashSet<Action>,
}

pub(crate) fn cam_movement(
    time: Res<Time>,
    inp: Res<Inputs>,
    mut cam: Query<&mut Transform, With<Camera>>,
) {
    let mut cam = cam.single_mut();

    let zoom = cam.scale.x.min(1.5);
    if inp.pressed.contains(&Action::CamUp) {
        cam.translation.y += zoom * 1000.0 * time.delta_seconds();
    }
    if inp.pressed.contains(&Action::CamDown) {
        cam.translation.y -= zoom * 1000.0 * time.delta_seconds();
    }
    if inp.pressed.contains(&Action::CamRight) {
        cam.translation.x += zoom * 1000.0 * time.delta_seconds();
    }
    if inp.pressed.contains(&Action::CamLeft) {
        cam.translation.x -= zoom * 1000.0 * time.delta_seconds();
    }

    const ZOOM_AMT: f32 = 0.9;
    if inp.just_pressed.contains(&Action::Zoom) {
        cam.scale.x *= ZOOM_AMT;
        cam.scale.y *= ZOOM_AMT;
    }
    if inp.just_pressed.contains(&Action::Dezoom) {
        cam.scale.x *= 1.0 / ZOOM_AMT;
        cam.scale.y *= 1.0 / ZOOM_AMT;
    }

    cam.translation.x = cam.translation.x.clamp(-1000.0, 1000.0);
    cam.translation.y = cam.translation.y.clamp(-1000.0, 1000.0);

    cam.scale.x = cam.scale.x.clamp(0.01, 2.0);
    cam.scale.y = cam.scale.y.clamp(0.01, 2.0);
}

pub(crate) fn input_mapping(
    mut inputs: ResMut<Inputs>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    inputs.just_pressed.clear();

    for v in keyboard_input_events.iter() {
        let action = match v.scan_code {
            17 => Action::CamUp,
            31 => Action::CamDown,
            32 => Action::CamRight,
            30 => Action::CamLeft,
            _ => continue,
        };

        if v.state == ElementState::Pressed {
            inputs.just_pressed.insert(action);
            inputs.pressed.insert(action);
        }
        if v.state == ElementState::Released {
            inputs.pressed.remove(&action);
        }
    }

    for v in scroll_evr.iter() {
        if v.y > 0.0 {
            inputs.just_pressed.insert(Action::Zoom);
        }
        if v.y < 0.0 {
            inputs.just_pressed.insert(Action::Dezoom);
        }
    }
}

pub struct MouseProj(pub(crate) Vec2);

pub(crate) fn mouse_project(
    mut commands: Commands,
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // get the window that the camera is displaying to (or the primary window)
    let wnd = if let bevy::render::camera::RenderTarget::Window(id) = camera.target {
        wnds.get(id).unwrap()
    } else {
        wnds.get_primary().unwrap()
    };

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = wnd.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        commands.insert_resource(MouseProj(world_pos));
    }
}
