use crate::gfx::MouseProj;
use crate::{Children, Entity, Parent, Query, Time, Vec2, Vec3, Without};
use bevy::math::{vec2, vec3, Rect, Vec3Swizzles};
use bevy::prelude::*;

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
    merged: bool,
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

const FOREST: Rect<f32> = Rect {
    left: -780.0,
    right: 780.0,
    top: 615.0,
    bottom: 140.0,
};

pub fn collision_avoidance(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut toavoid: Query<(
        Entity,
        &mut CollisionAvoid,
        &Transform,
        Option<&Chicken>,
        Option<&Dog>,
        Option<&DogChick>,
        Option<&mut Looker>,
        Option<&Wolf>,
    )>,
) {
    for (_, mut avoid, _, _, _, _, _, _) in toavoid.iter_mut() {
        avoid.getaway = Vec2::ZERO;
    }

    let mut combinaisons = toavoid.iter_combinations_mut::<2>();
    while let Some([mut a, mut b]) = combinaisons.fetch_next() {
        let diff = b.2.translation.xy() - a.2.translation.xy();
        let dist2 = diff.length_squared();

        if dist2 < 20.0 * 20.0 {
            let force = 10.0 * diff / dist2;
            a.1.getaway -= force;
            b.1.getaway += force;

            if let Some((looka, lookb)) = a.6.as_mut().zip(b.6.as_mut()) {
                if (a.3.is_some() == b.4.is_some()) && !looka.merged && !lookb.merged {
                    looka.merged = true;
                    lookb.merged = true;
                    commands.entity(a.0).despawn_recursive();
                    commands.entity(b.0).despawn_recursive();

                    let dogchickpos = a.2.translation + vec3(diff.x, diff.y, 0.0) / 2.0;

                    let offdir = if lookb.spawn_point.x > 0.0 {
                        -500.0
                    } else {
                        500.0
                    };

                    let sp = lookb.spawn_point + vec2(offdir, 0.0);

                    let dogchick = commands
                        .spawn()
                        .insert(AiResult {
                            target_speed: 10.0,
                            target_dir: vec2(0.0, 0.0),
                        })
                        .insert(Looker {
                            spawn_point: sp,
                            spawn_door: lookb.spawn_door + vec2(offdir, 0.0),
                            state: LookerState::HappyInside,
                            location: LookerLocation::Outside,
                            merged: true,
                        })
                        .insert(CollisionAvoid::default())
                        .insert(Wander {
                            randobjective: Some(sp),
                            confined_within: DOGCHICK_ENCLOT,
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

        if b.7.is_some() && a.6.is_some() && dist2 < 150.0 * 150.0 {
            if let Some(mut looka) = a.6 {
                match looka.state {
                    LookerState::Happy => {
                        looka.state = LookerState::Scared {
                            until: time.seconds_since_startup() + 10.0,
                        }
                    }
                    LookerState::HappyInside => {
                        looka.state = LookerState::ScaredInside {
                            until: time.seconds_since_startup() + 10.0,
                        }
                    }
                    LookerState::Scared { .. } => {}
                    LookerState::ScaredInside { .. } => {}
                }
            }
        }
    }
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
                max_speed = 70.0;
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

        speed.0 += (res.target_speed - speed.0).min(50.0 * time.delta_seconds());
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
            (Outside, Happy) if inp.0.distance(pos) < 110.0 => {
                max_speed = 120.0;
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

        res.target_speed = max_speed.min(0.3 * to_obj.length_squared());
        res.target_dir = to_obj.normalize_or_zero();

        let off = speed.0
            * time.delta_seconds()
            * (res.target_dir + vec2(fastrand::f32() * 0.1, 0.1 * fastrand::f32()) + avoid.getaway);
        trans.translation.x += off.x;
        trans.translation.y += off.y;

        speed.0 += (res.target_speed - speed.0).min(50.0 * time.delta_seconds());
    }
}

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
                merged: false,
            })
            .insert(CollisionAvoid::default())
            .insert(Wander {
                randobjective: None,
                confined_within: OUTSIDE,
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
                merged: false,
            })
            .insert(CollisionAvoid::default())
            .insert(Wander {
                randobjective: None,
                confined_within: OUTSIDE,
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
