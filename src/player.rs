use super::marks::Player;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::FRAC_PI_2;
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_player).add_system(player_kick);
    }
}

fn move_player(
    mut player_query: Query<&mut KinematicCharacterController, With<Player>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut player_ctrl = player_query.single_mut();
    let mut direction = Vec3::new(0.0, 0.0, 0.0);

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
    keyboard: Res<Input<KeyCode>>,
) {
    let mut player_tf = player_query.single_mut();

    if keyboard.just_pressed(KeyCode::Space) {
        player_tf.rotate_x(-FRAC_PI_2);
    }

    if keyboard.just_released(KeyCode::Space) {
        player_tf.rotate_x(FRAC_PI_2);
    }
}
