use bevy::{prelude::*, window::PrimaryWindow};

use crate::camera::MainCamera;

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct Control {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,

    pub camera_return: bool,

    pub hold: Option<Vec2>,
    pub drop_from: Option<Vec2>,

    pub cursor: Vec2,
    pub cursor_world: Vec2,
}

impl Default for Control {
    fn default() -> Self {
        Self {
            left: false,
            right: false,
            up: false,
            down: false,
            camera_return: false,
            hold: None,
            drop_from: None,
            cursor: Vec2::ZERO,
            cursor_world: Vec2::ZERO,
        }
    }
}

pub fn input_system(
    mut control: ResMut<Control>,
    q_camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    k_buttons: Res<ButtonInput<KeyCode>>,
    m_buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let (camera, camera_transform) = q_camera.into_inner();

    control.left = k_buttons.pressed(KeyCode::KeyA) || k_buttons.pressed(KeyCode::ArrowLeft);
    control.right = k_buttons.pressed(KeyCode::KeyD) || k_buttons.pressed(KeyCode::ArrowRight);
    control.up = k_buttons.pressed(KeyCode::KeyW) || k_buttons.pressed(KeyCode::ArrowUp);
    control.down = k_buttons.pressed(KeyCode::KeyS) || k_buttons.pressed(KeyCode::ArrowDown);

    control.camera_return = k_buttons.pressed(KeyCode::KeyR);

    let left_m_down = m_buttons.pressed(MouseButton::Left);
    //let right_m_down = m_buttons.pressed(MouseButton::Right);

    let Some((cursor_world, cursor)) = window
        .cursor_position()
        .and_then(|cursor| {
            camera
                .viewport_to_world(camera_transform, cursor)
                .ok()
                .map(|ray| (ray, cursor))
        })
        .map(|(ray, cursor)| (ray.origin.xy(), cursor))
    else {
        return;
    };

    control.cursor = cursor;
    control.cursor_world = cursor_world;

    control.drop_from = None;

    if left_m_down {
        if control.hold.is_none() {
            control.hold = Some(cursor)
        }
    } else {
        control.drop_from = control.hold.take();
    }
}
