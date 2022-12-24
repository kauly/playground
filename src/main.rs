use std::{
    f32::consts::{FRAC_PI_2, FRAC_PI_3},
    time::Duration,
};

use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*, render::view::RenderLayers};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::{
    prelude::*,
    rapier::prelude::{RigidBodyBuilder, RigidBodyType},
};
use marks::{Ball, Floor, GameCamera, Player};
use simula_action::ActionPlugin;
use simula_camera::{flycam::*, orbitcam::*};
use simula_video::rt;
use simula_viz::{
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
};

mod marks;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        //  .add_plugin(OrbitCameraPlugin)
        // .add_plugin(FlyCameraPlugin)
        .add_plugin(ActionPlugin)
        .add_plugin(GridPlugin)
        .add_plugin(LinesPlugin)
        .add_startup_system(setup_system)
        .add_startup_system(setup_physics)
        .add_system(move_player)
        //   .add_system(camera_follow)
        .run();
}

#[derive(Resource)]
struct KickTimer(Timer);

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
    mut images: ResMut<Assets<Image>>,
    line_mesh: Res<LineMesh>,
) {
    // lights
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.9, 1.0, 1.0),
        brightness: 0.14,
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3000.,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(1.5, 1.5, -2.5),
        ..default()
    });

    // camera
    let rt_image = images.add(rt::common_render_target_image(UVec2::new(256, 256)));
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, -10.0)
                .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            ..default()
        },
        // RenderLayers::all(),
        // FlyCamera::default(),
        GameCamera,
    ));
    /*         .with_children(|parent| {
        parent.spawn((Camera3dBundle {
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            camera: Camera {
                priority: -1,
                target: bevy::render::camera::RenderTarget::Image(rt_image.clone()),
                ..default()
            },
            ..default()
        },));
    }); */

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
        Collider::cuboid(10., 0., 10.),
        RigidBody::Fixed,
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
        Collider::cuboid(10., 0.1, 10.),
        RigidBody::Fixed,
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(10., 0.1, 20.))),
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
            transform: Transform::from_xyz(0.0, 0.5, -8.0),
            ..default()
        },
        Collider::capsule_y(0.5, 0.5),
        RigidBody::KinematicPositionBased,
        Restitution::coefficient(1.5),
        KinematicCharacterController::default(),
        Player,
        Name::new("player"),
    ));
}

fn move_player(
    mut player_query: Query<(&mut KinematicCharacterController, &mut Transform), With<Player>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut kick_timer: ResMut<KickTimer>,
) {
    let (mut player_ctrl, mut player_tf) = player_query.single_mut();
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

    if keyboard.just_pressed(KeyCode::Space) {
        player_tf.rotate_x(-FRAC_PI_3);
        kick_timer.0.reset();
    }

    kick_timer
        .0
        .tick(Duration::from_secs_f32(time.delta_seconds()));

    if kick_timer.0.just_finished() && player_tf.rotation.x < 0. {
        player_tf.rotate_x(FRAC_PI_3);
    }

    player_ctrl.translation = Some(direction * time.delta_seconds() * 2.5);
}

fn camera_follow(
    mut camera_query: Query<&mut Transform, (With<GameCamera>, Without<Player>)>,
    player_query: Query<&KinematicCharacterController, (With<Player>, Without<GameCamera>)>,
) {
    let mut cam_transform = camera_query.single_mut();
    if let Ok(player_controller) = player_query.get_single() {
        if let Some(translation) = player_controller.translation {
            cam_transform.translation = translation;
        }
    }
}
