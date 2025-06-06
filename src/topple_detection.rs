use avian2d::prelude::*;
use bevy::prelude::*;

use crate::arena::calculate_lowest_y;
use crate::camera::CameraTarget;
use crate::{AppState, During, GameOverReason};

pub struct ToppleDetectionPlugin;

impl Plugin for ToppleDetectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                update_toppleable,
                detect_finish,
                calculate_lowest_y.pipe(detect_toppleables_who_fell_out),
            )
                .in_set(During::Gameplay),
        );
    }
}

#[derive(Debug, Component)]
pub enum Toppleable {
    Standing,
    Falling { immobile_timer: Timer },
    Stopped,
    FellOut,
}

fn update_toppleable(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Toppleable,
        &Rotation,
        &LinearVelocity,
        &AngularVelocity,
    )>,
    camera_target_query: Query<Entity, With<CameraTarget>>,
    mut commands: Commands,
) {
    for (toppleable_entity, mut toppleable, rotation, linvel, angvel) in query.iter_mut() {
        match toppleable.as_mut() {
            Toppleable::Standing => {
                if 0.1 < rotation.sin.abs() {
                    *toppleable = Toppleable::Falling {
                        immobile_timer: Timer::from_seconds(1.0, TimerMode::Once),
                    };
                    for entity in camera_target_query.iter() {
                        commands.entity(entity).remove::<CameraTarget>();
                    }
                    commands.entity(toppleable_entity).insert(CameraTarget);
                }
            }
            Toppleable::Falling { immobile_timer } => {
                if 0.01 < linvel.length_squared() || 0.01 < angvel.abs() {
                    immobile_timer.reset();
                } else if immobile_timer.tick(time.delta()).just_finished() {
                    *toppleable = Toppleable::Stopped;
                }
            }
            Toppleable::Stopped | Toppleable::FellOut => {}
        }
    }
}

fn detect_toppleables_who_fell_out(
    lowest_y: In<Option<f32>>,
    mut query: Query<(&mut Toppleable, &Position)>,
) {
    let Some(lowest_y) = *lowest_y else { return };
    for (mut toppleable, position) in query.iter_mut() {
        if position.y < lowest_y {
            *toppleable = Toppleable::FellOut;
        }
    }
}

fn detect_finish(
    query: Query<(Entity, &Toppleable)>,
    camera_target_query: Query<Entity, With<CameraTarget>>,
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_over_reason: ResMut<GameOverReason>,
) {
    let mut any_standing = None;
    let mut num_still_standing = 0;
    let mut all_standing = true;
    for (entity, toppleable) in query.iter() {
        match toppleable {
            Toppleable::Standing => {
                any_standing = Some(entity);
                num_still_standing += 1;
            }
            Toppleable::Falling { .. } => {
                // No need to set `all_standing = false;` since nothing will look at it.
                return;
            }
            Toppleable::Stopped | Toppleable::FellOut => {
                all_standing = false;
            }
        }
    }
    if all_standing {
        return;
    }

    if let Some(standing_entity) = any_standing {
        for entity in camera_target_query.iter() {
            commands.entity(entity).remove::<CameraTarget>();
        }
        commands.entity(standing_entity).insert(CameraTarget);
        *game_over_reason = GameOverReason::TilesStillStanding(num_still_standing);
        app_state.set(AppState::GameOver);
    } else {
        app_state.set(AppState::LevelCompleted);
    }
}
