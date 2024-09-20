use bevy::{
    prelude::*,
    window::{Window, WindowResolution, ExitCondition},
};
use bevy::math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume};
use bevy::render::render_resource::Texture;

const RESOLUTION: Vec2 = Vec2::new(720., 720.);
const TURRET_BASE_SIZE: Vec2 = Vec2::new(26., 16.);
const TURRET_SCALE: f32 = 2.;
const INVADER_SCALE: f32 = 2.;
const BULLET_SCALE: f32 = 2.;
const INVADER_SCREEN_PERCENTAGE: f32 = 0.7;
const TURRET_SIZE: Vec2 = Vec2::new(TURRET_BASE_SIZE.x * TURRET_SCALE, TURRET_BASE_SIZE.y * TURRET_SCALE);
const BULLET_BASE_SIZE: Vec2 = Vec2::new(2., 8.);
const BULLET_SIZE: Vec2 = Vec2::new(BULLET_BASE_SIZE.x * BULLET_SCALE, BULLET_BASE_SIZE.y * BULLET_SCALE);
const BULLET_SPEED: f32 = 400.;
const TURRET_SPEED: f32 = 500.0;
const TURRET_PADDING: f32 = 10.;
const TURRET_COLOUR: Color = Color::srgb(0.3, 0.3, 0.5);
const BULLET_COLOUR: Color = Color::srgb(0.1, 0.1, 0.1);
const SHOOT_COOLDOWN: f32 = 0.5;
const INVADER_A_BASE_SIZE: Vec2 = Vec2::new(16., 16.);
const INVADER_B_BASE_SIZE: Vec2 = Vec2::new(22., 16.);
const INVADER_C_BASE_SIZE: Vec2 = Vec2::new(24., 16.);
const INVADER_A_SIZE: Vec2 = Vec2::new(INVADER_A_BASE_SIZE.x * INVADER_SCALE, INVADER_A_BASE_SIZE.y * TURRET_SCALE);
const INVADER_B_SIZE: Vec2 = Vec2::new(INVADER_B_BASE_SIZE.x * INVADER_SCALE, INVADER_B_BASE_SIZE.y * TURRET_SCALE);
const INVADER_C_SIZE: Vec2 = Vec2::new(INVADER_C_BASE_SIZE.x * INVADER_SCALE, INVADER_C_BASE_SIZE.y * TURRET_SCALE);

const GAP_BETWEEN_INVADERS: f32 = 10.;
const INVADER_STEP_SIZE: f32 = 26.0;
const INVADER_VERTICAL_STEP: f32 = 26.0;
const INVADER_MOVE_INTERVAL: f32 = 1.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(RESOLUTION.x, RESOLUTION.y).with_scale_factor_override(1.0),
                    ..default()
                }),
                exit_condition: ExitCondition::OnPrimaryClosed,
                close_when_requested: false,
            })
            .set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_turret, shoot_bullet))
        .add_systems(FixedUpdate, ((check_for_collisions, move_bullet).chain(), move_invaders, animate_invaders))
        .insert_resource(ShootTimer(Timer::from_seconds(SHOOT_COOLDOWN, TimerMode::Once)))
        .insert_resource(InvaderDirection::Right)
        .insert_resource(InvaderMoveTimer(Timer::from_seconds(INVADER_MOVE_INTERVAL, TimerMode::Repeating)))
        .insert_resource(AnimationTimer(Timer::from_seconds(INVADER_MOVE_INTERVAL, TimerMode::Repeating)))
        .add_event::<CollisionEvent>()
        .run();
}

#[derive(Resource)]
struct ShootTimer(Timer);

#[derive(Resource, Debug)]
enum InvaderDirection {
    Left,
    Right,
}

#[derive(Resource)]
struct InvaderMoveTimer(Timer);

#[derive(Resource)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct Turret;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Invader {
    invader_type: InvaderType,
    animation_frame: usize,
    collider: Collider,
}

#[derive(Component)]
enum InvaderType {
    A,
    B,
    C,
}

#[derive(Event, Default)]
struct CollisionEvent;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites\\turret.png"),
            sprite: Sprite {
                custom_size: Some(TURRET_SIZE),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., -RESOLUTION.y / 2. + TURRET_SIZE.y / 2. + TURRET_PADDING, 0.),
                ..default()
            },
            ..default()
        },
        Turret
    ));

    // let layout = TextureAtlasLayout::add_texture(asset_server.load("sprites/invader_a1.png"));

    let n_columns = (((RESOLUTION.x - 2. * TURRET_PADDING) / (INVADER_C_SIZE.x + GAP_BETWEEN_INVADERS)) * INVADER_SCREEN_PERCENTAGE).floor() as usize;
    let n_rows = 5;

    for row in 0..n_rows {
        for column in 0..n_columns {
            let (invader_type, invader_size, sprite_path) = match row {
                0..=1 => (InvaderType::A, INVADER_A_SIZE, "sprites\\invader_a1.png"),
                2..=3 => (InvaderType::B, INVADER_B_SIZE, "sprites\\invader_b1.png"),
                _ => (InvaderType::C, INVADER_C_SIZE, "sprites\\invader_c1.png"),
            };

            let invader_position = Vec2::new(
                -RESOLUTION.x / 2. + TURRET_PADDING + (column as f32 + 0.5) * (INVADER_C_SIZE.x + GAP_BETWEEN_INVADERS),
                RESOLUTION.y / 2. - (INVADER_A_SIZE.y + GAP_BETWEEN_INVADERS) * (row as f32 + 1.),
            );

            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load(sprite_path),
                    sprite: Sprite {
                        custom_size: Some(invader_size),
                        ..default()
                    },
                    transform: Transform {
                        translation: invader_position.extend(0.0),
                        ..default()
                    },
                    ..default()
                },
                Invader {
                    invader_type,
                    animation_frame: 1,
                },
                Collider
            ));
        }
    }
}

fn move_turret(keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Transform, With<Turret>>, time: Res<Time>) {
    let mut turret_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::KeyA) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += 1.0;
    }

    let new_turret_position = turret_transform.translation.x + direction * TURRET_SPEED * time.delta_seconds();

    let left_bound = -RESOLUTION.x / 2. + TURRET_SIZE.x / 2. + TURRET_PADDING;
    let right_bound = RESOLUTION.x / 2. - TURRET_SIZE.x / 2. - TURRET_PADDING;

    turret_transform.translation.x = new_turret_position.clamp(left_bound, right_bound);
}

fn shoot_bullet(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    query: Query<&Transform, With<Turret>>,
    time: Res<Time>,
    mut shoot_timer: ResMut<ShootTimer>,
    asset_server: Res<AssetServer>,
) {
    shoot_timer.0.tick(time.delta());

    if keyboard_input.just_pressed(KeyCode::Space) && shoot_timer.0.finished() {
        let turret_transform = query.single();

        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("sprites\\turret_bullet.png"),
                sprite: Sprite {
                    custom_size: Some(BULLET_SIZE),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(turret_transform.translation.x, turret_transform.translation.y + TURRET_SIZE.y / 2., 1.),
                    ..default()
                },
                ..default()
            },
            Collider,
            Bullet
        ));
        shoot_timer.0.reset();
    }
}

fn move_bullet(mut commands: Commands, mut query: Query<(Entity, &mut Transform), With<Bullet>>, time: Res<Time>) {
    for (entity, mut bullet_transform) in query.iter_mut() {
        bullet_transform.translation.y += BULLET_SPEED * time.delta_seconds();

        if (bullet_transform.translation.y > RESOLUTION.y / 2.) {
            commands.entity(entity).despawn()
        }
    }
}

fn move_invaders(
    mut query: Query<&mut Transform, With<Invader>>,
    mut direction: ResMut<InvaderDirection>,
    time: Res<Time>,
    mut move_timer: ResMut<InvaderMoveTimer>,
) {
    let mut move_down = false;
    let mut new_direction: InvaderDirection = InvaderDirection::Right;

    move_timer.0.tick(time.delta());

    let largest_x = query
        .iter()
        .max_by(|a, b| a.translation.x.partial_cmp(&b.translation.x).unwrap())
        .map(|t| t.translation.x);

    let smallest_x = query
        .iter()
        .min_by(|a, b| a.translation.x.partial_cmp(&b.translation.x).unwrap())
        .map(|t| t.translation.x);

    if move_timer.0.finished() {
        for mut transform in query.iter_mut() {
            if let (Some(largest_x), Some(smallest_x)) = (largest_x, smallest_x) {
                match *direction {
                    InvaderDirection::Left => {
                        if smallest_x < -RESOLUTION.x / 2. + GAP_BETWEEN_INVADERS + INVADER_C_SIZE.x {
                            move_down = true;
                            new_direction = InvaderDirection::Right;
                        } else {
                            transform.translation.x -= INVADER_STEP_SIZE;
                        }
                    }
                    InvaderDirection::Right => {
                        if largest_x > RESOLUTION.x / 2. - GAP_BETWEEN_INVADERS - INVADER_C_SIZE.x {
                            move_down = true;
                            new_direction = InvaderDirection::Left;
                        } else {
                            transform.translation.x += INVADER_STEP_SIZE;
                        }
                    }
                }
            }
        }
    }

    if move_down {
        for mut transform in query.iter_mut() {
            transform.translation.y -= INVADER_VERTICAL_STEP;
        }
        *direction = new_direction;
        move_down = false
    }
}

fn animate_invaders(
    mut query: Query<(&mut Handle<Image>, &mut Invader)>,
    mut animation_timer: ResMut<AnimationTimer>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
) {
    animation_timer.0.tick(time.delta());

    if animation_timer.0.just_finished() {
        for (mut texture_handle, mut invader) in query.iter_mut() {
            invader.animation_frame = if invader.animation_frame == 1 { 2 } else { 1 };

            let new_texture_path = get_invader_sprite_path(&invader.invader_type, invader.animation_frame);

            *texture_handle = asset_server.load(new_texture_path);
        }
    }
}

fn get_invader_sprite_path(invader_type: &InvaderType, frame: usize) -> String {
    match invader_type {
        InvaderType::A => format!("sprites\\invader_a{}.png", frame),
        InvaderType::B => format!("sprites\\invader_b{}.png", frame),
        InvaderType::C => format!("sprites\\invader_c{}.png", frame),
    }
}

fn check_for_collisions(
    mut commands: Commands,
    bullet_query: Query<&Transform, With<Bullet>>,
    collider_query: Query<(Entity, &Transform, Option<&Invader>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    for bullet_transform in bullet_query.iter() {
        for (collider_entity, collider_transform, maybe_invader) in collider_query.iter() {
            let bullet_bounding_box = Aabb2d::new(
                bullet_transform.translation.truncate(),
                bullet_transform.scale.truncate() / 2.,
            );
            let collider_bounding_box = Aabb2d::new(
                collider_transform.translation.truncate(),
                collider_transform.scale.truncate() / 2.,
            );

            let collision = bullet_bounding_box.intersects(&collider_bounding_box);

            if (collision) {
                println!("{}", collision);
            }

            if collision {
                collision_events.send_default();

                if maybe_invader.is_some() {
                    commands.entity(collider_entity).despawn();
                }
            }
        }
    }
}