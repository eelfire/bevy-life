#[allow(unused)]
mod actions;
mod audio;
mod loading;
mod menu;
mod pipeline;
mod player;

mod image;
// mod utils;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::pipeline::SmoothLifeNode;
use crate::player::PlayerPlugin;

#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_graph::RenderGraph;
use bevy::render::{RenderApp, RenderSet};
use bevy::{app::App, diagnostic::Diagnostics};
use image::SmoothLifeImage;
use pipeline::SmoothLifePipeline;
// use utils::SmoothLifeImage;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_plugin(LoadingPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(ActionsPlugin)
            .add_plugin(InternalAudioPlugin)
            .add_plugin(PlayerPlugin);

        #[cfg(debug_assertions)]
        {
            app.add_plugin(FrameTimeDiagnosticsPlugin)
                .add_plugin(LogDiagnosticsPlugin::default());
        }
    }
}

pub const SIM_SIZE: (u32, u32) = (1200, 600);
pub const WORKGROUP_SIZE: u32 = 8;

pub struct SmoothLifeShaderPlugin;
impl Plugin for SmoothLifeShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin)
            .add_plugin(ExtractResourcePlugin::<SmoothLifeImage>::default())
            .add_startup_system(setup)
            .add_system(window_fps);
        // .add_plugin(ExtractResourcePlugin::<SmoothLifeImage>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<SmoothLifePipeline>()
            .add_system(pipeline::queue_bind_group.in_set(RenderSet::Queue));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("smooth_life", SmoothLifeNode::default());
        render_graph.add_node_edge("smooth_life", bevy::render::main_graph::node::CAMERA_DRIVER);
    }
}

fn window_fps(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        let mut fps = 0.0;
        if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_smoothed) = fps_diagnostic.smoothed() {
                fps = fps_smoothed;
            }
        }

        // let mut frame_time = time.delta_seconds_f64();
        // if let Some(frame_time_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
        // {
        //     if let Some(frame_time_smoothed) = frame_time_diagnostic.smoothed() {
        //         frame_time = frame_time_smoothed;
        //     }
        // }

        text.sections[0].value = format!(
            // "This text changes in the bottom right - {fps:.1} fps, {frame_time:.3} ms/frame",
            "fps: {fps:.2}"
        );
    }
}

#[derive(Component)]
struct FpsText;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let image = image::create_image(SIM_SIZE.0, SIM_SIZE.1);
    // let image = utils::create_image(SIM_SIZE.0, SIM_SIZE.1);
    let image = images.add(image);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIM_SIZE.0 as f32, SIM_SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });

    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
            text: Text::from_section(
                "fps: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ),
            ..default()
        },
        FpsText,
    ));

    commands.insert_resource(image::SmoothLifeImage(image));
    // commands.insert_resource(utils::SmoothLifeImage(image));
}
