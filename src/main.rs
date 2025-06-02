use avian2d::PhysicsPlugins;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_egui_kbgp::prelude::*;
use bevy_pkv::PkvStore;
use bevy_skein::SkeinPlugin;
use bevy_tnua::prelude::TnuaControllerPlugin;
use bevy_tnua_avian2d::TnuaAvian2dPlugin;
use bevy_yoleck::vpeol_3d::{Vpeol3dPluginForEditor, Vpeol3dPluginForGame};
use bevy_yoleck::{YoleckPluginForEditor, YoleckPluginForGame};
use clap::Parser;
use time_to_topple::{ActionForKbgp, TimeToTopplePlugin};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long)]
    editor: bool,
    #[clap(long)]
    level: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        // Wasm builds will check for meta files (that don't exist) if this isn't set.
        // This causes errors and even panics in web builds on itch.
        // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
        meta_check: AssetMetaCheck::Never,
        ..default()
    }));

    app.add_plugins(EguiPlugin {
        enable_multipass_for_primary_context: true,
    });
    app.add_plugins((
        PhysicsPlugins::default(),
        TnuaControllerPlugin::new(FixedUpdate),
        TnuaAvian2dPlugin::new(FixedUpdate),
    ));

    app.insert_resource(PkvStore::new("AeonFelis", "TimeToTopple"));

    // app.add_plugins(RngPlugin::default());

    if args.editor {
        app.add_plugins((YoleckPluginForEditor, Vpeol3dPluginForEditor::topdown()));
        app.add_plugins(SkeinPlugin::default());
    } else {
        app.add_plugins((YoleckPluginForGame, Vpeol3dPluginForGame));
        app.add_plugins(SkeinPlugin { handle_brp: false });
        app.add_plugins(KbgpPlugin);
        app.insert_resource(KbgpSettings {
            disable_default_navigation: true,
            disable_default_activation: false,
            prevent_loss_of_focus: true,
            focus_on_mouse_movement: true,
            allow_keyboard: true,
            allow_mouse_buttons: false,
            allow_mouse_wheel: false,
            allow_mouse_wheel_sideways: false,
            allow_gamepads: true,
            bindings: {
                KbgpNavBindings::default()
                    .with_wasd_navigation()
                    .with_key(KeyCode::Escape, KbgpNavCommand::user(ActionForKbgp::Menu))
                    .with_key(
                        KeyCode::Backspace,
                        KbgpNavCommand::user(ActionForKbgp::RestartLevel),
                    )
                    .with_key(KeyCode::Space, KbgpNavCommand::Click)
                    .with_key(KeyCode::KeyJ, KbgpNavCommand::Click)
                    .with_gamepad_button(
                        GamepadButton::Start,
                        KbgpNavCommand::user(ActionForKbgp::Menu),
                    )
                    .with_gamepad_button(
                        GamepadButton::Select,
                        KbgpNavCommand::user(ActionForKbgp::RestartLevel),
                    )
            },
        });
    }

    app.add_plugins(TimeToTopplePlugin {
        is_editor: args.editor,
        start_at_level: args.level,
    });
    app.run();
}
