mod camera;
mod level_handling;
mod menu;
mod player;

use bevy::prelude::*;
use bevy_yoleck::prelude::YoleckSyncWithEditorState;

use self::camera::TimeToToppleCameraPlugin;
use self::level_handling::{LevelHandlingPlugin, LevelProgress};
use self::menu::MenuPlugin;

pub struct TimeToTopplePlugin {
    pub is_editor: bool,
    pub start_at_level: Option<String>,
}

impl Plugin for TimeToTopplePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                During::Menu.run_if(|state: Res<State<AppState>>| state.is_menu()),
                During::Gameplay.run_if(in_state(AppState::Game)),
            ),
        );
        app.insert_state(AppState::MainMenu);
        app.add_plugins(TimeToToppleCameraPlugin);
        if self.is_editor {
            app.add_plugins(YoleckSyncWithEditorState {
                when_editor: AppState::Editor,
                when_game: AppState::Game,
            });
        } else {
            app.add_plugins(MenuPlugin);
            app.add_plugins(LevelHandlingPlugin);
            if let Some(start_at_level) = &self.start_at_level {
                let start_at_level = if start_at_level.ends_with(".yol") {
                    start_at_level.clone()
                } else {
                    format!("{}.yol", start_at_level)
                };
                app.add_systems(
                    Startup,
                    move |mut level_progress: ResMut<LevelProgress>,
                          mut app_state: ResMut<NextState<AppState>>| {
                        level_progress.current_level = Some(start_at_level.clone());
                        app_state.set(AppState::LoadLevel);
                    },
                );
            }
        }
        // app.add_plugins(AnimatingPlugin);
        // app.add_plugins(PlayerPlugin);
        // app.add_plugins(ArenaPlugin);
        // app.add_plugins(PlayerControlsPlugin);
        // app.add_plugins(CannonPlugin);
        // app.add_plugins(MissilePlugin);
        // app.add_plugins(ExplosionPlugin);
        // app.add_plugins(DoorPlugin);
        // app.add_plugins(ArrowPlugin);
        //app.add_plugins(FloatingTextPlugin);

        app.add_systems(Update, enable_disable_physics);
    }
}

#[derive(SystemSet, Clone, PartialEq, Eq, Debug, Hash)]
pub enum During {
    Menu,
    Gameplay,
}

#[derive(States, Clone, Hash, Debug, PartialEq, Eq)]
pub enum AppState {
    MainMenu,
    PauseMenu,
    LevelSelectMenu,
    LoadLevel,
    Editor,
    Game,
    LevelCompleted,
    GameOver,
}

impl AppState {
    pub fn is_menu(&self) -> bool {
        match self {
            AppState::MainMenu => true,
            AppState::PauseMenu => true,
            AppState::LevelSelectMenu => true,
            AppState::LoadLevel => false,
            AppState::Editor => false,
            AppState::Game => false,
            AppState::LevelCompleted => false,
            AppState::GameOver => true,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum ActionForKbgp {
    Menu,
    RestartLevel,
}

fn enable_disable_physics(
    state: Res<State<AppState>>,
    mut avian_time: ResMut<Time<avian2d::schedule::Physics>>,
) {
    use avian2d::schedule::PhysicsTime;
    if matches!(state.get(), AppState::Game) {
        avian_time.unpause();
    } else {
        avian_time.pause();
    }
}
