use crate::entities::{spawn_chicken, spawn_dog, spawn_wolf, start_game};
use crate::{DespawnQry, Score};
use bevy::prelude::*;
use bevy_egui::egui::Align;
use bevy_egui::{egui, EguiContext};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GameState {
    Menu,
    Playing,
    EndGamePlaying,
    EndGame,
}

pub(crate) fn set_style(mut egui_context: ResMut<EguiContext>) {
    let ctx = egui_context.ctx_mut();
    let mut style: egui::Style = (*ctx.style()).clone();
    style.visuals.window_shadow.extrusion = 0.0;
    style.text_styles.iter_mut().for_each(|x| {
        x.1.size *= 1.5;
    });
    ctx.set_style(style);
}

pub(crate) fn ui_example(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    time: Res<Time>,
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<GameState>,
    score: Res<Score>,
    qry: DespawnQry,
) {
    match *state {
        GameState::Menu => {
            egui::Window::new("Main Menu")
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
                .show(egui_context.ctx_mut(), |ui| {
                    ui.vertical_centered(|ui| {
                        if ui.button("Start Game").clicked() {
                            *state = GameState::Playing;
                            start_game(qry, &mut commands, &asset_server, &time);
                        }
                    });
                });
        }
        GameState::Playing => {
            egui::Window::new("Score")
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .fixed_size((200.0, 100.0))
                .anchor(egui::Align2::CENTER_TOP, (0.0, 0.0))
                .show(egui_context.ctx_mut(), |ui| {
                    let time_left = score.time_end - time.seconds_since_startup();

                    ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                        ui.label(format!("Time left: {}s", time_left as i64));
                    })
                });
        }
        GameState::EndGamePlaying => {
            egui::Window::new("The End")
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::RIGHT_TOP, (0.0, 0.0))
                .show(egui_context.ctx_mut(), |ui| {
                    if ui.button("Restart").clicked() {
                        *state = GameState::Playing;
                        start_game(qry, &mut commands, &asset_server, &time);
                    }

                    if ui.button("More chickens & dogs").clicked() {
                        for _ in 0..10 {
                            spawn_dog(&mut commands, &asset_server);
                            spawn_chicken(&mut commands, &asset_server);
                        }
                    }

                    if ui.button("More wolves").clicked() {
                        for _ in 0..10 {
                            spawn_wolf(&mut commands, &asset_server);
                        }
                    }
                });
        }
        GameState::EndGame => {
            egui::Window::new("The End")
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
                .show(egui_context.ctx_mut(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(format!("You scored: {}", score.score));
                        ui.label(format!("Good job!"));

                        if ui.button("Restart").clicked() {
                            *state = GameState::Playing;
                            start_game(qry, &mut commands, &asset_server, &time);
                        }

                        if ui.button("Continue playing").clicked() {
                            *state = GameState::EndGamePlaying;
                        }
                    });
                });
        }
    }
}
