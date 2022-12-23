use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*, render::view::RenderLayers};
use bevy_egui::EguiPlugin;
use bevy_rapier3d::prelude::*;
use simula_action::ActionPlugin;
use simula_camera::{flycam::*, orbitcam::*};
use simula_video::rt;
use simula_viz::{
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //   .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(FlyCameraPlugin)
        .add_plugin(ActionPlugin)
        .add_plugin(GridPlugin)
        .add_plugin(LinesPlugin)
        .add_startup_system(setup_system)
        .add_startup_system(setup_physics)
        // .add_system(print_ball_altitude)
        .run();
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.0, -10.0)
                    .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
                ..default()
            },
            RenderLayers::all(),
            FlyCamera::default(),
        ))
        .with_children(|parent| {
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
        });

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
    ));
}

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    {
        let ball_texture = asset_server.load("textures/texture2.png");
        // create a bouncing ball
        commands.spawn((
            Collider::ball(0.5),
            RigidBody::Dynamic,
            Restitution::coefficient(0.7),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: 0.5,
                    subdivisions: 20,
                })),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(ball_texture),
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, 4.0, 0.0),
                ..default()
            },
        ));
    }

    {
        let ball_texture = asset_server.load("textures/texture.png");
        // create a bouncing ball
        commands.spawn((
            Collider::ball(0.5),
            RigidBody::Dynamic,
            Restitution::coefficient(0.7),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: 0.5,
                    subdivisions: 20,
                })),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(ball_texture),
                    ..default()
                }),
                transform: Transform::from_xyz(2.0, 4.0, 0.0),
                ..default()
            },
        ));
    }
}

// a query that will print the ball altitude every frame
/* fn print_ball_altitude(query: Query<&Transform, With<RigidBody>>) {
    if let Ok(ball_transform) = query.get_single() {
        println!("Ball altitude: {}", ball_transform.translation.y);
    }
}
 */
