use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::{TnuaAnimatingState, prelude::*};
use bevy_tnua_avian2d::TnuaAvian2dSensorShape;
use bevy_yoleck::prelude::*;
use bevy_yoleck::vpeol::VpeolWillContainClickableChildren;
use bevy_yoleck::vpeol_3d::Vpeol3dPosition;

use crate::arena::calculate_lowest_y;
use crate::camera::CameraTarget;
use crate::picking_up::Picker;
use crate::{AppState, During, GameOverReason};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_entity_type({
            YoleckEntityType::new("Player")
                .with::<Vpeol3dPosition>()
                .insert_on_init(|| (IsPlayer, CameraTarget))
        });
        app.add_systems(YoleckSchedule::Populate, populate_player);
        app.add_systems(
            Update,
            (
                set_player_facing,
                #[cfg(any())]
                animate_player,
            )
                .in_set(During::Gameplay),
        );
        app.add_systems(
            Update,
            calculate_lowest_y
                .pipe(kill_player_when_they_fall)
                .in_set(During::Gameplay),
        );
    }
}

#[derive(Component)]
pub struct IsPlayer;

#[derive(Component, Debug)]
pub enum PlayerFacing {
    Left,
    Right,
}

impl PlayerFacing {
    pub fn direction(&self) -> Dir3 {
        match self {
            PlayerFacing::Left => Dir3::NEG_X,
            PlayerFacing::Right => Dir3::X,
        }
    }

    pub fn direction_2d(&self) -> Dir2 {
        match self {
            PlayerFacing::Left => Dir2::NEG_X,
            PlayerFacing::Right => Dir2::X,
        }
    }
}

#[derive(Component)]
struct RotationBasedOn(Entity);

fn populate_player(
    mut populate: YoleckPopulate<(), With<IsPlayer>>,
    asset_server: Res<AssetServer>,
) {
    populate.populate(|ctx, mut cmd, ()| {
        if ctx.is_first_time() {
            cmd.insert(VpeolWillContainClickableChildren);
            let rotation_based_on = RotationBasedOn(cmd.id());
            let child = cmd
                .commands()
                .spawn((
                    SceneRoot(asset_server.load("Player.glb#Scene0")),
                    rotation_based_on,
                ))
                .id();
            cmd.add_child(child);
            // cmd.insert(ApplyRotationToChild(child));
            // cmd.insert(AnimationsOwner::default());
            // cmd.insert(GetClipsFrom(asset_server.load("Player.glb")));
        }
        // cmd.insert(VisibilityBundle::default());
        cmd.insert(RigidBody::Dynamic);
        // cmd.insert(Velocity::default());
        cmd.insert(Collider::capsule(0.5, 0.5));
        cmd.insert(Friction::ZERO);

        cmd.insert(TnuaController::default());
        cmd.insert(LockedAxes::ROTATION_LOCKED);
        cmd.insert(TnuaAvian2dSensorShape(Collider::rectangle(0.45, 0.0)));
        // cmd.insert(ActiveEvents::COLLISION_EVENTS);
        // cmd.insert(SolverGroups {
        // memberships: crate::solver_groups::PLAYER,
        // filters: crate::solver_groups::PLANTED,
        // });

        cmd.insert(PlayerFacing::Right);

        // cmd.insert(Killable::default());
        cmd.insert(TnuaAnimatingState::<PlayerAnimationState>::default());

        cmd.insert(Picker::default());
    });
}

fn set_player_facing(
    mut query: Query<(&mut Transform, &RotationBasedOn)>,
    players_query: Query<&PlayerFacing>,
) {
    for (mut transform, rotation_based_on) in query.iter_mut() {
        let Ok(facing) = players_query.get(rotation_based_on.0) else {
            continue;
        };
        transform.look_at(*facing.direction(), Vec3::Y);
    }
}

#[derive(Debug)]
#[allow(unused)]
pub enum PlayerAnimationState {
    Standing,
    Running(f32),
    Jumping,
    AirJumping,
    Dashing,
}

#[cfg(any())]
fn animate_player(
    mut query: Query<(
        &mut TnuaAnimatingState<PlayerAnimationState>,
        &TnuaController,
        &AnimationsOwner,
    )>,
    mut animation_players_query: Query<&mut AnimationPlayer>,
) {
    for (mut animating_state, controller, animations_owner) in query.iter_mut() {
        let Some(animation_player) = animations_owner.players.get("Armature") else {
            continue;
        };
        let Ok(mut animation_player) = animation_players_query.get_mut(*animation_player) else {
            continue;
        };
        match animating_state.update_by_discriminant({
            match controller.action_name() {
                Some(TnuaBuiltinJump::NAME) => PlayerAnimationState::Jumping,
                Some("air-jump") => PlayerAnimationState::AirJumping,
                Some(TnuaBuiltinDash::NAME) => PlayerAnimationState::Dashing,
                Some(name) => panic!("Unknown action {name}"),
                None => {
                    let Some((_, walk_state)) = controller.concrete_basis::<TnuaBuiltinWalk>()
                    else {
                        continue;
                    };
                    let speed = walk_state.running_velocity.length();
                    if 0.1 < speed {
                        PlayerAnimationState::Running(0.35 * speed)
                    } else {
                        PlayerAnimationState::Standing
                    }
                }
            }
        }) {
            TnuaAnimatingStateDirective::Maintain { state } => {
                if let PlayerAnimationState::Running(speed) = state {
                    animation_player.set_speed(*speed);
                }
            }
            bevy_tnua::TnuaAnimatingStateDirective::Alter {
                old_state: _,
                state,
            } => match state {
                PlayerAnimationState::Standing => {
                    let Some(clip) = animations_owner.clips.get("Stand") else {
                        continue;
                    };
                    animation_player
                        .play_with_transition(clip.clone(), Duration::from_secs_f32(0.25))
                        .set_speed(1.0);
                }
                PlayerAnimationState::Running(speed) => {
                    let Some(clip) = animations_owner.clips.get("Walk") else {
                        continue;
                    };
                    animation_player
                        .play(clip.clone())
                        .repeat()
                        .set_speed(*speed);
                }
                PlayerAnimationState::Jumping => {
                    let Some(clip) = animations_owner.clips.get("Jump") else {
                        continue;
                    };
                    animation_player.play(clip.clone()).set_speed(3.0);
                }
                PlayerAnimationState::AirJumping => {
                    let Some(clip) = animations_owner.clips.get("AirJump") else {
                        continue;
                    };
                    animation_player.play(clip.clone()).repeat().set_speed(3.0);
                }
                PlayerAnimationState::Dashing => {
                    let Some(clip) = animations_owner.clips.get("Dash") else {
                        continue;
                    };
                    animation_player.play(clip.clone());
                }
            },
        }
    }
}

fn kill_player_when_they_fall(
    lowest_y: In<Option<f32>>,
    players_query: Query<&GlobalTransform, With<IsPlayer>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_over_reason: ResMut<GameOverReason>,
) {
    let Some(lowest_y) = *lowest_y else { return };
    for player_transform in players_query.iter() {
        if player_transform.translation().y < lowest_y {
            *game_over_reason = GameOverReason::PlayerFell;
            app_state.set(AppState::GameOver);
        }
    }
}
