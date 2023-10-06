use bevy::prelude::*;
use bevy::math::Vec3Swizzles;

use bevy_rand::prelude::*;
use bevy_prng::ChaCha8Rng;
use rand::prelude::Rng;

use crate::*;

pub const PLAYER_SPEED: f32 = 0.8;
pub const BALANCE_BASE: f32 = 90.0;
pub const BALANCE_WIGGLE_ROOM: f32 = 3.0;

#[derive(Resource)]
pub struct PlayerPath {
    waypoints: Vec<Vec2>
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Player {
    pub balance: f32,
    pub speed: f32,
    pub path_index: usize,
    pub disrupt_timer: Timer
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Handlebar {
    pub prev_rotation: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Indicator;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .register_type::<Player>()
        .register_type::<Handlebar>()
        .insert_resource(PlayerPath {
            waypoints: vec![
                Vec2::new(1.0, 1.0),
                Vec2::new(2.0, 3.0),
                Vec2::new(2.0, -2.0),
                Vec2::new(5.0, -1.0),
                Vec2::new(-2.0, -1.0),
                Vec2::new(2.0, 0.0),
                Vec2::new(6.0, 0.0),
                Vec2::new(4.0, 3.0),
                Vec2::new(0.0, 0.0),
            ]
        })
        .add_systems(OnEnter(GameState::Gameplay), spawn_player)
        .add_systems(OnExit(GameState::Gameplay), despawn_player)
        .add_systems(Update, move_player.run_if(in_state(GameState::Gameplay)))
        .add_systems(Update, disrupt_player.run_if(in_state(GameState::Gameplay)))
        //.add_systems(Update, handlebar_controls.run_if(in_state(GameState::Gameplay)));
        .add_systems(Update, controller_events.run_if(in_state(GameState::Gameplay)));
    }
}

fn disrupt_player(
    mut _commands: Commands,
    mut player_q: Query<&mut Player>,
    mut handle_q: Query<(&mut Transform, &mut Handlebar)>,
    mut rng_q: Query<&mut EntropyComponent<ChaCha8Rng>>,
    time: Res<Time>,
    mut parrot_event_writer: EventWriter<DistressedParrotEvent>,
    //mut sprite_params : Sprite3dParams
) {
    let mut player = player_q.single_mut();
    let (mut handle_transform, mut handlebar) = handle_q.single_mut();
        
    player.disrupt_timer.tick(time.delta());
    if player.disrupt_timer.just_finished() {
        // all the transformation stuff could be called on a custom event as well
        let mut rng = rng_q.single_mut();
        let rand_range = 0.2f32;
        let rand_val = rng.gen_range(-rand_range..rand_range);

        // first rotate back with old amount
        handle_transform.rotate_local_z(-handlebar.prev_rotation);
        // then roate with new amount
        handle_transform.rotate_local_z(rand_val);
        handlebar.prev_rotation = rand_val;

        let out_min = BALANCE_BASE - (BALANCE_BASE * 0.5);
        let out_max = BALANCE_BASE + (BALANCE_BASE * 0.5);
        player.balance = map_range(rand_val, -rand_range, rand_range, out_min, out_max);
        parrot_event_writer.send(DistressedParrotEvent);
    }
}

fn map_range(num: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    return (num - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}

// keyboard based approach
fn _handlebar_controls(
    keyboard: Res<Input<KeyCode>>,
    mut player_q: Query<&mut Player>,
    mut handle_q: Query<(&mut Transform, &mut Handlebar), Without<Indicator>>,
    time: Res<Time>,
    measurement_q: Query<&Measurement>,
    mut parrot_event_writer: EventWriter<RelaxedParrotEvent>
) {
    let mut player = player_q.single_mut();
    //info!("b {}", player.balance);
    let (mut handle_transform, mut handlebar) = handle_q.single_mut();

    let step = 30.0;
    
    if keyboard.pressed(KeyCode::A) {
        player.balance += step * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::D) {
        player.balance -= step * time.delta_seconds();
    }

    let lower_bound = BALANCE_BASE - BALANCE_WIGGLE_ROOM;
    let upper_bound = BALANCE_BASE + BALANCE_WIGGLE_ROOM;
    if player.balance > lower_bound && player.balance < upper_bound {
        handle_transform.rotate_local_z(-handlebar.prev_rotation);
        handle_transform.rotate_local_z(0.0);
        handlebar.prev_rotation = 0.0;
        parrot_event_writer.send(RelaxedParrotEvent);
    }
}

// events from polling
fn controller_events(
    mut player_q: Query<&mut Player>,
    mut handle_q: Query<(&mut Transform, &mut Handlebar), Without<Indicator>>,
    mut indicator_q: Query<&mut Transform, With<Indicator>>,
    mut measure_events: EventReader<MeasureEvent>
) {
    let mut player = player_q.single_mut();
    //info!("b {}", player.balance);

    let mut indicator_transform = indicator_q.single_mut();
    let (mut handle_transform, mut handlebar) = handle_q.single_mut();

    for event in measure_events.iter() {
        let step = 30.0;

        match event {
            MeasureEvent::Left => {
                player.balance += step;
                indicator_transform.rotate_local_z(PI);
            },
            MeasureEvent::Right => {
                player.balance -= step;
                indicator_transform.rotate_local_z(-PI);
            }
        }
    
        let lower_bound = BALANCE_BASE - BALANCE_WIGGLE_ROOM;
        let upper_bound = BALANCE_BASE + BALANCE_WIGGLE_ROOM;
        if player.balance > lower_bound && player.balance < upper_bound {
            handle_transform.rotate_local_z(-handlebar.prev_rotation);
            handle_transform.rotate_local_z(0.0);
            handlebar.prev_rotation = 0.0;
        }
    }    
}

fn spawn_player(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
    mut sprite_params : Sprite3dParams,
) {
    let camera_player_id = commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        },
        Name::new("PlayerCam")
    )).id();

    let handlebar_id = commands.spawn((
        Sprite3d {
            image:game_assets.handlebar_image.clone(),
            pixels_per_metre: 500.,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            transform: Transform::from_xyz(0., -0.3, -1.0)
                .with_scale(Vec3::new(0.55, 0.45, 0.55)),
            ..default()
            }.bundle(&mut sprite_params),
        Handlebar { prev_rotation: 0.0 },
        Name::new("Handlebar")
    )).with_children(|commands| {
        // this selection could be randomized
        let colors = [ParrotType::Blue, ParrotType::Red, ParrotType::Blue,ParrotType::Red];
        for i in 0..=3 {
            let x = -0.6 + (i as f32 * 0.40);
            let z = 0.01 + (i as f32 * 0.01);
            let xyz = Vec3::new(x, 0.5, z);
            spawn_parrot(commands, &game_assets, &mut sprite_params, xyz, colors[i]);
        }
    })
    .id();

    let dash_id = commands.spawn((Sprite3d {
        image:game_assets.rotation_indicator.clone(),
        pixels_per_metre: 200.,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        transform: Transform::from_xyz(0., -0.24, -0.8)
            .with_scale(Vec3::new(0.15, 0.15, 0.15)),
        ..default()
        }.bundle(&mut sprite_params), Name::new("Dashboard"))).id();

    let indicator_id = commands.spawn((Sprite3d {
        image:game_assets.handle_indicator.clone(),
        pixels_per_metre: 200.,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        transform: Transform::from_xyz(0., -0.24, -0.7)
            .with_scale(Vec3::new(0.1, 0.1, 0.1)),
        ..default()
        }.bundle(&mut sprite_params),
        Indicator,
        Name::new("Indicator"))).id();

    let mut player = commands.spawn((
        SpatialBundle::from_transform(Transform::from_xyz(1.0, 1.0, 1.0)),
        Player {
            balance: BALANCE_BASE,
            path_index: 0,
            speed: PLAYER_SPEED,
            disrupt_timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        },
        EntropyComponent::from(&mut rng),
        Name::new("Player")
    ));
    player.push_children(&[camera_player_id, handlebar_id, dash_id, indicator_id]);
}

fn despawn_player(
    mut commands: Commands,
    player_q: Query<Entity, With<Player>>
) {
    let entity = player_q.single();
    commands.entity(entity).despawn_recursive();
}

fn move_player(
    mut commands: Commands,
    path: Res<PlayerPath>,
    mut player_q: Query<(&mut Transform, &mut Player)>,
    mut game_state: ResMut<NextState<GameState>>,
    time: Res<Time>
) {
    let (mut transform, mut player) = player_q.single_mut();

    let delta = player.speed * time.delta_seconds();

    if path.waypoints.len() > player.path_index {
        let delta_target = path.waypoints[player.path_index] - transform.translation.xz();

        // this step will get us closer to the goal
        if delta_target.length() > delta {
            let movement = delta_target.normalize() * delta;
            transform.translation += movement.extend(0.0).xzy();
            // copy for ownership reasons
            let y = transform.translation.y;
            transform.look_at(path.waypoints[player.path_index].extend(y).xzy(), Vec3::Y);
        } else {
            player.path_index += 1;
        }
    } else {
        game_state.set(GameState::GameOver);
    }
}