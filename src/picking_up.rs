use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::player::PlayerFacing;

pub struct PickingUpPlugin;

#[derive(Component)]
pub struct IsPickable;

#[derive(InputAction, Debug)]
#[input_action(output = bool)]
pub struct PlayerPickUp;

impl Plugin for PickingUpPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(initiate_pick_up);
        app.add_systems(FixedUpdate, apply_forces_to_held_objects);
    }
}

#[derive(Default, Debug, Component)]
pub struct Picker {
    holding: Option<Entity>,
}

#[derive(Debug, Component)]
pub struct HeldBy(Entity);

#[derive(Debug, Component)]
pub enum HeldStatus {
    Lifted,
    // Carried,
    Placed(Dir2),
}

fn initiate_pick_up(
    trigger: Trigger<Started<PlayerPickUp>>,
    mut picker_query: Query<(&mut Picker, &Position, &PlayerFacing)>,
    pickable_filter: Query<(), (With<IsPickable>, Without<HeldStatus>)>,
    mut held_query: Query<(&HeldBy, &mut HeldStatus), With<IsPickable>>,
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
                picker.holding = None;
                break 'place_held_object;
            };
            if held_by.0 != picker_entity {
                warn!(
                    "{picker_entity} should be holding {held_entity} - but it's held by {}",
                    held_by.0
                );
                picker.holding = None;
                break 'place_held_object;
            }
            *held_status = HeldStatus::Placed(facing.direction_2d());
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
    picker.holding = Some(pickable_entity);
}

fn apply_forces_to_held_objects(
    mut held_query: Query<(
        Entity,
        &HeldBy,
        &mut HeldStatus,
        &Position,
        &LinearVelocity,
        &mut ExternalForce,
        &ComputedMass,
        &GravityScale,
    )>,
    mut picker_query: Query<(&mut Picker, &Position)>,
    mut commands: Commands,
    time: Res<Time>,
    gravity: Res<Gravity>,
) {
    for (
        held_entity,
        &HeldBy(picker_entity),
        mut held_status,
        held_position,
        linvel,
        mut force,
        mass,
        gravity_scale,
    ) in held_query.iter_mut()
    {
        force.clear();
        let anti_gravity = -gravity.0 * gravity_scale.0 * mass.value();
        let Ok((_picker, picker_position)) = picker_query.get_mut(picker_entity) else {
            commands
                .entity(held_entity)
                .remove::<(HeldBy, HeldStatus)>();
            continue;
        };
        let held_status = held_status.as_mut();
        match held_status {
            HeldStatus::Lifted => {
                let target_position = picker_position.0 + 3.0 * Vec2::Y;
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
                force.apply_force(desired_boost / time.delta_secs() + anti_gravity);
            }
            // HeldStatus::Carried(entity) => todo!(),
            HeldStatus::Placed(_dir) => todo!(),
        }
    }
}
