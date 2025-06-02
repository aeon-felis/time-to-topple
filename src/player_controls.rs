use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_tnua::prelude::*;
use bevy_yoleck::prelude::*;

use crate::During;
use crate::player::{IsPlayer, PlayerFacing};

#[derive(InputAction, Debug)]
#[input_action(output = f32)]
struct PlayerRun;

#[derive(InputAction, Debug)]
#[input_action(output = bool)]
struct PlayerJump;

#[derive(InputContext)]
struct PlayerOnFoot;

pub struct PlayerControlsPlugin;

impl Plugin for PlayerControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<PlayerOnFoot>();
        app.add_systems(YoleckSchedule::Populate, add_controls_to_player);
        app.add_systems(
            FixedUpdate,
            apply_controls
                .in_set(TnuaUserControlsSystemSet)
                .in_set(During::Gameplay),
        );
    }
}

fn add_controls_to_player(mut populate: YoleckPopulate<(), With<IsPlayer>>) {
    populate.populate(|ctx, mut cmd, ()| {
        if ctx.is_in_editor() {
            return;
        }
        let mut input_map = Actions::<PlayerOnFoot>::default();

        input_map.bind::<PlayerRun>().to((
            Cardinal::arrow_keys(),
            Cardinal::wasd_keys(),
            Cardinal::dpad_buttons(),
            Axial::left_stick(),
        ));

        input_map
            .bind::<PlayerJump>()
            .to((KeyCode::Space, KeyCode::KeyJ, GamepadButton::South));
        cmd.insert(input_map);
    });
}

fn apply_controls(
    // time: Res<Time>,
    mut query: Query<(
        &Actions<PlayerOnFoot>,
        &mut TnuaController,
        &mut PlayerFacing,
    )>,
) {
    for (input, mut controller, mut player_facing) in query.iter_mut() {
        let controller = controller.as_mut();
        let x_input = input.value::<PlayerRun>().unwrap().as_axis1d();
        let desired_velocity = Vec3::X * 20.0 * x_input;

        if x_input <= -0.1 {
            *player_facing = PlayerFacing::Left;
        } else if 0.1 <= x_input {
            *player_facing = PlayerFacing::Right;
        }
        controller.basis(TnuaBuiltinWalk {
            desired_velocity,
            float_height: 1.5,
            cling_distance: 0.5,
            ..Default::default()
        });
        if input.state::<PlayerJump>().unwrap() == ActionState::Fired {
            controller.action(TnuaBuiltinJump {
                height: 5.0,
                allow_in_air: false,
                ..Default::default()
            });
        }
    }
}
