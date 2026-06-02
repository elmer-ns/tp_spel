use bevy::{
    camera::primitives::Aabb,
    color::palettes::css::{BLACK, BLUE, WHITE},
    post_process::bloom::Bloom,
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
    window::PrimaryWindow,
};

use crate::{
    camera::{CameraPlugin, MainCamera},
    physics::{
        Bounciness, FrictionCoefficient, Mass, SolidBody, Velocity, friction, player_collision,
        update_positions,
    },
    player::{Control, Player, input_system},
};

mod camera;
mod physics;
mod player;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Material2dPlugin::<CheckeredMaterial>::default(),
            CameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, input_system)
        .add_systems(
            Update,
            (control_player, friction, update_positions, player_collision),
        )
        .init_resource::<Control>()
        .run();
}

fn control_player(
    player: Single<(&mut Player, &mut Velocity, &Transform)>,
    control: Res<Control>,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut gizmos: Gizmos,
) {
    let (mut player, mut velocity, transform) = player.into_inner();
    let (camera, camera_transform) = camera.into_inner();

    //println!("{:?}", transform);

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

    if let Some(hold) = control.hold {
        let Ok(hold_world) = camera.viewport_to_world_2d(camera_transform, hold) else {
            return;
        };

        let world_difference = hold_world - cursor_world;
        let player_position = transform.translation.xy();
        gizmos.line_2d(player_position, player_position - world_difference, BLUE);
    }

    if let Some(drop_from) = control.drop_from {
        let cursor_difference = {
            let d = drop_from - cursor;
            vec2(d.x, -d.y)
        };

        velocity.0 += cursor_difference * 0.05;
        velocity.0 = velocity.0.normalize() * velocity.0.length(); //.min(10.0);
    }
}

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
    const PLAYER_RADIUS: f32 = 0.05;

    let circle = meshes.add(Circle::new(PLAYER_RADIUS));

    commands.spawn((
        SolidBody,
        Aabb::from_min_max(vec3(-1.0, -1.0, 1.0), vec3(1.0, 1.0, 1.0)),
        Transform::from_xyz(1.0, 3.0, 1.0),
        Mesh2d(meshes.add(Rectangle::from_size(vec2(2.0, 2.0)))),
        Bounciness(0.75),
        MeshMaterial2d(materials.add(Color::Srgba(Srgba::BLUE))),
    ));

    commands.spawn((
        SolidBody,
        Aabb::from_min_max(vec3(-1.0, -1.0, 1.0), vec3(1.0, 1.0, 1.0)),
        Transform::from_xyz(4.0, 3.0, 1.0),
        Mesh2d(meshes.add(Rectangle::from_size(vec2(2.0, 2.0)))),
        Bounciness(0.5),
        MeshMaterial2d(materials.add(Color::Srgba(Srgba::BLUE))),
    ));

    commands.spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 1.0),
        Velocity(vec2(0.0, 0.0)),
        Mass(0.045),
        FrictionCoefficient(0.5),
        Aabb::from_min_max(
            vec3(-PLAYER_RADIUS / 2.0, -PLAYER_RADIUS / 2.0, 1.0),
            vec3(PLAYER_RADIUS / 2.0, PLAYER_RADIUS / 2.0, 1.0),
        ),
        Mesh2d(circle),
        MeshMaterial2d(materials.add(Color::Srgba(Srgba::RED))),
    ));

    let mut proj = OrthographicProjection::default_2d();
    proj.scale = 0.005;
    commands.spawn((MainCamera, Projection::Orthographic(proj), Bloom::default()));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(2000.0, 2000.0))),
        MeshMaterial2d(checkered_materials.add(CheckeredMaterial {
            tile_width: 1.5,
            tile_height: 1.5,
            color_a: WHITE.into(),
            color_b: BLACK.into(),
        })),
    ));
}
