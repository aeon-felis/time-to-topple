use avian2d::prelude::*;
use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_yoleck::prelude::*;
use bevy_yoleck::vpeol_3d::{Vpeol3dPosition, Vpeol3dRotation, Vpeol3dScale};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::utils::CachedPbrMaker;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_entity_type({
            YoleckEntityType::new("Block")
                .with::<Vpeol3dPosition>()
                .with::<Vpeol3dScale>()
                .with::<Vpeol3dRotation>()
                .with::<BlockFriction>()
                .insert_on_init(|| (IsBlock))
        });

        app.add_yoleck_edit_system(resize_block);
        app.add_yoleck_edit_system(rotate_block);
        app.add_yoleck_edit_system(set_block_friction);

        app.add_systems(YoleckSchedule::Populate, populate_block);
    }
}

#[derive(Component)]
pub struct IsBlock;

#[derive(Component, YoleckComponent, Serialize, Deserialize, PartialEq, Clone)]
struct BlockFriction(f32);

impl Default for BlockFriction {
    fn default() -> Self {
        Self(10.0)
    }
}

fn set_block_friction(mut ui: ResMut<YoleckUi>, mut edit: YoleckEdit<&mut BlockFriction>) {
    let Ok(mut friction) = edit.single_mut() else {
        return;
    };
    ui.add(egui::Slider::new(&mut friction.0, 0.0..=10.0).text("Friction"));
}

fn populate_block(
    mut populate: YoleckPopulate<&BlockFriction, With<IsBlock>>,
    mut pbr: CachedPbrMaker,
) {
    populate.populate(|ctx, mut cmd, BlockFriction(friction)| {
        if ctx.is_first_time() {
            cmd.insert(pbr.make_pbr_with(
                || Mesh::from(Cuboid::new(1.0, 1.0, 1.0)),
                || StandardMaterial::from_color(css::GRAY),
            ));
            cmd.insert(RigidBody::Static);
            cmd.insert(Collider::rectangle(1.0, 1.0));
        }
        if !ctx.is_in_editor() {
            cmd.insert(Friction::new(*friction));
        }
    });
}

fn resize_block(
    mut edit: YoleckEdit<
        (&Vpeol3dRotation, &mut Vpeol3dScale, &mut Vpeol3dPosition),
        With<IsBlock>,
    >,
    mut knobs: YoleckKnobs,
    mut pbr: CachedPbrMaker,
) {
    let Ok((rotation, mut scale, mut position)) = edit.single_mut() else {
        return;
    };

    let knob_pbr = pbr.make_pbr_with(
        || Mesh::from(Cuboid::new(0.4, 0.4, 1.1)),
        || StandardMaterial::from_color(css::ORANGE),
    );

    for (i, diagonal) in [
        Vec2::new(1.0, 1.0),
        Vec2::new(-1.0, 1.0),
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, -1.0),
    ]
    .into_iter()
    .enumerate()
    {
        let offset = 0.5 * diagonal * scale.0.truncate();
        let mut knob = knobs.knob(("resize-marker", i));
        if knob.is_new {
            knob.cmd.insert(knob_pbr.clone());
        }
        knob.cmd.insert(Transform::from_translation(
            position.0 + rotation.0 * offset.extend(0.0),
        ));

        if let Some(new_marker_pos) = knob.get_passed_data::<Vec3>() {
            let inverse_rotation = rotation.0.inverse();
            let other_corner = position.0 - (inverse_rotation * offset.extend(0.0));
            let size_f = (*new_marker_pos - other_corner).truncate();
            let size_f = size_f * diagonal;
            let size_f = Vec2::from_array(size_f.to_array().map(|coord| coord.max(0.0)));
            scale.0 = size_f.extend(1.0);
            position.0 = other_corner + 0.5 * (inverse_rotation * (diagonal * size_f).extend(0.0));
        }
    }
}

fn rotate_block(
    mut edit: YoleckEdit<
        (&mut Vpeol3dRotation, &Vpeol3dScale, &mut Vpeol3dPosition),
        With<IsBlock>,
    >,
    mut knobs: YoleckKnobs,
    mut pbr: CachedPbrMaker,
) {
    let Ok((mut rotation, scale, position)) = edit.single_mut() else {
        return;
    };

    let knob_pbr = pbr.make_pbr_with(
        || Mesh::from(Sphere::new(0.4)),
        || StandardMaterial::from_color(css::GREEN),
    );

    for (i, knob_direction) in [
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(-1.0, 0.0),
        Vec2::new(0.0, -1.0),
    ]
    .into_iter()
    .enumerate()
    {
        let offset = 0.5 * knob_direction * scale.0.truncate();
        let rotated_offset = rotation.0 * offset.extend(0.0);
        let mut knob = knobs.knob(("rotate-marker", i));
        if knob.is_new {
            knob.cmd.insert(knob_pbr.clone());
        }
        knob.cmd
            .insert(Transform::from_translation(position.0 + rotated_offset));

        if let Some(new_marker_pos) = knob.get_passed_data::<Vec3>() {
            let desired_direction = (*new_marker_pos - position.0)
                .truncate()
                .normalize_or_zero();
            let new_rotation = Quat::from_rotation_arc_2d(knob_direction, desired_direction);
            rotation.0 = new_rotation;
        }
    }
}

pub fn calculate_lowest_y(objects_query: Query<&GlobalTransform, With<IsBlock>>) -> Option<f32> {
    objects_query
        .iter()
        .flat_map(|transform| {
            static FOUR_CORNERS: [Vec3; 4] = [
                Vec3::new(0.5, 0.5, 0.0),
                Vec3::new(-0.5, 0.5, 0.0),
                Vec3::new(-0.5, -0.5, 0.0),
                Vec3::new(0.5, -0.5, 0.0),
            ];
            FOUR_CORNERS.map(|corner| transform.transform_point(corner).y)
        })
        .min_by_key(|y| OrderedFloat(*y))
}
