use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioPlugin, AudioSource};

pub struct AudioState {
	background_handle: Handle<AudioSource>,
}

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugin(AudioPlugin)
			.add_startup_system_to_stage(StartupStage::PreStartup, load_audio)
			.add_startup_system(start_bg_music);
	}
}

fn load_audio(mut commands: Commands, assets: Res<AssetServer>) {
	let background_handle: Handle<AudioSource> = assets.load("sounds/background/default.ogg");
	commands.insert_resource(AudioState { background_handle });
}

fn start_bg_music(audio: Res<AudioState>, player: Res<Audio>) {
	player.play_looped(audio.background_handle.clone());
}
