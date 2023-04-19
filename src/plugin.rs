use crate::{
    map::{check_map_ready_events, MapReadyEvent},
    pipeline::FastTileMapPipeline,
};
use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    render::{
        render_asset::PrepareAssetSet,
        render_phase::AddRenderCommand, 
        render_resource::SpecializedRenderPipelines,
        RenderApp, RenderSet,
    },
};

use crate::{
    extract::extract_fast_tilemap,
    prepare::prepare_fast_tilemap,
    queue::{queue_fast_tilemap, DrawFastTileMap},
    shader::{SHADER_CODE, SHADER_HANDLE},
};

pub struct FastTileMapPlugin;

impl Default for FastTileMapPlugin {
    fn default() -> FastTileMapPlugin {
        FastTileMapPlugin {}
    }
}

impl Plugin for FastTileMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MapReadyEvent>()
            .add_system(check_map_ready_events);

        let mut shaders = app.world.resource_mut::<Assets<Shader>>();
        shaders.set_untracked(SHADER_HANDLE, Shader::from_wgsl(SHADER_CODE));

        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .init_resource::<FastTileMapPipeline>()
            .init_resource::<SpecializedRenderPipelines<FastTileMapPipeline>>()
            .add_render_command::<Transparent2d, DrawFastTileMap>()
            .add_system(extract_fast_tilemap.in_schedule(ExtractSchedule))
            .add_system(
                prepare_fast_tilemap
                    .in_set(RenderSet::Prepare)
                    .after(PrepareAssetSet::PreAssetPrepare),
            )
            .add_system(queue_fast_tilemap.in_set(RenderSet::Queue));
    }
}
