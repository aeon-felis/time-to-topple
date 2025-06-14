use avian2d::prelude::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::camera::CameraTarget;
use crate::player::PlayerFacing;

pub struct PickingUpPlugin;

#[derive(Component)]
pub struct Pickable {
    pub hold_at_offset: Vec2,
}

#[derive(InputAction, Debug)]
#[input_action(output = bool)]
pub struct PlayerPickUp;

impl Plugin for PickingUpPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(initiate_pick_up);
        app.add_systems(
            FixedUpdate,
            (
                apply_forces_to_held_objects,
                InitialCollisions::initiate,
                InitialCollisions::clear,
                break_hold_when_pickable_collides_with_something
                    .after(apply_forces_to_held_objects),
            ),
        );
    }
}

#[derive(Default, Debug, Component)]
pub struct Picker {
    holding: Option<Entity>,
    pub immobilized: bool,
}

impl Picker {
    fn clear(&mut self) {
        *self = Default::default();
    }
}

pub const PICKER_OFFSET: Vec2 = Vec2::new(0.0, 3.0);

#[derive(Debug, Component)]
pub struct HeldBy(Entity);

#[derive(Debug, Component)]
pub enum HeldStatus {
    Lifted,
    Carried,
    Placed(Dir2),
}

fn initiate_pick_up(
    trigger: Trigger<Started<PlayerPickUp>>,
    mut picker_query: Query<
        (&mut Picker, &Position, &PlayerFacing),
        // When we lose camera target that means the toppling has begun - and we no longer
        // want to allow the player to pick bricks.
        With<CameraTarget>,
    >,
    pickable_filter: Query<(), (With<Pickable>, Without<HeldStatus>)>,
    mut held_query: Query<(&HeldBy, &mut HeldStatus), With<Pickable>>,
    spatial_query: Res<SpatialQueryPipeline>,
    mut commands: Commands,
) {
    let picker_entity = trigger.target();
    let Ok((mut picker, picker_position, facing)) = picker_query.get_mut(picker_entity) else {
        return;
    };
    'place_held_object: {
        if let Some(held_entity) = picker.holding {
            let Ok((held_by, mut held_status)) = held_query.get_mut(held_entity) else {
                warn!(
                    "{picker_entity} should be holding {held_entity} - but its query returns nothing"
                );
                picker.clear();
                break 'place_held_object;
            };
            if held_by.0 != picker_entity {
                warn!(
                    "{picker_entity} should be holding {held_entity} - but it's held by {}",
                    held_by.0
                );
                picker.clear();
                break 'place_held_object;
            }
            *held_status = HeldStatus::Placed(facing.direction_2d());
            picker.immobilized = true;
            return;
        }
    }
    let Some(hit) = spatial_query.cast_shape_predicate(
        &Collider::rectangle(0.0, 0.5),
        picker_position.0,
        0.0,
        facing.direction_2d(),
        &ShapeCastConfig {
            max_distance: 2.0,
            ..Default::default()
        },
        &Default::default(),
        &|entity| pickable_filter.contains(entity),
    ) else {
        return;
    };
    let pickable_entity = hit.entity;
    commands
        .entity(pickable_entity)
        .insert((HeldBy(picker_entity), HeldStatus::Lifted));
    *picker = Picker {
        holding: Some(pickable_entity),
        immobilized: true,
    }
}

fn apply_forces_to_held_objects(
    mut held_query: Query<(
        Entity,
        &Pickable,
        &HeldBy,
        &mut HeldStatus,
        &Position,
        &Rotation,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut ExternalForce,
        &ComputedMass,
        &GravityScale,
    )>,
    mut picker_query: Query<(&mut Picker, &Position, &LinearVelocity), Without<Pickable>>,
    mut commands: Commands,
    time: Res<Time>,
    gravity: Res<Gravity>,
) {
    for (
        held_entity,
        pickable,
        &HeldBy(picker_entity),
        mut held_status,
        held_position,
        held_rotation,
        mut linvel,
        mut angvel,
        mut force,
        mass,
        gravity_scale,
    ) in held_query.iter_mut()
    {
        let anti_gravity = -gravity.0 * gravity_scale.0 * mass.value();
        force.set_force(anti_gravity);
        let Ok((mut picker, picker_position, picker_velocity)) =
            picker_query.get_mut(picker_entity)
        else {
            commands
                .entity(held_entity)
                .remove::<(HeldBy, HeldStatus)>();
            continue;
        };
        let held_status = held_status.as_mut();

        let angle_to_add = held_rotation.angle_between(Rotation::IDENTITY);
        angvel.0 = angle_to_add / time.delta_secs();

        match held_status {
            HeldStatus::Lifted => {
                let target_position = picker_position.0 + PICKER_OFFSET - pickable.hold_at_offset;
                let vec_to_target = target_position - held_position.0;
                const LIFT_SPEED: f32 = 10.0;
                let desired_velocity = if 0.5 < vec_to_target.y {
                    LIFT_SPEED * Vec2::Y
                } else if 2.0 * LIFT_SPEED * time.delta_secs() < vec_to_target.x.abs() {
                    vec_to_target.clamp_length(10.0, 10.0)
                } else {
                    vec_to_target.clamp_length_max(10.0)
                };
                let desired_boost = desired_velocity - linvel.0;
                force.apply_force(desired_boost / time.delta_secs());
                if vec_to_target.length_squared() < 0.1 {
                    *held_status = HeldStatus::Carried;
                    picker.immobilized = false;
                }
            }
            HeldStatus::Carried => {
                let target_position = picker_position.0 + PICKER_OFFSET - pickable.hold_at_offset;
                let vec_to_target = target_position - held_position.0;
                let desired_velocity = 0.5 * vec_to_target / time.delta_secs() + picker_velocity.0; //.clamp_length_max(40.0);
                let desired_boost = desired_velocity - linvel.0;
                force.apply_force(desired_boost / time.delta_secs());
            }
            HeldStatus::Placed(dir) => {
                let target_position =
                    picker_position.0 + 1.5 * **dir - pickable.hold_at_offset - 1.0 * Vec2::Y;
                let vec_to_target = target_position - held_position.0;
                const LIFT_SPEED: f32 = 10.0;
                let desired_velocity = if vec_to_target.dot(**dir) < 0.5 {
                    -LIFT_SPEED * Vec2::Y
                } else if 2.0 * LIFT_SPEED * time.delta_secs() < vec_to_target.x.abs() {
                    vec_to_target.clamp_length(10.0, 10.0)
                } else {
                    vec_to_target.clamp_length_max(10.0)
                };
                let desired_boost = desired_velocity - linvel.0;
                force.apply_force(desired_boost / time.delta_secs());
                if vec_to_target.length_squared() < 0.1 {
                    commands
                        .entity(held_entity)
                        .remove::<(HeldBy, HeldStatus)>();
                    linvel.0 = Vec2::ZERO;
                    force.clear();
                    picker.clear();
                    // *held_status = HeldStatus::Carried;
                    // picker.immobilized = false;
                }
            }
        }
    }
}

#[derive(Component)]
struct InitialCollisions(HashMap<Entity, bool>);

impl InitialCollisions {
    fn initiate(
        query: Query<Entity, (With<HeldBy>, Without<InitialCollisions>)>,
        collisions: Collisions,
        mut commands: Commands,
    ) {
        for held_entity in query.iter() {
            commands.entity(held_entity).insert(InitialCollisions(
                collisions
                    .entities_colliding_with(held_entity)
                    .map(|e| (e, true))
                    .collect(),
            ));
        }
    }

    fn clear(
        query: Query<Entity, (Without<HeldBy>, With<InitialCollisions>)>,
        mut commands: Commands,
    ) {
        for no_longer_held_entity in query.iter() {
            commands
                .entity(no_longer_held_entity)
                .try_remove::<InitialCollisions>();
        }
    }
}

fn break_hold_when_pickable_collides_with_something(
    mut pickable_query: Query<(Entity, &HeldBy, &mut InitialCollisions, &mut ExternalForce)>,
    mut picker_query: Query<&mut Picker>,
    collisions: Collisions,
    mut commands: Commands,
) {
    for (pickable_entity, &HeldBy(held_by_entity), mut initial_collisions, mut force) in
        pickable_query.iter_mut()
    {
        for still_touching_this_step in initial_collisions.0.values_mut() {
            *still_touching_this_step = false;
        }
        let mut any_colliding = false;
        for colliding_entity in collisions.entities_colliding_with(pickable_entity) {
            if colliding_entity == held_by_entity {
                continue;
            }
            if let Some(still_touching_this_step) = initial_collisions.0.get_mut(&colliding_entity)
            {
                *still_touching_this_step = true;
                continue;
            }
            any_colliding = true;
        }

        if any_colliding {
            force.clear();
            commands
                .entity(pickable_entity)
                .remove::<(HeldBy, HeldStatus)>();
            if let Ok(mut picker) = picker_query.get_mut(held_by_entity) {
                if picker.holding == Some(pickable_entity) {
                    picker.clear();
                }
            }
        }
    }
}
