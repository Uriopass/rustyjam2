use crate::entities::{spawn_chicken, spawn_dog, spawn_wolf, start_game};
use crate::{DespawnQry, Score};
use bevy::prelude::*;
use bevy_egui::egui::Align;
use bevy_egui::{egui, EguiContext};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

pub struct LeaderboardResult {
    pub username: String,
    pub score: f32,
}

pub enum GameState {
    Menu,
    Playing,
    EndGamePlaying,
    EndGame {
        score_sent: bool,
        leaderboard_load: bool,
        finished_loading: AtomicBool,
        finished_sending: AtomicBool,
        username: String,
        leaderboard_result: Mutex<Vec<LeaderboardResult>>,
    },
}

// Parse the json_encoded leaderboard results into a vector of LeaderboardResult structs without using
// a json library. Using a manual parser.
// The fields can be in any order, so the parser must check the key names.
//
// An example of such json is:
// {
//     "username": "test",
//     "score": 1.0,
// }
//
// another example is
// {
//     "score": 3.0,
//     "username": "test",
// }
pub fn parse_leaderboard_results(json_str: &str) -> Vec<LeaderboardResult> {
    let mut result = vec![];
    let mut username = String::new();
    let mut score = String::new();
    let mut in_username = false;
    let mut in_score = false;
    let mut in_str = false;
    let mut in_value = false;

    for c in json_str.chars() {
        if c == '}' {
            println!("{}", username);
            println!("{}", score);
            result.push(LeaderboardResult {
                username: std::mem::take(&mut username),
                score: score.trim().parse().unwrap(),
            });
            score.clear();
            in_username = false;
            in_score = false;
            in_value = false;
        } else if c == 'u' && in_str && !in_value {
            in_username = true;
            in_score = false;
        } else if c == 's' && in_str && !in_value {
            in_score = true;
            in_username = false;
        } else if c == ',' {
            in_value = false;
            in_username = false;
            in_score = false;
        } else if c == ':' {
            in_value = true;
        } else if c == '"' {
            in_str = !in_str;
        } else if in_username && in_str && in_value {
            username.push(c);
        } else if in_score && in_value {
            score.push(c);
        }
    }

    result
}

#[cfg(test)]
#[test]
fn test_parse_leaderboard_results() {
    let json_str = "[{\"username\":\"test\",\"score\":1.0}, {\"score\":1.0,\"username\":\"test\"}]";
    let result = parse_leaderboard_results(json_str);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].username, "test");
    assert_eq!(result[0].score, 1.0);

    let json_str = "[{\"score\":2.0,\"username\":\"test\"}]";
    let result = parse_leaderboard_results(json_str);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].username, "test");
    assert_eq!(result[0].score, 2.0);
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
    mut score: ResMut<Score>,
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
                            start_game(qry, &mut commands, &asset_server, &time, &mut score);
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
                .show(egui_context.ctx_mut(), move |ui| {
                    if ui.button("Restart").clicked() {
                        *state = GameState::Playing;
                        start_game(qry, &mut commands, &asset_server, &time, &mut score);
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
        GameState::EndGame {
            ref finished_loading,
            ref finished_sending,
            ref mut score_sent,
            ref mut username,
            ref mut leaderboard_load,
            ref leaderboard_result,
        } => {
            let mut newstate = None;

            egui::Window::new("The End")
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
                .show(egui_context.ctx_mut(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(format!("You scored: {}", score.score));
                        ui.label(format!("Good job!"));

                        let mut lol = username.clone();
                        ui.horizontal(|ui| {
                            ui.label("Username: ");
                            ui.text_edit_singleline(&mut lol);
                        });
                        *username = lol.clone();

                        if !*score_sent
                            && ui
                                .add_enabled(username.len() > 0, egui::Button::new("Send score"))
                                .clicked()
                        {
                            *score_sent = true;
                            let formatted_json = format!(
                                r#"{{"game": "rustyjam2", "score": {}, "username": {}}}"#,
                                score.score, &username
                            );

                            let request = ehttp::Request::post(
                                "https://leaderboard.douady.paris/api/score",
                                formatted_json.into(),
                            );
                            ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
                                println!("Status code: {:?}", result.unwrap().status);
                            });
                        }

                        let finished_sending =
                            finished_sending.load(std::sync::atomic::Ordering::Relaxed);

                        if *score_sent && !finished_sending {
                            ui.label("Sending...");
                        }

                        if finished_sending && !*leaderboard_load {
                            *leaderboard_load = true;
                        }

                        if ui.button("Restart").clicked() {
                            start_game(qry, &mut commands, &asset_server, &time, &mut score);
                            newstate = Some(GameState::Playing);
                        }

                        if ui.button("Continue playing").clicked() {
                            newstate = Some(GameState::EndGamePlaying);
                        }
                    });
                });
            if let Some(newstate) = newstate.take() {
                *state = newstate;
            }
        }
    }
}
