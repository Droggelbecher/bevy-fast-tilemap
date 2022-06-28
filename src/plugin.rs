use bevy::{prelude::*, sprite::Material2dPlugin};
use crate::map::{MapLayerMaterial, MapReadyEvent, check_map_ready_events};
use crate::material::{TILEMAP_SHADER_HANDLE, TILEMAP_SHADER};

// TODO: shaders should not be in assets dir

pub struct FastTileMapPlugin;

impl Default for FastTileMapPlugin {
    fn default() -> FastTileMapPlugin {
        FastTileMapPlugin {}
    }
}

impl Plugin for FastTileMapPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.resource_mut::<Assets<Shader>>();
        shaders.set_untracked(
            TILEMAP_SHADER_HANDLE,
            Shader::from_wgsl(TILEMAP_SHADER),
        );

        app.add_plugin(Material2dPlugin::<MapLayerMaterial>::default())
            .add_event::<MapReadyEvent>()
            .add_system(check_map_ready_events);
    }
}

