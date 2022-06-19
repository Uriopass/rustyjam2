mod entities;
mod gfx;
mod ui;

use crate::gfx::Inputs;
use crate::ui::GameState;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::prelude::ShapePlugin;
use entities::start_game;

fn main() {
    static UI_EARLY: &str = "ui_early";

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Inputs::default())
        .insert_resource(GameState::Menu)
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(ShapePlugin)
        .add_startup_system(ui::set_style)
        .add_startup_system(gfx::gfx_setup)
        .add_startup_system(
            |commands: Commands, asset_server: Res<AssetServer>, mut state: ResMut<GameState>| {
                start_game(commands, asset_server);
                *state = GameState::Playing;
            },
        )
        .add_stage_before(CoreStage::Update, UI_EARLY, SystemStage::single_threaded())
        .add_system_to_stage(UI_EARLY, gfx::mouse_project)
        .add_system_to_stage(UI_EARLY, gfx::cam_movement)
        .add_system_to_stage(UI_EARLY, gfx::input_mapping)
        .add_system(entities::collision_avoidance)
        .add_system(ui::ui_example)
        .add_system(entities::dogchick_ai)
        .add_system(entities::speedbob)
        .add_system(entities::wolf_ai)
        .add_system(entities::wander_update)
        .run();
}
