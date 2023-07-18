use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        query::ROQueryItem,
        system::{lifetimeless::Read, SystemParamItem},
    },
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_phase::{
            DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{PipelineCache, SpecializedRenderPipelines},
        view::{ExtractedView, VisibleEntities},
    },
    sprite::{
        DrawMesh2d, Mesh2dHandle, Mesh2dPipelineKey, Mesh2dUniform, SetMesh2dBindGroup,
        SetMesh2dViewBindGroup,
    },
    utils::FloatOrd,
};

use crate::{pipeline::MapPipeline, prepare::PreparedMap};

pub type DrawMap = (
    // Set the pipeline
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    SetMapBindGroup<1>,
    // Set the mesh uniform as bind group 1
    SetMesh2dBindGroup<2>,
    // Draw the mesh
    DrawMesh2d,
);

pub struct SetMapBindGroup<const I: usize>();

impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetMapBindGroup<I> {
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<PreparedMap>;

    fn render<'w>(
        _item: &P,
        _view: (),
        prepared_map: ROQueryItem<'w, Self::ItemWorldQuery>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &prepared_map.bind_group.bind_group, &[]);
        RenderCommandResult::Success
    }
}

/// Queue map for rendering in the views.
/// This calls `MapPipeline.specialize()` and instantiates the draw functions
/// to schedule them onto the GPU.
///
/// This is a system in the render app.
pub fn queue_fast_tilemap(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    map_pipeline: Res<MapPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<MapPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    map: Query<(&Mesh2dHandle, &Mesh2dUniform), With<PreparedMap>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    mut views: Query<(
        &VisibleEntities,
        &mut RenderPhase<Transparent2d>,
        &ExtractedView,
    )>,
    msaa: Res<Msaa>,
) {
    if map.is_empty() {
        return;
    }

    // Iterate each view (a camera is a view)
    for (visible_entities, mut transparent_phase, view) in &mut views {
        let draw_map = transparent_draw_functions.read().id::<DrawMap>();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples())
            | Mesh2dPipelineKey::from_hdr(view.hdr);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            if let Ok((mesh2d_handle, mesh2d_uniform)) = map.get(*visible_entity) {
                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
                    mesh2d_key |=
                        Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);
                }

                let pipeline_id = pipelines.specialize(&pipeline_cache, &map_pipeline, mesh2d_key);

                let mesh_z = mesh2d_uniform.transform.w_axis.z;
                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
                    draw_function: draw_map,
                    pipeline: pipeline_id,
                    // The 2d render items are sorted according to their z value before rendering,
                    // in order to get correct transparency
                    sort_key: FloatOrd(mesh_z),
                    // This material is not batched
                    batch_range: None,
                });
            }
        }
    } // for visible_entities
} // queue_fast_tilemap()
