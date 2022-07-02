use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioApp, AudioChannel, AudioPlugin, AudioSource};

pub struct GameAudioPlugin;
#[derive(Component, Default, Clone)]
struct FirstChannel;

pub struct AudioState {
	background_handle: Handle<AudioSource>,
	volume: f32,
}

impl Plugin for GameAudioPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugin(AudioPlugin).add_startup_system(play_loop);
	}

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}
fn play_loop(asset_server: Res<AssetServer>, audio: Res<Audio>) {
	audio.play_looped(asset_server.load("default.ogg"));
}
// fn start_bg_music(assets: Res<AssetServer>, audio: Res<Audio>) {
// 	let background_handle = assets.get_handle("default.ogg");
// 	audio.play(background_handle);
// 	audio.set_volume(0.9);
// }
