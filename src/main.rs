use std::collections::HashMap;

use bevy::{
    camera::primitives::Aabb, color::palettes::css::{BLACK, BLUE, WHITE}, post_process::bloom::Bloom, prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef, sprite_render::{Material2d, Material2dPlugin}, window::PrimaryWindow
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Material2dPlugin::<CheckeredMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (control_player, move_camera, collision, update_positions, friction),
        )
        .run();
}

const GRAVITY_ACC: f32 = 9.81; // m/s²

#[derive(Component)]
struct Player {
    hold: Option<Vec2>,
}

#[derive(Component)]
struct MainCamera;

fn control_player(
    player: Single<(&mut Player, &mut Velocity, &Transform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut gizmos: Gizmos,
) {
    let (mut player, mut velocity, transform) = player.into_inner();
    let (camera, camera_transform) = camera.into_inner();

    let Some((cursor_world, cursor)) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok().map(|ray| (ray, cursor)))
        .map(|(ray, cursor)| (ray.origin.xy(), cursor))
    else {
        return;
    };

    let button_pressed = buttons.pressed(MouseButton::Left); 

    if let Some(hold_cursor) = player.hold {
        let Ok(hold_world) = camera.viewport_to_world_2d(camera_transform, hold_cursor) else {
            return
        };

        let world_difference = hold_world - cursor_world;
        let player_position = transform.translation.xy();
        gizmos.line_2d(player_position, player_position - world_difference, BLUE);

        if !button_pressed {
            let cursor_difference = {
                let d = hold_cursor - cursor;
                vec2(d.x, -d.y)
            };

            velocity.0 += cursor_difference * 0.05;
            velocity.0 = velocity.0.normalize() * velocity.0.length().min(10.0);

            player.hold = None;
        }
    } else {
        if button_pressed {
            player.hold = Some(cursor);
        }
    }
}

fn move_camera(
    mut camera: Single<&mut Transform, With<MainCamera>>,
    player: Single<&Transform, (With<Player>, Without<MainCamera>)>,
) {
    let d = player.translation - camera.translation;

    camera.translation += d * 0.25;
}

#[derive(Component)]
struct SolidBody;

fn collision(
    mut commands: Commands,
    time: Res<Time>,
    bodies: Query<(Entity, &GlobalTransform, &mut Velocity, &Mass, &Aabb), Without<SolidBody>>,
    solids: Query<(Entity, &GlobalTransform, Option<&Velocity>, &Aabb), With<SolidBody>>,
) {
    for (body, body_t, body_v, _body_m, body_bb) in bodies {
        for (solid, solid_t, solid_v, solid_bb) in solids {
            let dt = time.delta_secs();

            let b_move = body_v.0 * dt;
            let b_min = body_bb.min().xy() + solid_t.translation().xy() + b_move;
            let b_max = body_bb.max().xy() + solid_t.translation().xy() + b_move;

            let s_move = solid_v.map(|v| v.0).unwrap_or(Vec2::ZERO) * dt;
            let s_min = solid_bb.min().xy() + solid_t.translation().xy() + s_move;
            let s_max = solid_bb.max().xy() + solid_t.translation().xy() + s_move;

            let collision_x = b_min.x <= s_max.x && b_max.x >= s_min.x;
            let collision_y = b_min.y <= s_max.y && b_max.y >= s_min.y;

            let mut body = commands.entity(body);
        
            if collision_x && collision_y {
                body.insert(Velocity(vec2(-body_v.0.x, -body_v.0.y)));
            } else if collision_x {
                body.insert(Velocity(vec2(-body_v.0.x, body_v.0.y)));
            } else if collision_y {
                body.insert(Velocity(vec2(body_v.0.x, -body_v.0.y)));
            }

        }
    }
}

#[derive(Component)]
struct Velocity(pub Vec2);

#[derive(Component)]
struct Mass(pub f32);

#[derive(Component)]
struct FrictionCoefficient(pub f32);

#[derive(Asset, TypePath, AsBindGroup, Clone, Copy)]
struct CheckeredMaterial {
    #[uniform(0)]
    tile_width: f32,
    #[uniform(1)]
    tile_height: f32,
    #[uniform(2)]
    color_a: LinearRgba,
    #[uniform(3)]
    color_b: LinearRgba,
}

impl Material2d for CheckeredMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        ShaderRef::Path("shaders/checkered.wgsl".into())
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut checkered_materials: ResMut<Assets<CheckeredMaterial>>,
) {
    let circle = meshes.add(Circle::new(0.05));

    // commands.spawn((
    //     SolidBody, 
    //     Aabb::from_min_max(vec3(-10.0, -10.0, 0.0), vec3(10.0, 10.0, 0.0)),
    //     Transform::from_xyz(100.0, 0.0, 0.0),
    //     Mesh2d(meshes.add(Rectangle::from_corners(vec2(-10.0, -10.0), vec2(10.0, 10.0)))),
    //     MeshMaterial2d(materials.add(Color::Srgba(Srgba::BLUE))),
    // ));

    commands.spawn((
        Player { hold: None },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Velocity(vec2(1.0, 1.0)),
        Mass(0.045),
        FrictionCoefficient(0.5),
        Mesh2d(circle),
        MeshMaterial2d(materials.add(Color::Srgba(Srgba::RED))),
    ));

    let mut proj = OrthographicProjection::default_2d();
    proj.scale = 0.005;
    commands.spawn((
        MainCamera,
        Camera2d,
        Projection::Orthographic(proj),
        Bloom::default(),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(2000.0, 2000.0))),
        MeshMaterial2d(checkered_materials.add(CheckeredMaterial {
            tile_width: 1.0,
            tile_height: 1.0,
            color_a: WHITE.into(),
            color_b: BLACK.into(),
        })),
    ));
}

fn update_positions(time: Res<Time>, q: Query<(&Velocity, &mut Transform)>) {
    let dt = time.delta_secs();

    for (Velocity(velocity), mut transform) in q {
        transform.translation.x += velocity.x * dt;
        transform.translation.y += velocity.y * dt;
    }
}

fn friction(time: Res<Time>, q: Query<(&mut Velocity, &Mass, &FrictionCoefficient)>) {
    let dt = time.delta_secs();

    for (mut velocity, Mass(mass), FrictionCoefficient(friction_coefficient)) in q {
        let normal_force = mass * GRAVITY_ACC;
        let friction_force = normal_force * friction_coefficient;
        let friction_deceleration = friction_force / mass;

        let dir = velocity.0.normalize();

        let acceleration = -friction_deceleration * dir; // m/s²

        velocity.0 += acceleration * dt;
    }
}
