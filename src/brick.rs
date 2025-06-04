use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_yoleck::prelude::*;
use bevy_yoleck::vpeol_3d::Vpeol3dPosition;

use crate::picking_up::IsPickable;

pub struct BrickPlugin;

impl Plugin for BrickPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_entity_type({
            YoleckEntityType::new("Brick")
                .with::<Vpeol3dPosition>()
                .insert_on_init(|| (IsBrick))
        });

        app.add_yoleck_entity_type({
            YoleckEntityType::new("PickableBrick")
                .with::<Vpeol3dPosition>()
                .insert_on_init(|| (IsBrick, IsPickable))
        });

        app.add_systems(YoleckSchedule::Populate, populate_brick);
    }
}

#[derive(Component)]
pub struct IsBrick;

fn populate_brick(
    mut populate: YoleckPopulate<Has<IsPickable>, With<IsBrick>>,
    asset_server: Res<AssetServer>,
) {
    populate.populate(|ctx, mut cmd, pickable| {
        if ctx.is_first_time() {
            cmd.insert(bevy_yoleck::vpeol::VpeolWillContainClickableChildren);
            cmd.insert(SceneRoot(asset_server.load(if pickable {
                "PickableBrick.glb#Scene0"
            } else {
                "Brick.glb#Scene0"
            })));
        }
        cmd.insert(RigidBody::Dynamic);
        cmd.insert(Collider::rectangle(0.2, 4.0));
        cmd.insert(Friction::new(0.1));
        cmd.insert(Mass(10.0));
        cmd.insert(GravityScale(5.0));
    });
}
