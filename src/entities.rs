use crate::gfx::MouseProj;
use crate::{Children, Entity, Parent, Query, Time, Vec2, Vec3, Without};
use bevy::math::{vec2, vec3, Rect, Vec3Swizzles};
use bevy::prelude::*;

#[derive(Component)]
pub struct Wolf;

#[derive(Component)]
pub struct Chicken;

#[derive(Component)]
pub struct Dog;

#[derive(Component)]
pub struct DogChick;

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
    getaway: Vec2,
    randobjective: Option<Vec2>,
    confined_within: Rect<f32>,
    merged: bool,
}

#[derive(Component)]
pub struct AiResult {
    target_speed: f32,
    target_dir: Vec2,
}

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct BobAnim(pub f32);

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

pub fn collision_avoidance(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut toavoid: Query<(
        Entity,
        &mut Looker,
        &Transform,
        Option<&Chicken>,
        Option<&Dog>,
        Option<&DogChick>,
    )>,
) {
    for (_, mut looker, _, _, _, _) in toavoid.iter_mut() {
        looker.getaway = Vec2::ZERO;
    }

    let mut combinaisons = toavoid.iter_combinations_mut::<2>();
    while let Some([mut a, mut b]) = combinaisons.fetch_next() {
        let diff = b.2.translation.xy() - a.2.translation.xy();
        let dist2 = diff.length_squared();
        if dist2 < 20.0 * 20.0 {
            let force = 10.0 * diff / dist2;
            a.1.getaway -= force;
            b.1.getaway += force;
            if a.3.is_some() && b.4.is_some() && !a.1.merged && !b.1.merged {
                a.1.merged = true;
                b.1.merged = true;
                commands.entity(a.0).despawn_recursive();
                commands.entity(b.0).despawn_recursive();

                let dogchickpos = a.2.translation + vec3(diff.x, diff.y, 0.0) / 2.0;

                let dogchick = commands
                    .spawn()
                    .insert(AiResult {
                        target_speed: 10.0,
                        target_dir: vec2(0.0, 0.0),
                    })
                    .insert(Looker {
                        spawn_point: b.1.spawn_point + vec2(500.0, 0.0),
                        spawn_door: b.1.spawn_door + vec2(500.0, 0.0),
                        state: LookerState::HappyInside,
                        location: LookerLocation::Outside,
                        getaway: vec2(0.0, 0.0),
                        randobjective: None,
                        confined_within: DOGCHICK_ENCLOT,
                        merged: false,
                    })
                    .insert(Speed(0.0))
                    .insert(DogChick)
                    .insert_bundle(SpriteBundle {
                        transform: Transform::default()
                            .with_translation(dogchickpos)
                            .with_scale(vec3(0.6, 0.6, 1.0)),
                        texture: asset_server.load("shadow.png"),
                        ..Default::default()
                    })
                    .id();
                commands
                    .spawn()
                    .insert(Parent(dogchick))
                    .insert(BobAnim(fastrand::f32() * 32.0))
                    .insert_bundle(SpriteBundle {
                        transform: Transform::default().with_scale(Vec3::new(1.0, 1.0, 1.0)),
                        texture: asset_server.load("dogchick.png"),
                        ..Default::default()
                    });
            }
        }
    }
}

pub fn dogchick_ai(
    time: Res<Time>,
    inp: Res<MouseProj>,
    mut qry: Query<(&mut Transform, &mut Looker, &mut AiResult, &mut Speed)>,
) {
    for (mut trans, mut looker, mut res, mut speed) in qry.iter_mut() {
        let pos = trans.translation.xy();
        let mut max_speed = 50.0_f32;

        if looker.randobjective.is_none() || looker.randobjective.unwrap().distance(pos) < 5.0 {
            let newpos =
                pos + vec2(fastrand::f32() - 0.5, fastrand::f32() - 0.5).normalize() * 100.0;
            looker.randobjective = Some(newpos);
            let r = looker.confined_within;
            if newpos.y < r.bottom || newpos.y > r.top || newpos.x < r.left || newpos.x > r.right {
                looker.randobjective = None;
            }
        }

        use LookerLocation::*;
        use LookerState::*;
        match looker.state {
            Happy => {}
            HappyInside => {}
            Scared { until } => {
                if until > time.seconds_since_startup() {
                    looker.state = Happy;
                }
            }
            ScaredInside { until } => {
                if until > time.seconds_since_startup() {
                    looker.state = HappyInside;
                }
            }
        }
        let objective = match (looker.location, looker.state) {
            (Inside, HappyInside) => looker.randobjective.unwrap_or(pos),
            (Inside, Happy) | (Outside, HappyInside) => looker.spawn_door,
            (Inside, Scared { .. } | ScaredInside { .. }) => looker.spawn_point,
            (Outside, Scared { .. } | ScaredInside { .. }) => looker.spawn_door,
            (Outside, Happy) if inp.0.distance(pos) < 110.0 => {
                max_speed = 100.0;
                let mut obj = inp.0;
                if obj.y < -630.0 {
                    obj.y = -630.0;
                }
                obj
            }
            (Outside, Happy) => looker.randobjective.unwrap_or(pos),
        };

        if looker.spawn_door.distance(pos) < 5.0 {
            looker.location = match looker.state {
                Happy => Outside,
                HappyInside => Inside,
                Scared { .. } => Inside,
                ScaredInside { .. } => Inside,
            }
        }

        let to_obj = objective - pos;

        res.target_speed = max_speed.min(0.3 * to_obj.length_squared());
        res.target_dir = to_obj.normalize_or_zero();

        let off = speed.0
            * time.delta_seconds()
            * (res.target_dir
                + vec2(fastrand::f32() * 0.1, 0.1 * fastrand::f32())
                + looker.getaway);
        trans.translation.x += off.x;
        trans.translation.y += off.y;

        speed.0 += (res.target_speed - speed.0).min(50.0 * time.delta_seconds());
    }
}

pub fn mutant_ai() {}

pub fn speedbob(
    time: Res<Time>,
    mut qry: Query<(&Speed, &Children, &mut Transform, &AiResult)>,
    mut bobqry: Query<(&mut Transform, &mut BobAnim), Without<AiResult>>,
) {
    for (speed, children, mut trans, airesult) in qry.iter_mut() {
        for child in children.iter() {
            let (mut trans, mut bobanim) = bobqry.get_mut(*child).unwrap();
            bobanim.0 += speed.0 * time.delta_seconds() * 0.3;
            trans.translation.y = bobanim.0.cos() * 6.0;
        }
        trans.scale.x = if (airesult.target_dir.x > 0.0) != (trans.scale.x < 0.0) {
            -trans.scale.x
        } else {
            trans.scale.x
        };
    }
}

pub fn start_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    for _ in 0..10 {
        let x = (-0.5 + fastrand::f32()) * 1000.0;
        let y = fastrand::f32() * 200.0 + 300.0;

        let wolf = commands
            .spawn()
            .insert(AiResult {
                target_speed: 10.0,
                target_dir: vec2(0.0, 0.0),
            })
            .insert(Speed(0.0))
            .insert(Wolf)
            .insert_bundle(SpriteBundle {
                transform: Transform::default().with_translation(Vec3::new(x, y, 0.22)),
                texture: asset_server.load("shadow.png"),
                ..Default::default()
            })
            .id();
        commands
            .spawn()
            .insert(Parent(wolf))
            .insert(BobAnim(fastrand::f32() * 32.0))
            .insert_bundle(SpriteBundle {
                transform: Transform::default().with_scale(Vec3::new(1.0, 1.0, 1.0)),
                texture: asset_server.load("wolf.png"),
                ..Default::default()
            });
    }

    for _ in 0..50 {
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
                getaway: Default::default(),
                randobjective: None,
                confined_within: OUTSIDE,
                merged: false,
            })
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
            .insert(BobAnim(fastrand::f32() * 32.0))
            .insert_bundle(SpriteBundle {
                transform: Transform::default(),
                texture: asset_server.load("dog.png"),
                ..Default::default()
            });
    }

    for _ in 0..50 {
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
                getaway: Default::default(),
                confined_within: OUTSIDE,
                merged: false,
                randobjective: None,
            })
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
            .insert(BobAnim(fastrand::f32() * 32.0))
            .insert_bundle(SpriteBundle {
                transform: Transform::default(),
                texture: asset_server.load("chicken.png"),
                ..Default::default()
            });
    }
}
