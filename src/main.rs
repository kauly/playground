use bevy::{prelude::*, render::view::RenderLayers};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use marks::{Ball, Floor, GameCamera, Player};
use simula_viz::{
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
};
use std::{f32::consts::FRAC_PI_3, time::Duration};

mod marks;

const BOARD_DIM: (f32, f32, f32) = (10.0, 0.1, 20.0);
const GOAL_GAP: f32 = 2.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(GridPlugin)
        .add_plugin(LinesPlugin)
        .add_startup_system(setup_system)
        .add_startup_system(setup_physics)
        .add_system(move_player)
        .add_system(player_kick)
        .run();
}

#[derive(Resource)]
struct KickTimer(Timer);

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
    line_mesh: Res<LineMesh>,
) {
    // lights
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.9, 1.0, 1.0),
        brightness: 0.14,
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 6000.,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-BOARD_DIM.0, 1.5, BOARD_DIM.2 * 0.5),
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.5, 5.0, -25.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        GameCamera,
    ));

    // grid
    let grid_color = Color::rgb(0.01, 0.01, 0.01);
    commands.spawn((
        GridBundle {
            grid: Grid {
                size: 20,
                divisions: 20,
                start_color: grid_color,
                end_color: grid_color,
                ..default()
            },
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
            transform: Transform::from_xyz(0.0, -2.0, 0.0),
            ..default()
        },
        Name::new("grid"),
    ));

    // kick timer resource
    commands.insert_resource(KickTimer(Timer::from_seconds(0.25, TimerMode::Once)));
}

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // create a bouncing ball
    let ball_texture = asset_server.load("textures/ball/ball.png");

    commands.spawn((
        Collider::ball(0.5),
        Restitution::coefficient(1.0),
        RigidBody::Dynamic,
        Damping {
            angular_damping: 1.0,
            linear_damping: 0.5,
        },
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.5,
                ..default()
            })),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(ball_texture),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 4.0, 0.0),
            ..default()
        },
        Ball,
        Name::new("ball"),
    ));

    // create a static floor
    commands.spawn((
        Collider::cuboid(BOARD_DIM.0 / 2.0, BOARD_DIM.1 / 2.0, BOARD_DIM.2 / 2.0),
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(
                BOARD_DIM.0,
                BOARD_DIM.1,
                BOARD_DIM.2,
            ))),
            material: materials.add(StandardMaterial {
                base_color: Color::BLACK.into(),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -2.0, 0.0),
            ..default()
        },
        Floor,
        Name::new("floor"),
    ));

    // spawn a player capsule
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule::default())),
            material: materials.add(StandardMaterial {
                base_color: Color::FUCHSIA,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.5, -(BOARD_DIM.2 / 2.0) + 0.5),
            ..default()
        },
        Collider::capsule_y(0.5, 0.5),
        RigidBody::KinematicPositionBased,
        Restitution::coefficient(1.5),
        KinematicCharacterController {
            autostep: None,
            ..default()
        },
        Player,
        Name::new("player"),
    ));

    // spawn a goal box
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 1.0, 1.0))),
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                ..default()
            }),
            transform: Transform::from_xyz(-GOAL_GAP, -1.5, (BOARD_DIM.2 / 2.0) - 1.0),
            ..default()
        },
        RigidBody::Fixed,
        Name::new("EnemyGoalRight"),
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 1.0, 1.0))),
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                ..default()
            }),
            transform: Transform::from_xyz(GOAL_GAP, -1.5, (BOARD_DIM.2 / 2.0) - 1.0),
            ..default()
        },
        RigidBody::Fixed,
        Name::new("EnemyGoalLeft"),
    ));
}

fn move_player(
    mut player_query: Query<&mut KinematicCharacterController, With<Player>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut player_ctrl = player_query.single_mut();
    let mut direction = Vec3::new(0.0, -1.5, 0.0);

    if keyboard.pressed(KeyCode::W) {
        direction += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::S) {
        direction -= Vec3::Z;
    }
    if keyboard.pressed(KeyCode::A) {
        direction += Vec3::X;
    }
    if keyboard.pressed(KeyCode::D) {
        direction -= Vec3::X;
    }

    player_ctrl.translation = Some(direction * time.delta_seconds() * 4.0);
}

fn player_kick(
    mut player_query: Query<&mut Transform, With<Player>>,
    mut kick_timer: ResMut<KickTimer>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut player_tf = player_query.single_mut();

    kick_timer
        .0
        .tick(Duration::from_secs_f32(time.delta_seconds()));

    if keyboard.just_pressed(KeyCode::Space) {
        player_tf.rotate_x(-FRAC_PI_3);
        kick_timer.0.reset();
    }

    if kick_timer.0.just_finished() && player_tf.rotation.x < 0.0 {
        player_tf.rotate_x(FRAC_PI_3);
    }
}
