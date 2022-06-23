mod entities;
mod gfx;
mod ui;

use crate::entities::{
    ChickenScaredSound, ChickenSound, DespawnQry, DogScaredSound, DogSound, GameOverSound,
    MergeSound, Score, SoundState, TrackedByKDTree,
};
use crate::gfx::Inputs;
use crate::ui::GameState;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_kira_audio::{Audio, AudioApp, AudioPlugin};
use bevy_prototype_lyon::prelude::ShapePlugin;
use bevy_spatial::KDTreePlugin2D;
use entities::start_game;

fn main() {
    static UI_EARLY: &str = "ui_early";

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Inputs::default())
        .insert_resource(GameState::Menu)
        .insert_resource(SoundState::default())
        .insert_resource(Score::new(0.0))
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(ShapePlugin)
        .add_plugin(KDTreePlugin2D::<TrackedByKDTree>::default())
        .add_plugin(AudioPlugin)
        .add_startup_system(ui::set_style)
        .add_startup_system(start_background_audio)
        .add_startup_system(gfx::gfx_setup)
        .add_startup_system(
            |mut commands: Commands,
             asset_server: Res<AssetServer>,
             mut state: ResMut<GameState>,
             qry: DespawnQry,
             time: Res<Time>| {
                #[cfg(debug_assertions)]
                {
                    start_game(qry, &mut commands, &asset_server, &time);
                    *state = GameState::Playing;
                }
            },
        )
        .add_stage_before(CoreStage::Update, UI_EARLY, SystemStage::single_threaded())
        .add_system_to_stage(UI_EARLY, gfx::mouse_project)
        .add_system_to_stage(UI_EARLY, gfx::cam_movement)
        .add_system_to_stage(UI_EARLY, gfx::input_mapping)
        .add_audio_channel::<ChickenSound>()
        .add_audio_channel::<ChickenScaredSound>()
        .add_audio_channel::<DogScaredSound>()
        .add_audio_channel::<MergeSound>()
        .add_audio_channel::<GameOverSound>()
        .add_audio_channel::<DogSound>()
        .add_system(entities::collision_avoidance)
        .add_system(ui::ui_example)
        .add_system(entities::sound_update)
        .add_system(entities::dogchick_ai)
        .add_system(entities::speedbob)
        .add_system(entities::wolf_ai)
        .add_system(entities::despawnin)
        .add_system(entities::wander_update)
        .add_system(entities::dogchickanim_update)
        .add_system(entities::score_merge)
        .add_system(entities::game_over_system)
        .run();
}

fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play_looped(asset_server.load("I-Knew-a-Guy.ogg"));
}
