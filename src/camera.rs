use bevy::prelude::*;

use crate::{Player, physics::Velocity, player::Control};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(CameraState::Follow);
        app.add_systems(
            PostUpdate,
            (
                camera_follow.run_if(in_state(CameraState::Follow)),
                free_camera.run_if(in_state(CameraState::Free)),
            ),
        );
    }
}

#[derive(Component)]
#[require(Camera2d)]
pub struct MainCamera;

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum CameraState {
    /// Move towards (follow) the player
    Follow,
    Free,
}

fn camera_follow(
    mut next_state: ResMut<NextState<CameraState>>,
    camera: Single<&mut Transform, With<MainCamera>>,
    player: Single<(&Transform, &Velocity), (With<Player>, Without<MainCamera>)>,
    control: Res<Control>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    if control.left || control.right || control.up || control.down {
        next_state.set(CameraState::Free);
        return;
    }

    let mut c_transform = camera.into_inner();

    let (p_transform, Velocity(p_velocity)) = player.into_inner();

    const TIME_CONSTANT: f32 = 0.3;
    const LOOKAHEAD_TIME: f32 = 0.325;

    let lookahead = p_velocity * LOOKAHEAD_TIME;
    let target = p_transform.translation.xy() + lookahead;

    let t = 1.0 - (-dt / TIME_CONSTANT).exp();

    let c_move = (target - c_transform.translation.xy()) * t;

    c_transform.translation.x += c_move.x;
    c_transform.translation.y += c_move.y;
}

fn free_camera(
    mut next_state: ResMut<NextState<CameraState>>,
    camera: Single<&mut Transform, With<MainCamera>>,
    control: Res<Control>,
) {
    if control.camera_return {
        next_state.set(CameraState::Follow);
        return;
    }

    const SPEED: f32 = 0.125;

    let x = (control.right as i32 - control.left as i32) as f32 * SPEED;
    let y = (control.up as i32 - control.down as i32) as f32 * SPEED;

    let mut transform = camera;

    transform.translation += vec3(x, y, 0.0);
}
