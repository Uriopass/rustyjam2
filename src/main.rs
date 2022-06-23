mod entities;
mod gfx;
mod ui;

use crate::entities::{DespawnQry, Score, SoundState, TrackedByKDTree};
use crate::gfx::Inputs;
use crate::ui::GameState;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_spatial::KDTreePlugin2D;

fn main() {
    static UI_EARLY: &str = "ui_early";

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Inputs::default())
        .insert_resource(GameState::Menu)
        .insert_resource(SoundState::default())
        .insert_resource(Score::new(0.0))
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(KDTreePlugin2D::<TrackedByKDTree>::default())
        .add_startup_system(ui::set_style)
        .add_startup_system(gfx::gfx_setup)
        .add_startup_system(start_background_audio)
        .add_stage_before(CoreStage::Update, UI_EARLY, SystemStage::single_threaded())
        .add_system_to_stage(UI_EARLY, gfx::mouse_project)
        .add_system_to_stage(UI_EARLY, gfx::cam_movement)
        .add_system_to_stage(UI_EARLY, gfx::input_mapping)
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
        .add_system(entities::wolf_scared)
        .add_system(entities::game_over_system)
        .run();
}

fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    std::mem::forget(asset_server.load::<AudioSource, _>("chicken1.ogg"));
    std::mem::forget(asset_server.load::<AudioSource, _>("dogbark1.ogg"));
    std::mem::forget(asset_server.load::<AudioSource, _>("merge.ogg"));
    std::mem::forget(asset_server.load::<AudioSource, _>("scared_dog.ogg"));
    std::mem::forget(asset_server.load::<AudioSource, _>("scared_chicken.ogg"));
    std::mem::forget(asset_server.load::<AudioSource, _>("tada.ogg"));
    std::mem::forget(asset_server.load::<AudioSource, _>("wolfwhine.ogg"));

    audio.play_with_settings(
        asset_server.load("I-Knew-a-Guy.ogg"),
        PlaybackSettings {
            repeat: true,
            ..default()
        },
    );
}
