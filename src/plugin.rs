use crate::{
    map::{check_map_ready_events, MapReadyEvent},
    pipeline::MapPipeline,
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
    queue::{queue_fast_tilemap, DrawMap},
    shader::{SHADER_CODE, SHADER_HANDLE},
};

/// Plugin for fast tilemap.
/// Add this to you app and then spawn one or multiple maps use `MapDescriptor`.
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
            .init_resource::<MapPipeline>()
            .init_resource::<SpecializedRenderPipelines<MapPipeline>>()
            .add_render_command::<Transparent2d, DrawMap>()
            .add_system(extract_fast_tilemap.in_schedule(ExtractSchedule))
            .add_system(
                prepare_fast_tilemap
                    .in_set(RenderSet::Prepare)
                    .after(PrepareAssetSet::PreAssetPrepare),
            )
            .add_system(queue_fast_tilemap.in_set(RenderSet::Queue));
    }
}
