use crate::gfx::MouseProj;
use crate::{Children, Entity, GameState, Parent, Query, Time, Vec2, Vec3, Without};
use bevy::audio::prelude::*;
use bevy::audio::AudioSink;
use bevy::math::{vec2, vec3, Rect, Vec3Swizzles};
use bevy::prelude::*;
use bevy_spatial::{KDTreeAccess2D, SpatialAccess};
use std::collections::HashSet;

const HAND_SIZE: f32 = 80.0;

#[derive(Default)]
pub struct Score {
    pub score: i32,
    pub time_end: f64,
}

impl Score {
    pub fn new(start: f64) -> Score {
        Score {
            score: 0,
            time_end: start + 300.0,
        }
    }
}

#[derive(Component)]
pub struct Wolf {
    tired_until: f64,
}

#[derive(Component)]
pub struct Chicken;

#[derive(Component)]
pub struct Dog;

#[derive(Component)]
pub struct DogChick;

#[derive(Component)]
pub struct TrackedByKDTree;

type NNTree = KDTreeAccess2D<TrackedByKDTree>; // type alias for later

#[derive(Copy, Clone)]
pub enum LookerLocation {
    Inside,
    Outside,
}

#[derive(Copy, Clone)]
pub enum LookerState {
    Happy,
    HappyInside,
    Scared { until: f64 },
    ScaredInside { until: f64 },
}

#[derive(Component)]
pub struct Looker {
    spawn_point: Vec2,
    spawn_door: Vec2,
    state: LookerState,
    location: LookerLocation,
}

#[derive(Default, Component)]
pub struct CollisionAvoid {
    getaway: Vec2,
}

#[derive(Component)]
pub struct Wander {
    randobjective: Option<Vec2>,
    confined_within: Rect<f32>,
}

#[derive(Component)]
pub struct AiResult {
    target_speed: f32,
    target_dir: Vec2,
}

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct BobAnim {
    pub anim: f32,
}

const OUTSIDE: Rect<f32> = Rect {
    left: -1000.0,
    right: 1000.0,
    top: 1000.0,
    bottom: -650.0,
};

const DOGCHICK_ENCLOT: Rect<f32> = Rect {
    left: -217.0,
    right: 217.0,
    top: -720.0,
    bottom: -1000.0,
};

const FOREST: Rect<f32> = Rect {
    left: -780.0,
    right: 780.0,
    top: 615.0,
    bottom: 140.0,
};

#[derive(Component, Default)]
pub struct DogChickAnim {
    t: f32,
    w: f32,
}

#[derive(Component)]
pub struct DespawnIn {
    until: f64,
    scale: Option<f32>,
}

// Write a system that changes the state to gameend when the game is over
pub fn game_over_system(
    score: Res<Score>,
    mut state: ResMut<GameState>,
    time: Res<Time>,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
) {
    if matches!(*state, GameState::Playing) && time.seconds_since_startup() > score.time_end {
        audio.play(asset_server.load("tada.ogg"));
        *state = GameState::EndGame;
    }
}

pub fn despawnin(
    mut commands: Commands,
    time: Res<Time>,
    mut qry: Query<(Entity, &mut Transform, &mut DespawnIn)>,
) {
    for (ent, mut trans, mut v) in qry.iter_mut() {
        let diff = (v.until - time.seconds_since_startup()) as f32;

        let scale = match v.scale {
            None => {
                v.scale = Some(trans.scale.x);
                trans.scale.x
            }
            Some(x) => x,
        };

        if diff < 0.1 {
            trans.scale.x = scale * diff * 10.0;
            trans.scale.y = scale * diff * 10.0;
        }

        if diff > 0.8 {
            trans.scale.x = scale * (1.0 + (diff - 0.8) * 20.0);
            trans.scale.y = scale * (1.0 + (diff - 0.8) * 20.0);
        }

        if diff < 0.0 {
            commands.entity(ent).despawn_recursive();
        }
    }
}

// Add 100 points when any dogs and chickens merge together to the Score resource
// Spawn a text floating above the added dogchick that says the number of points added using a brown color
pub fn score_merge(
    mut commands: Commands,
    mut score: ResMut<Score>,
    state: Res<GameState>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    qry: Query<Entity, Added<DogChick>>,
) {
    if *state != GameState::Playing {
        return;
    }

    for ent in qry.iter() {
        score.score += 100;
        commands
            .spawn()
            .insert_bundle(Text2dBundle {
                text: Text::with_section(
                    format!("+100"),
                    TextStyle {
                        font: asset_server.load("Roboto-Bold.ttf"),
                        font_size: 30.0,
                        color: Color::rgb(0.8, 0.6, 0.3),
                    },
                    TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                ),
                transform: Transform::from_translation(vec3(00.0, 40.0, 1.0)),
                ..Default::default()
            })
            .insert(Parent(ent))
            .insert(DespawnIn {
                until: time.seconds_since_startup() + 1.0,
                scale: Some(0.5),
            });
    }
}

#[derive(Default)]
pub struct SoundState {
    new_scared_chicken: bool,
    new_scared_dog: bool,

    clear_chick: Option<Handle<AudioSink>>,
    clear_dog: Option<Handle<AudioSink>>,

    hand_state_chick: HashSet<Entity>,
    hand_state_dog: HashSet<Entity>,
}

pub fn sound_update(
    hand: Res<MouseProj>,
    mut state: ResMut<SoundState>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>,
    chicks: Query<(Entity, &Transform, &Looker), With<Chicken>>,
    dogs: Query<(Entity, &Transform, &Looker), With<Dog>>,
) {
    if let Some(h) = &state.clear_chick {
        if let Some(x) = audio_sinks.get(h) {
            x.play();
        }
    }
    if let Some(h) = &state.clear_dog {
        if let Some(x) = audio_sinks.get(h) {
            x.play();
        }
    }

    if state.new_scared_chicken {
        if let Some(h) = &state.clear_chick {
            if let Some(x) = audio_sinks.get(h) {
                x.stop();
            }
        }
        let wh = audio.play(asset_server.load("scared_chicken.ogg"));
        let sh = audio_sinks.get_handle(wh);
        state.clear_chick = Some(sh);
        state.new_scared_chicken = false;
    }

    if state.new_scared_dog {
        if let Some(h) = &state.clear_dog {
            if let Some(x) = audio_sinks.get(h) {
                x.stop();
            }
        }
        let wh = audio.play(asset_server.load("scared_dog.ogg"));
        let sh = audio_sinks.get_handle(wh);
        state.clear_dog = Some(sh);
        state.new_scared_dog = false;
    }
    let mut already = false;

    let mut newset = HashSet::new();
    for (ent, trans, chick) in chicks.iter() {
        if matches!(chick.location, LookerLocation::Outside)
            && matches!(chick.state, LookerState::Happy)
            && trans.translation.xy().distance(hand.0) < HAND_SIZE
        {
            newset.insert(ent);
            if !already {
                if state.hand_state_chick.insert(ent) {
                    already = true;
                    audio.play_with_settings(
                        asset_server.load("chicken1.ogg"),
                        PlaybackSettings {
                            repeat: false,
                            volume: 1.0,
                            speed: fastrand::f32() * 0.3 + 1.0,
                        },
                    );
                }
            }
        }
    }
    state.hand_state_chick = newset;

    let mut newset = HashSet::new();

    let mut already = false;

    for (ent, trans, dog) in dogs.iter() {
        if matches!(dog.location, LookerLocation::Outside)
            && matches!(dog.state, LookerState::Happy)
            && trans.translation.xy().distance(hand.0) < HAND_SIZE
        {
            newset.insert(ent);
            if !already {
                if state.hand_state_dog.insert(ent) {
                    already = true;
                    audio.play_with_settings(
                        asset_server.load("dogbark1.ogg"),
                        PlaybackSettings {
                            repeat: false,
                            volume: 1.0,
                            speed: fastrand::f32() * 0.3 + 1.0,
                        },
                    );
                }
            }
        }
    }
    state.hand_state_dog = newset;
}

pub fn dogchickanim_update(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut qry: Query<(Entity, &mut Transform, &mut DogChickAnim)>,
) {
    for (ent, mut trans, mut anim) in qry.iter_mut() {
        anim.w += time.delta_seconds();

        trans.rotation =
            Quat::from_axis_angle(Vec3::Z, 100.0 * anim.t.powi(2) * time.delta_seconds())
                * trans.rotation;
        trans.scale.x = 1.0 - anim.t * 0.8;
        trans.scale.y = 1.0 - anim.t * 0.8;

        anim.t += time.delta_seconds();

        if anim.t >= 1.0 {
            spawn_dogchick(&mut commands, &asset_server, trans.translation);
            commands.entity(ent).despawn_recursive();
        }
    }
}

pub fn collision_avoidance(
    mut commands: Commands,
    time: Res<Time>,
    tree: Res<NNTree>,
    audio: Res<Audio>,
    mut soundstate: ResMut<SoundState>,
    asset_server: Res<AssetServer>,
    mut toavoid: Query<
        (Entity, &mut CollisionAvoid, &Transform),
        Or<(With<Wolf>, With<Dog>, With<Chicken>, With<DogChick>)>,
    >,
    mut lookers: Query<(Entity, &Transform), Or<(With<Dog>, With<Chicken>)>>,
    wolved: Query<&Transform, With<Wolf>>,
    isdog: Query<&Dog>,
    ischick: Query<&Chicken>,
    mut islooker: Query<&mut Looker>,
    transqry: Query<&Transform>,
    childs: Query<&Children>,
) {
    for (e, mut avoid, trans) in toavoid.iter_mut() {
        avoid.getaway = Vec2::ZERO;

        for (pos, e2) in tree.within_distance(trans.translation, 20.0) {
            let diff = pos.xy() - trans.translation.xy();
            let dist2 = diff.length_squared();

            if dist2 < f32::EPSILON || e == e2 {
                continue;
            }

            let force = 10.0 * diff / dist2;
            avoid.getaway -= force;
        }
    }

    let mut merged = vec![];
    for (e, trans) in lookers.iter_mut() {
        for (pos, e2) in tree.within_distance(trans.translation, 20.0) {
            if !merged.contains(&e)
                && !merged.contains(&e2)
                && (isdog.contains(e) && ischick.contains(e2)
                    || isdog.contains(e2) && ischick.contains(e))
            {
                audio.play(asset_server.load("merge.ogg"));
                merged.push(e);
                merged.push(e2);

                let dogchickpos = (trans.translation + pos) / 2.0;

                let anim = commands
                    .spawn()
                    .insert(DogChickAnim::default())
                    .insert_bundle(TransformBundle {
                        local: Transform::default().with_translation(dogchickpos),
                        global: Default::default(),
                    })
                    .id();

                commands
                    .entity(e)
                    .insert(Parent(anim))
                    .remove::<Looker>()
                    .remove::<Dog>()
                    .remove::<Chicken>()
                    .insert(trans.with_translation(trans.translation - dogchickpos));
                commands
                    .entity(e2)
                    .insert(Parent(anim))
                    .remove::<Looker>()
                    .remove::<Dog>()
                    .remove::<Chicken>()
                    .insert(
                        transqry
                            .get(e2)
                            .unwrap()
                            .with_translation(pos - dogchickpos),
                    );
            }
        }
    }

    for trans in wolved.iter() {
        for (_, e) in tree.within_distance(trans.translation, 150.0) {
            if let Ok(mut l) = islooker.get_mut(e) {
                use LookerState::*;
                match l.state {
                    Happy => {
                        l.state = Scared {
                            until: time.seconds_since_startup() + 10.0,
                        };

                        if ischick.contains(e) {
                            soundstate.new_scared_chicken = true;
                        } else {
                            soundstate.new_scared_dog = true;
                        }

                        let y = if ischick.contains(e) { 30.0 } else { 23.0 };
                        let x = if ischick.contains(e) { -20.0 } else { -20.0 };

                        commands
                            .spawn()
                            .insert(DespawnIn {
                                until: time.seconds_since_startup() + 1.0,
                                scale: None,
                            })
                            .insert(Parent(childs.get(e).unwrap()[0]))
                            .insert_bundle(SpriteBundle {
                                transform: Transform::default()
                                    .with_translation(Vec3::new(x, y, 0.0))
                                    .with_scale(vec3(0.32, 0.32, 0.0)),
                                texture: asset_server.load("scared.png"),
                                ..Default::default()
                            });
                    }
                    HappyInside => {
                        l.state = ScaredInside {
                            until: time.seconds_since_startup() + 10.0,
                        };
                    }
                    Scared { .. } => {}
                    ScaredInside { .. } => {}
                }
            }
        }
    }
}

fn spawn_dogchick(commands: &mut Commands, asset_server: &Res<AssetServer>, pos: Vec3) {
    let x = DOGCHICK_ENCLOT.left + fastrand::f32() * (DOGCHICK_ENCLOT.right - DOGCHICK_ENCLOT.left);
    let y =
        DOGCHICK_ENCLOT.bottom + fastrand::f32() * (DOGCHICK_ENCLOT.top - DOGCHICK_ENCLOT.bottom);

    let door = (fastrand::f32() - 0.5) * 100.0;

    let sp = vec2(x, y);
    let dogchick = commands
        .spawn()
        .insert(AiResult {
            target_speed: 10.0,
            target_dir: vec2(0.0, 0.0),
        })
        .insert(Looker {
            spawn_point: sp,
            spawn_door: vec2(door, -650.0),
            state: LookerState::HappyInside,
            location: LookerLocation::Outside,
        })
        .insert(CollisionAvoid::default())
        .insert(Wander {
            randobjective: Some(sp),
            confined_within: DOGCHICK_ENCLOT,
        })
        .insert(Speed(0.0))
        .insert(TrackedByKDTree)
        .insert(DogChick)
        .insert_bundle(SpriteBundle {
            transform: Transform::default()
                .with_translation(pos)
                .with_scale(vec3(0.6, 0.6, 1.0)),
            texture: asset_server.load("shadow.png"),
            ..Default::default()
        })
        .id();
    commands
        .spawn()
        .insert(Parent(dogchick))
        .insert(BobAnim {
            anim: fastrand::f32() * 32.0,
        })
        .insert_bundle(SpriteBundle {
            transform: Transform::default().with_scale(Vec3::new(1.0, 1.0, 1.0)),
            texture: asset_server.load("dogchick.png"),
            ..Default::default()
        });
}

pub fn wolf_ai(
    time: Res<Time>,
    mut qry: Query<(
        &mut Transform,
        &mut Wolf,
        &Wander,
        &CollisionAvoid,
        &mut AiResult,
        &mut Speed,
    )>,
    mut targets: Query<(&Transform, &Looker), (Without<Wolf>, Or<(With<Dog>, With<Chicken>)>)>,
) {
    for (mut trans, mut wolf, wander, avoid, mut res, mut speed) in qry.iter_mut() {
        let mut max_speed = 20.0_f32;
        let pos = trans.translation.xy();

        let mut nearest = None;
        let mut neares_dist = f32::INFINITY;

        for (trans, look) in targets.iter_mut() {
            if matches!(look.location, LookerLocation::Inside) {
                continue;
            }

            let tpos = trans.translation.xy();
            let d = tpos.distance_squared(pos);

            if d < neares_dist {
                neares_dist = d;
                nearest = Some(tpos);
            }
        }

        let objective = match nearest {
            Some(x)
                if x.distance(pos) < 600.0 && wolf.tired_until < time.seconds_since_startup() =>
            {
                max_speed = 100.0;
                x
            }
            _ => {
                let obj = wander.randobjective.unwrap_or(pos);
                if obj.distance_squared(pos) > 300.0 * 300.0 {
                    max_speed = 60.0;
                }
                obj
            }
        };

        if trans.translation.y < -530.0 {
            wolf.tired_until = time.seconds_since_startup() + 15.0;
        }

        let to_obj = objective - pos;

        res.target_speed = max_speed.min(0.3 * to_obj.length_squared());
        res.target_dir = to_obj.normalize_or_zero();

        let off = speed.0
            * time.delta_seconds()
            * (res.target_dir + vec2(fastrand::f32() * 0.1, 0.1 * fastrand::f32()) + avoid.getaway);
        trans.translation.x += off.x;
        trans.translation.y += off.y;

        speed.0 += (res.target_speed - speed.0).min(100.0 * time.delta_seconds());
    }
}

pub fn wander_update(mut qry: Query<(&Transform, &mut Wander)>) {
    for (trans, mut wander) in qry.iter_mut() {
        let pos = trans.translation.xy();
        if wander.randobjective.is_none()
            || wander.randobjective.unwrap().distance(pos) < 5.0
            || wander.randobjective.unwrap().distance(pos) > 70.0
        {
            let newpos =
                pos + vec2(fastrand::f32() - 0.5, fastrand::f32() - 0.5).normalize() * 70.0;
            let r = wander.confined_within;
            if newpos.y >= r.bottom
                && newpos.y <= r.top
                && newpos.x >= r.left
                && newpos.x <= r.right
            {
                wander.randobjective = Some(newpos);
            }
        }
    }
}

pub fn dogchick_ai(
    time: Res<Time>,
    inp: Res<MouseProj>,
    mut qry: Query<(
        &mut Transform,
        &mut Looker,
        &CollisionAvoid,
        &Wander,
        &mut AiResult,
        &mut Speed,
    )>,
) {
    for (mut trans, mut looker, avoid, wander, mut res, mut speed) in qry.iter_mut() {
        let pos = trans.translation.xy();
        let mut max_speed = 50.0_f32;

        use LookerLocation::*;
        use LookerState::*;
        match looker.state {
            Happy => {}
            HappyInside => {}
            Scared { until } => {
                max_speed = 180.0;
                if until < time.seconds_since_startup() {
                    looker.state = Happy;
                }
            }
            ScaredInside { until } => {
                max_speed = 180.0;
                if until < time.seconds_since_startup() {
                    looker.state = HappyInside;
                }
            }
        }
        let objective = match (looker.location, looker.state) {
            (Inside, HappyInside) => wander.randobjective.unwrap_or(pos),
            (Inside, Happy) | (Outside, HappyInside) => looker.spawn_door,
            (Inside, Scared { .. } | ScaredInside { .. }) => looker.spawn_point,
            (Outside, Scared { .. } | ScaredInside { .. }) => looker.spawn_door,
            (Outside, Happy) if inp.0.distance(pos) < HAND_SIZE => {
                max_speed = 150.0;
                let mut obj = inp.0;
                if obj.y < -630.0 {
                    obj.y = -630.0;
                }
                obj
            }
            (Outside, Happy) => wander.randobjective.unwrap_or(pos),
        };

        if looker.spawn_door.distance(pos) < 20.0 {
            looker.location = match looker.state {
                Happy => Outside,
                HappyInside => Inside,
                Scared { .. } => Inside,
                ScaredInside { .. } => Inside,
            }
        }

        let to_obj = objective - pos;

        res.target_speed = max_speed.min(0.8 * to_obj.length_squared());
        res.target_dir = to_obj.normalize_or_zero();

        let off = speed.0
            * time.delta_seconds()
            * (res.target_dir + vec2(fastrand::f32() * 0.1, 0.1 * fastrand::f32()) + avoid.getaway);
        trans.translation.x += off.x;
        trans.translation.y += off.y;

        speed.0 += (res.target_speed - speed.0).min(100.0 * time.delta_seconds());
    }
}

pub fn speedbob(
    time: Res<Time>,
    mut qry: Query<(&Speed, &Children, &AiResult)>,
    mut bobqry: Query<(&mut Transform, &mut BobAnim), Without<AiResult>>,
) {
    for (speed, children, airesult) in qry.iter_mut() {
        for child in children.iter() {
            let (mut trans, mut bobanim) = match bobqry.get_mut(*child) {
                Ok(x) => x,
                Err(_) => continue,
            };
            bobanim.anim += speed.0 * time.delta_seconds() * 0.3;
            trans.translation.y = bobanim.anim.cos() * 6.0;
            trans.scale.x = if (airesult.target_dir.x > 0.0) != (trans.scale.x < 0.0) {
                -trans.scale.x
            } else {
                trans.scale.x
            };
        }
    }
}

pub type DespawnQry<'a, 'b> =
    Query<'a, 'b, Entity, Or<(With<Dog>, With<DogChick>, With<Wolf>, With<Chicken>)>>;

pub fn start_game(
    qry: DespawnQry,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    time: &Res<Time>,
) {
    commands.insert_resource(Score::new(time.seconds_since_startup()));

    for ent in qry.iter() {
        commands.entity(ent).despawn_recursive();
    }

    for _ in 0..10 {
        spawn_wolf(commands, asset_server);
    }

    for _ in 0..70 {
        spawn_dog(commands, asset_server);
    }

    for _ in 0..70 {
        spawn_chicken(commands, asset_server);
    }
}

pub fn spawn_wolf(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let x = (-0.5 + fastrand::f32()) * 1000.0;
    let y = fastrand::f32() * 200.0 + 300.0;

    let wolf = commands
        .spawn()
        .insert(AiResult {
            target_speed: 10.0,
            target_dir: vec2(0.0, 0.0),
        })
        .insert(Speed(0.0))
        .insert(Wolf { tired_until: 0.0 })
        .insert(Wander {
            randobjective: None,
            confined_within: FOREST,
        })
        .insert(CollisionAvoid::default())
        .insert_bundle(SpriteBundle {
            transform: Transform::default().with_translation(Vec3::new(x, y, 0.22)),
            texture: asset_server.load("shadow.png"),
            ..Default::default()
        })
        .insert(TrackedByKDTree)
        .id();
    commands
        .spawn()
        .insert(Parent(wolf))
        .insert(BobAnim {
            anim: fastrand::f32() * 32.0,
        })
        .insert_bundle(SpriteBundle {
            transform: Transform::default().with_scale(Vec3::new(1.0, 1.0, 1.0)),
            texture: asset_server.load("wolf.png"),
            ..Default::default()
        });
}

pub fn spawn_chicken(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let x = 500.0 + (-0.5 + fastrand::f32()) * 300.0;
    let y = fastrand::f32() * 300.0 - 1000.0;

    let chicken = commands
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform::default()
                .with_translation(Vec3::new(x, y, 0.22))
                .with_scale(vec3(0.3, 0.3, 1.0)),
            texture: asset_server.load("shadow.png"),
            ..Default::default()
        })
        .insert(Looker {
            spawn_point: vec2(x, y),
            spawn_door: vec2(500.0 + 100.0 * (fastrand::f32() - 0.5), -650.0),
            state: LookerState::Happy,
            location: LookerLocation::Inside,
        })
        .insert(CollisionAvoid::default())
        .insert(Wander {
            randobjective: None,
            confined_within: OUTSIDE,
        })
        .insert(TrackedByKDTree)
        .insert(AiResult {
            target_speed: 10.0,
            target_dir: vec2(0.0, 0.0),
        })
        .insert(Speed(0.0))
        .insert(Chicken)
        .id();

    commands
        .spawn()
        .insert(Parent(chicken))
        .insert(BobAnim {
            anim: fastrand::f32() * 32.0,
        })
        .insert_bundle(SpriteBundle {
            transform: Transform::default(),
            texture: asset_server.load("chicken.png"),
            ..Default::default()
        });
}

pub fn spawn_dog(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let x = -500.0 + (-0.5 + fastrand::f32()) * 300.0;
    let y = fastrand::f32() * 300.0 - 1000.0;

    let dog = commands
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform::default()
                .with_translation(Vec3::new(x, y, 0.22))
                .with_scale(vec3(0.5, 0.5, 1.0)),
            texture: asset_server.load("shadow.png"),
            ..Default::default()
        })
        .insert(Looker {
            spawn_point: vec2(x, y),
            spawn_door: vec2(-500.0 + 100.0 * (fastrand::f32() - 0.5), -650.0),
            state: LookerState::Happy,
            location: LookerLocation::Inside,
        })
        .insert(CollisionAvoid::default())
        .insert(Wander {
            randobjective: None,
            confined_within: OUTSIDE,
        })
        .insert(TrackedByKDTree)
        .insert(AiResult {
            target_speed: 10.0,
            target_dir: vec2(0.0, 0.0),
        })
        .insert(Speed(0.0))
        .insert(Dog)
        .id();

    commands
        .spawn()
        .insert(Parent(dog))
        .insert(BobAnim {
            anim: fastrand::f32() * 32.0,
        })
        .insert_bundle(SpriteBundle {
            transform: Transform::default(),
            texture: asset_server.load("dog.png"),
            ..Default::default()
        });
}
