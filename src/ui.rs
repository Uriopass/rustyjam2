use crate::entities::start_game;
use crate::gfx::MouseProj;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GameState {
    Menu,
    Playing,
}

pub(crate) fn set_style(mut egui_context: ResMut<EguiContext>) {
    let ctx = egui_context.ctx_mut();
    let mut style: egui::Style = (*ctx.style()).clone();
    style.visuals.window_shadow.extrusion = 0.0;
    ctx.set_style(style);
}

pub(crate) fn ui_example(
    asset_server: Res<AssetServer>,
    commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<GameState>,
    proj: Res<MouseProj>,
) {
    if *state == GameState::Menu {
        egui::Window::new("Main Menu")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
            .show(egui_context.ctx_mut(), |ui| {
                if ui.button("Start Game").clicked() {
                    *state = GameState::Playing;
                    start_game(commands, asset_server);
                }
            });
    }

    egui::Window::new("Debug")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::LEFT_TOP, (0.0, 0.0))
        .show(egui_context.ctx_mut(), |ui| {
            ui.label(format!("{} {}", proj.0.x as i64, proj.0.y as i64));
        });
}
