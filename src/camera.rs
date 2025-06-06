use bevy::prelude::*;
use bevy_yoleck::vpeol::prelude::*;
use dolly::prelude::*;

use crate::AppState;
use crate::player::PlayerFacing;

pub struct TimeToToppleCameraPlugin;

impl Plugin for TimeToToppleCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        //app.add_systems(Update, apply_dolly_camera_controls.in_set(During::Gameplay));
        app.add_systems(
            Update,
            apply_dolly_camera_controls.run_if(|state: Res<State<AppState>>| {
                matches!(**state, AppState::Game | AppState::GameOver)
            }),
        );
    }
}

#[derive(Component)]
pub struct CameraTarget;

#[derive(Component)]
struct CameraController(CameraRig);

fn setup_camera(mut commands: Commands) {
    let mut cmd = commands.spawn_empty();
    cmd.insert(Camera3d::default());
    cmd.insert(
        Transform::from_xyz(0.0, 3.0, 100.0).looking_to(Vec3::new(0.0, -3.0, -10.0), Vec3::Y),
    );
    cmd.insert(VpeolCameraState::default());
    cmd.insert(Vpeol3dCameraControl::sidescroller());
    cmd.insert(CameraController(
        CameraRig::builder()
            .with(Position::default())
            .with(Arm::new([0.0, 10.0, 50.0]))
            .with(Smooth::new_position(10.0))
            .with(LookAt::new(Vec3::ZERO.to_array()).tracking_smoothness(5.0))
            .build(),
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 50_000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(0.0, 1000.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn apply_dolly_camera_controls(
    time: Res<Time>,
    mut camera_query: Query<(&mut CameraController, &mut Transform)>,
    target_query: Query<(&GlobalTransform, Option<&PlayerFacing>), With<CameraTarget>>,
) {
    let Ok((target_transform, target_facing)) = target_query.single() else {
        return;
    };
    let target_position = target_transform.translation();
    for (mut camera_controller, mut camera_transform) in camera_query.iter_mut() {
        camera_controller.0.driver_mut::<Position>().position = target_position.to_array().into();
        camera_controller.0.driver_mut::<LookAt>().target = (target_position
            + 3.0 * Vec3::Y
            + match target_facing {
                Some(facing) => 4.0 * facing.direction(),
                None => Vec3::ZERO,
            })
        .to_array()
        .into();
        camera_controller.0.update(time.delta_secs());
        camera_transform.translation =
            Vec3::from_array(camera_controller.0.final_transform.position.into());
        camera_transform.rotation =
            Quat::from_array(camera_controller.0.final_transform.rotation.into());
    }
}
