use bevy::{camera::primitives::Aabb, prelude::*};

use crate::Player;

const GRAVITY_ACC: f32 = 9.81; // m/s²

#[derive(Component)]
pub struct SolidBody;

#[derive(Component, Clone, Copy)]
pub struct Bounciness(pub f32);


#[derive(Component, Default, Clone, Copy)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Mass(pub f32);

#[derive(Component)]
pub struct FrictionCoefficient(pub f32);


pub fn player_collision(
    time: Res<Time>,
    player: Single<(&mut Transform, &mut Velocity, Option<&Bounciness>, &Aabb), With<Player>>,
    solids: Query<
        (&Transform, Option<&Velocity>, Option<&Bounciness>, &Aabb),
        (With<SolidBody>, Without<Player>),
    >,
) {
    let dt = time.delta_secs();

    let (mut p_transform, mut p_velocity, p_bounciness, p_aabb) = player.into_inner();
    for (s_transform, s_velocity, s_bounciness, s_aabb) in solids {
        // player position and velocity relative to solid body
        let relative_position = p_transform.translation.xy() - s_transform.translation.xy();
        let relative_velocity = p_velocity.0 - s_velocity.cloned().unwrap_or_default().0;
        let relative_move = relative_velocity * dt;

        //println!("Relative Position: {:?},\nRelative Velocity: {:?},\nRelative Move: {:?},", relative_position, relative_velocity, relative_move);

        let relative_position_pre_move = relative_position - relative_move;

        fn check_collision(a: &Aabb, b: &Aabb, a_translation: &Vec3) -> bool {
            (a.min().x + a_translation.x <= b.max().x && a.max().x + a_translation.x >= b.min().x)
                && (a.min().y + a_translation.y <= b.max().y
                    && a.max().y + a_translation.y >= b.min().y)
        }

        let x_collision = check_collision(
            p_aabb,
            s_aabb,
            &vec3(relative_position.x, relative_position_pre_move.y, 0.0),
        );
        let y_collision = check_collision(
            p_aabb,
            s_aabb,
            &vec3(relative_position_pre_move.x, relative_position.y, 0.0),
        );

        let bounciness =
            p_bounciness.unwrap_or(&Bounciness(1.0)).0 * s_bounciness.unwrap_or(&Bounciness(1.0)).0;

        if x_collision {
            p_velocity.0.x *= -bounciness;

            if relative_position.x <= s_aabb.center.x {
                p_transform.translation.x =
                    s_transform.translation.x + s_aabb.min().x - p_aabb.max().x;
            } else {
                p_transform.translation.x =
                    s_transform.translation.x + s_aabb.max().x - p_aabb.min().x;
            }
        }
        if y_collision {
            p_velocity.0.y *= -bounciness;

            if relative_position.y <= s_aabb.center.y {
                p_transform.translation.y =
                    s_transform.translation.y + s_aabb.min().y - p_aabb.max().y;
            } else {
                p_transform.translation.y =
                    s_transform.translation.y + s_aabb.max().y - p_aabb.min().y;
            }
        }
    }
}

pub fn update_positions(time: Res<Time>, q: Query<(&Velocity, &mut Transform)>) {
    let dt = time.delta_secs();

    for (Velocity(velocity), mut transform) in q {
        transform.translation.x += velocity.x * dt;
        transform.translation.y += velocity.y * dt;
    }
}

pub fn friction(time: Res<Time>, q: Query<(&mut Velocity, &Mass, &FrictionCoefficient)>) {
    let dt = time.delta_secs();

    for (mut velocity, Mass(mass), FrictionCoefficient(friction_coefficient)) in q {
        let normal_force = mass * GRAVITY_ACC;
        let friction_force = normal_force * friction_coefficient;
        let friction_deceleration = friction_force / mass;

        let dir = velocity.0.normalize_or_zero();

        let acceleration = -friction_deceleration * dir; // m/s²

        velocity.0 += acceleration * dt;

        if velocity.0.length() < 0.05 {
            velocity.0 = Vec2::ZERO;
        }
    }
}
