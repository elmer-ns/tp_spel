use bevy::prelude::*;

use crate::{Player, player::Control};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(CameraState::Follow { lock: false });
        app.add_systems(
            PostUpdate,
            (
                camera_follow.run_if(
                    in_state(CameraState::Follow { lock: false })
                        .or(in_state(CameraState::Follow { lock: true })),
                ),
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
    Follow {
        lock: bool,
    },
    Free,
}

fn camera_follow(
    state: Res<State<CameraState>>,
    mut next_state: ResMut<NextState<CameraState>>,
    camera: Single<(&Camera, &Projection, &mut Transform), With<MainCamera>>,
    player: Single<&Transform, (With<Player>, Without<MainCamera>)>,
    control: Res<Control>,
) {
    if control.left || control.right || control.up || control.down {
        next_state.set(CameraState::Free);
        return;
    }

    let (_camera, Projection::Orthographic(proj), mut transform) = camera.into_inner() else {
        error!("Camera has wrong projection. Should be Orthographic");
        return;
    };

    let d = player.translation - transform.translation;

    transform.translation += d * 0.125;

    let player_pos = player.translation.xy();
    let half_proj_size = proj.area.size() * 0.5;

    const MARGIN_FACTOR: f32 = 0.75;

    let min = player_pos - half_proj_size * MARGIN_FACTOR;
    let max = player_pos + half_proj_size * MARGIN_FACTOR;

    let pos = transform.translation.xy();

    let clamped_pos = pos.max(min).min(max);

    let CameraState::Follow { lock } = **state else {
        unreachable!()
    };

    if lock && false {
        transform.translation.x = clamped_pos.x;
        transform.translation.y = clamped_pos.y;
    } else if (clamped_pos - transform.translation.xy()).length_squared() < 0.01 {
        next_state.set(CameraState::Follow { lock: true });
    }
}

fn free_camera(
    mut next_state: ResMut<NextState<CameraState>>,
    camera: Single<&mut Transform, With<MainCamera>>,
    control: Res<Control>,
) {
    if control.camera_return {
        next_state.set(CameraState::Follow { lock: false });
        return;
    }

    const SPEED: f32 = 0.125;

    let x = (control.right as i32 - control.left as i32) as f32 * SPEED;
    let y = (control.up as i32 - control.down as i32) as f32 * SPEED;

    let mut transform = camera;

    transform.translation += vec3(x, y, 0.0);
}
