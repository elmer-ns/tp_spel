use bevy::prelude::*;

use crate::Player;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<CameraState>();
        app.add_systems(Update, (
            camera_follow.run_if(in_state(CameraState::Follow).or(in_state(CameraState::MoveBack))),
            free_camera.run_if(in_state(CameraState::Free)),
        ));
    }
}

#[derive(Component)]
#[require(Camera2d)]
pub struct MainCamera;

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
pub enum CameraState {
    #[default]
    /// Move towards (follow) the player, changing to [Self::Follow] once the player is within view
    MoveBack,
    /// Move towards (follow) the player, making sure to keep the player in view
    Follow,
    Free,
}

fn camera_follow(
    state: Res<State<CameraState>>,
    mut next_state: ResMut<NextState<CameraState>>,
    camera: Single<(&Camera, &Projection, &mut Transform), With<MainCamera>>,
    player: Single<&Transform, (With<Player>, Without<MainCamera>)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let left = keys.pressed(KeyCode::KeyA);
    let right = keys.pressed(KeyCode::KeyD);
    let up = keys.pressed(KeyCode::KeyW);
    let down = keys.pressed(KeyCode::KeyS);

    if left || right || up || down {
        next_state.set(CameraState::Free);
        return;
    }

    let (_camera, Projection::Orthographic(proj), mut transform) = camera.into_inner() else {
        error!("Camera has wrong projection. Should be Orthographic");
        return
    };

    let d = player.translation - transform.translation;

    transform.translation += d * 0.25;

    let player_pos = player.translation.xy();
    let half_proj_size = proj.area.size() / 2.0;

    let min = player_pos - half_proj_size;
    let max = player_pos + half_proj_size;

    let clamped_pos = player_pos.max(min).min(max);

    if *state == CameraState::Follow {
        let player_pos = player.translation.xy();

        transform.translation.x = clamped_pos.x;
        transform.translation.y = clamped_pos.y;
    } else if *state == CameraState::MoveBack && clamped_pos == player_pos {
        next_state.set(CameraState::Follow);
    }
}

fn free_camera(
    mut next_state: ResMut<NextState<CameraState>>,
    camera: Single<&mut Transform, With<MainCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let left = keys.pressed(KeyCode::KeyA);
    let right = keys.pressed(KeyCode::KeyD);
    let up = keys.pressed(KeyCode::KeyW);
    let down = keys.pressed(KeyCode::KeyS);

    if keys.pressed(KeyCode::KeyR) {
        next_state.set(CameraState::MoveBack);
        return
    }

    const SPEED: f32 = 1.0;

    let x = (right as i32 - left as i32) as f32 * SPEED;
    let y = (down as i32 - up as i32) as f32 * SPEED;

    let mut transform = camera;

    transform.translation += vec3(x,y,0.0);
}