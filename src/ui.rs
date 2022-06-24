use crate::entities::{spawn_chicken, spawn_dog, spawn_wolf, start_game};
use crate::{DespawnQry, Score};
use bevy::prelude::*;
use bevy_egui::egui::Align;
use bevy_egui::{egui, EguiContext};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub struct LeaderboardResult {
    pub username: String,
    pub score: f32,
}

pub enum GameState {
    Menu {
        leaderboard_load: bool,
        finished_loading: Arc<AtomicBool>,
        leaderboard_result: Arc<Mutex<Vec<LeaderboardResult>>>,
    },
    Playing,
    EndGamePlaying,
    EndGame {
        score_sent: bool,
        leaderboard_load: bool,
        finished_loading: Arc<AtomicBool>,
        finished_sending: Arc<AtomicBool>,
        username: String,
        leaderboard_result: Arc<Mutex<Vec<LeaderboardResult>>>,
        error: Arc<AtomicBool>,
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
        } else if c == 'o' && in_str && !in_value {
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
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].username, "test");
    assert_eq!(result[0].score, 1.0);
    assert_eq!(result[1].username, "test");
    assert_eq!(result[1].score, 1.0);

    let json_str = "[{\"score\":2.0,\"username\":\"test\"}]";
    let result = parse_leaderboard_results(json_str);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].username, "test");
    assert_eq!(result[0].score, 2.0);

    let json_str = "[]";
    let result = parse_leaderboard_results(json_str);
    assert_eq!(result.len(), 0);
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
        GameState::Menu {
            ref leaderboard_result,
            ref finished_loading,
            ref mut leaderboard_load,
        } => {
            let mut newstate = None;
            egui::Window::new("Main Menu")
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
                .show(egui_context.ctx_mut(), |ui| {
                    ui.vertical_centered(|ui| {
                        if ui.button("Start Game").clicked() {
                            newstate = Some(GameState::Playing);
                            start_game(qry, &mut commands, &asset_server, &time, &mut score);
                        }
                        if !*leaderboard_load {
                            *leaderboard_load = true;
                            let cpy = finished_loading.clone();
                            let cpyres = leaderboard_result.clone();
                            let request = ehttp::Request::get("https://leaderboard.douady.paris/api/score/rustyjam2");
                            ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
                                match result {
                                    Ok(v) if v.status == 200 => {
                                        let v = String::from_utf8_lossy(&v.bytes);
                                        println!("got leaderboards: {}", &v);
                                        let res = parse_leaderboard_results(&*v);
                                        *cpyres.lock().unwrap() = res;
                                    }
                                    Ok(v) => println!("errored out with status: {}", v.status),
                                    Err(e) => {
                                        println!("errored out: {:?}", e)
                                    },
                                }
                                cpy.store(true, Ordering::SeqCst);
                            });
                        }

                        if finished_loading.load(Ordering::SeqCst) {
                            let leads = leaderboard_result.lock().unwrap();
                            let g = egui::Grid::new("leaderboards_mainmenu");

                            g.show(ui, |ui| {
                                if leads.len() == 0 {
                                    return;
                                }
                                ui.label("Leaderboard");
                                ui.end_row();
                                ui.label("Name");
                                ui.label("Score");
                                ui.end_row();
                                for r in leads.iter() {
                                    ui.label(&r.username);
                                    ui.label(format!("{}", r.score));
                                    ui.end_row();
                                }
                            });
                        }
                    });
                });

            if let Some(newstate) = newstate {
                *state = newstate;
            }
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
            ref mut error
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

                        if error.load(Ordering::SeqCst) {
                            ui.label("Error sending score :( sorry");
                            if ui.button("retry").clicked() {
                                finished_sending.store(false, Ordering::SeqCst);
                                finished_loading.store(false, Ordering::SeqCst);
                                *score_sent = false;
                                *leaderboard_load = false;
                                error.store(false, Ordering::SeqCst);
                            }
                        } else {
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
                                    r#"{{"game":"rustyjam2","score":{},"username":"{}"}}"#,
                                    score.score as f64, &username
                                );

                                let mut request = ehttp::Request::post(
                                    "https://leaderboard.douady.paris/api/score",
                                    formatted_json.into(),
                                );
                                request.headers.insert("Content-Type".to_string(), "application/json".to_string());
                                let cpy = finished_sending.clone();
                                let cpye = error.clone();
                                ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
                                    match result {
                                        Ok(v) if v.status == 201 => {
                                            cpy.store(true, Ordering::SeqCst);
                                        }
                                        Ok(v) => {
                                            println!("errored cuz status: {}", v.status);
                                            cpye.store(true, Ordering::SeqCst);
                                        }
                                        Err(e) => {
                                            println!("errored cuz: {:?}", e);
                                            cpye.store(true, Ordering::SeqCst)
                                        },
                                    }
                                });
                            }

                            let finished_sending =
                                finished_sending.load(Ordering::SeqCst);

                            if *score_sent && !finished_sending {
                                ui.label("Sending...");
                            }

                            if finished_sending && !*leaderboard_load {
                                *leaderboard_load = true;
                                let cpy = finished_loading.clone();
                                let cpye = error.clone();
                                let cpyres = leaderboard_result.clone();
                                let request = ehttp::Request::get("https://leaderboard.douady.paris/api/score/rustyjam2");
                                ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
                                    match result {
                                        Ok(v) if v.status == 200 => {
                                            let v = String::from_utf8_lossy(&v.bytes);
                                            let res = parse_leaderboard_results(&*v);
                                            *cpyres.lock().unwrap() = res;

                                            cpy.store(true, Ordering::SeqCst);
                                        }
                                        _ => cpye.store(true, Ordering::SeqCst),
                                    }
                                });
                            }

                            let finished_loading = finished_loading.load(Ordering::SeqCst);

                            if *leaderboard_load && !finished_loading {
                                ui.label("Loading leaderboards...");
                            }

                            if finished_loading {
                                let leads = leaderboard_result.lock().unwrap();
                                let g = egui::Grid::new("leaderboards");

                                g.show(ui, |ui| {
                                    if leads.len() == 0 {
                                        return;
                                    }
                                    ui.label("Leaderboard");
                                    ui.end_row();
                                    ui.label("Name");
                                    ui.label("Score");
                                    ui.end_row();
                                    for r in leads.iter() {
                                        ui.label(&r.username);
                                        ui.label(format!("{}", r.score));
                                        ui.end_row();
                                    }
                                });
                            }
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
