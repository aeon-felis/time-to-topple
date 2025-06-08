use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_yoleck::vpeol_3d::Vpeol3dPosition;
use bevy_yoleck::{YoleckDirective, prelude::*};

use crate::picking_up::Pickable;
use crate::topple_detection::Toppleable;

pub struct BrickPlugin;

impl Plugin for BrickPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_entity_type({
            YoleckEntityType::new("Brick")
                .with::<Vpeol3dPosition>()
                .insert_on_init_during_editor(|| Dupable("Brick"))
                .insert_on_init(|| (IsBrick, Toppleable::Standing))
        });

        app.add_yoleck_entity_type({
            YoleckEntityType::new("PickableBrick")
                .with::<Vpeol3dPosition>()
                .insert_on_init(|| {
                    (
                        IsBrick,
                        Pickable {
                            hold_at_offset: -2.0 * Vec2::Y,
                        },
                        Toppleable::Standing,
                    )
                })
        });

        app.add_systems(YoleckSchedule::Populate, populate_brick);
        app.add_yoleck_edit_system(dup_buttons);
    }
}

#[derive(Component)]
pub struct IsBrick;

#[derive(Component)]
pub struct Dupable(&'static str);

fn populate_brick(
    mut populate: YoleckPopulate<Has<Pickable>, With<IsBrick>>,
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

fn dup_buttons(
    mut ui: ResMut<YoleckUi>,
    edit: YoleckEdit<(&YoleckBelongsToLevel, &Vpeol3dPosition, &Dupable)>,
    mut writer: EventWriter<YoleckDirective>,
) {
    let Ok((belongs_to_level, position, dupable)) = edit.single() else {
        return;
    };

    let mut make_button = |ui: &mut egui::Ui, label: &str, direction: Dir3| {
        if ui.button(label).clicked() {
            writer.write({
                YoleckDirective::spawn_entity(belongs_to_level.level, dupable.0, true)
                    .with(Vpeol3dPosition(position.0 + direction * 3.5))
                    .modify_exclusive_systems(|queue| queue.clear())
                    .into()
            });
        }
    };

    ui.horizontal(|ui| {
        make_button(ui, "<-", Dir3::NEG_X);
        ui.label("duplicate");
        make_button(ui, "->", Dir3::X);
    });
}
