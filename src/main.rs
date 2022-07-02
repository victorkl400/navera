use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy::utils::HashSet;
use bevy_kira_audio::{Audio, AudioPlugin};
use components::{
	Enemy, Explosion, ExplosionTimer, ExplosionToSpawn, FromEnemy, FromPlayer, Laser, Player,
	SpriteSize,
};
use components::{Movable, Velocity};

mod audio;
mod components;
mod player;
use crate::player::*;

mod enemy;
use crate::audio::*;
use crate::enemy::*;

//Asset Const
const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_SIZE: (f32, f32) = (144.0, 75.0);

const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const PLAYER_LASER_SIZE: (f32, f32) = (9.0, 54.0);
const PLAYER_RESPAWN_DELAY: f64 = 2.0;

const ENEMY_SPRITE: &str = "enemy_a_01.png";
const ENEMY_SIZE: (f32, f32) = (144.0, 75.0);

const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const ENEMY_LASER_SIZE: (f32, f32) = (17.0, 55.0);

const EXPLOSION_SHEET: &str = "explo_a_sheet.png";

const EXPLOSION_LEN: usize = 16;

const SPRITE_SCALE: f32 = 0.5;

const TIME_STEP: f32 = 1.0 / 60.0;
const BASE_SPEED: f32 = 500.0;
const FORMATION_MEMBERS_MAX: u32 = 2;
const ENEMY_MAX: u32 = 4;

struct EnemyCount(u32);
pub struct WinSize {
	pub w: f32,
	pub h: f32,
}
struct GameTextures {
	player: Handle<Image>,
	player_laser: Handle<Image>,
	enemy: Handle<Image>,
	enemy_laser: Handle<Image>,
	explosion: Handle<TextureAtlas>,
}

struct PlayerState {
	on: bool,       //alive
	last_shot: f64, //-1 if not shot
}
impl Default for PlayerState {
	fn default() -> Self {
		Self {
			on: false,
			last_shot: -1.0,
		}
	}
}
impl PlayerState {
	pub fn shot(&mut self, time: f64) {
		self.on = false;
		self.last_shot = time;
	}

	pub fn spawned(&mut self) {
		self.on = true;
		self.last_shot = -1.0;
	}
}
fn main() {
	App::new()
		.insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
		.insert_resource(WindowDescriptor {
			title: String::from("Navera - The Game"),
			width: 600.0, //Float required
			height: 680.0,
			..Default::default()
		})
		.add_plugins(DefaultPlugins)
		.add_plugin(PlayerPlugin)
		.add_plugin(EnemyPlugin)
		.add_plugin(GameAudioPlugin)
		.add_plugin(AudioPlugin)
		.add_startup_system(play_loop)
		.add_startup_system(setup_system)
		.add_system(movable_system)
		.add_system(player_laser_hit_enemy_system)
		.add_system(explosion_to_spawn_system)
		.add_system(explosion_animation_system)
		.add_system(enemy_laser_hit_player_system)
		.run();
}
fn play_loop(asset_server: Res<AssetServer>, audio: Res<Audio>) {
	audio.play_looped(asset_server.load("default.ogg"));
}
fn setup_system(
	mut commands: Commands,
	assets_server: Res<AssetServer>,
	mut windows: ResMut<Windows>,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
	// Camera
	commands.spawn_bundle(OrthographicCameraBundle::new_2d());

	//Get the window size
	let window = windows.get_primary_mut().unwrap();
	let (win_w_, win_h) = (window.width(), window.height());

	let win_size = WinSize { w: win_w_, h: win_h };

	commands.insert_resource(win_size);

	let texture_handle = assets_server.load(EXPLOSION_SHEET);
	let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 4);
	let explosion = texture_atlases.add(texture_atlas);
	//Add game texture resources

	let game_textures = GameTextures {
		player: assets_server.load(PLAYER_SPRITE),
		player_laser: assets_server.load(PLAYER_LASER_SPRITE),
		enemy: assets_server.load(ENEMY_SPRITE),
		enemy_laser: assets_server.load(ENEMY_LASER_SPRITE),
		explosion,
	};
	commands.insert_resource(game_textures);
	commands.insert_resource(EnemyCount(0));
}

fn movable_system(
	mut commands: Commands,
	win_size: Res<WinSize>,
	mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>,
) {
	for (entity, velocity, mut transform, movable) in query.iter_mut() {
		let translation = &mut transform.translation;
		translation.x += velocity.x * TIME_STEP * BASE_SPEED;
		translation.y += velocity.y * TIME_STEP * BASE_SPEED;

		if movable.auto_despawn {
			const MARGIN: f32 = 200.0;
			if translation.y > win_size.h / 2.0 + MARGIN
				|| translation.y < -win_size.h / 2.0 + -MARGIN
				|| translation.x > win_size.h / 2.0 + MARGIN
				|| translation.x < -win_size.h / 2.0 + -MARGIN
			{
				commands.entity(entity).despawn();
			}
		}
	}
}
fn enemy_laser_hit_player_system(
	mut commands: Commands,
	mut player_state: ResMut<PlayerState>,
	time: Res<Time>,
	laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromEnemy>)>,
	player_query: Query<(Entity, &Transform, &SpriteSize), With<Player>>,
) {
	if let Ok((player_entity, player_tf, player_size)) = player_query.get_single() {
		let player_scale = Vec2::from((player_tf.scale.x, player_tf.scale.y));

		for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
			let laser_scale = Vec2::from((player_tf.scale.x, player_tf.scale.y));
			let collision = collide(
				laser_tf.translation,
				laser_size.0 * laser_scale,
				player_tf.translation,
				player_size.0 * player_scale,
			);

			if let Some(_) = collision {
				commands.entity(player_entity).despawn();
				player_state.shot(time.seconds_since_startup());

				commands.entity(laser_entity).despawn();

				commands.spawn().insert(ExplosionToSpawn(player_tf.translation.clone()));

				break;
			}
		}
	}
}
fn player_laser_hit_enemy_system(
	mut commands: Commands,
	mut enemy_count: ResMut<EnemyCount>,
	laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromPlayer>)>,
	enemy_query: Query<(Entity, &Transform, &SpriteSize), With<Enemy>>,
) {
	let mut despawned_entities: HashSet<Entity> = HashSet::new();

	///iter
	for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
		if despawned_entities.contains(&laser_entity) {
			continue;
		}

		let laser_scale = Vec2::from((laser_tf.scale.x, laser_tf.scale.y));

		for (enemy_entity, enemy_tf, enemy_size) in enemy_query.iter() {
			if despawned_entities.contains(&laser_entity)
				|| despawned_entities.contains(&enemy_entity)
			{
				continue;
			}

			let enemy_scale = Vec2::from((enemy_tf.scale.x, enemy_tf.scale.y));

			let collision = collide(
				laser_tf.translation,
				laser_size.0 * laser_scale,
				enemy_tf.translation,
				enemy_size.0 * enemy_scale,
			);

			if let Some(_) = collision {
				//remover inimigo morto
				commands.entity(enemy_entity).despawn();
				despawned_entities.insert(enemy_entity);
				enemy_count.0 -= 1;

				//remover laser atingido
				commands.entity(laser_entity).despawn();
				despawned_entities.insert(laser_entity);

				//spawn the explosionToSpawn
				commands.spawn().insert(ExplosionToSpawn(enemy_tf.translation.clone()));
			}
		}
	}
}
fn explosion_to_spawn_system(
	mut commands: Commands,
	game_textures: Res<GameTextures>,
	query: Query<(Entity, &ExplosionToSpawn)>,
) {
	for (explosion_entity, explosion_to_spawn) in query.iter() {
		commands
			.spawn_bundle(SpriteSheetBundle {
				texture_atlas: game_textures.explosion.clone(),
				transform: Transform {
					translation: explosion_to_spawn.0,
					..Default::default()
				},
				..Default::default()
			})
			.insert(Explosion)
			.insert(ExplosionTimer::default());

		commands.entity(explosion_entity).despawn()
	}
}
fn explosion_animation_system(
	mut commands: Commands,
	time: Res<Time>,
	mut query: Query<(Entity, &mut ExplosionTimer, &mut TextureAtlasSprite), With<Explosion>>,
) {
	for (entity, mut timer, mut sprite) in query.iter_mut() {
		timer.0.tick(time.delta());
		if timer.0.finished() {
			sprite.index += 1;
			if sprite.index >= EXPLOSION_LEN {
				commands.entity(entity).despawn();
			}
		}
	}
}
