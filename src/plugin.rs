use crate::map::{log_map_events, update_loading_maps, update_map_vertex_attributes};
use bevy::{
    prelude::*,
    render::render_resource::{encase::internal::WriteInto, AsBindGroup, ShaderSize, ShaderType},
    sprite::Material2dPlugin,
};

use crate::{
    map::{DefaultUserData, Map},
    shader::SHADER_CODE,
};

/// Implement this trait to customize the shader code and user data.
pub trait Customization: Sync + Send + 'static + TypePath + Clone
{
    const CUSTOM_SHADER_CODE: &'static str;
    const SHADER_HANDLE: Handle<Shader>;
    type UserData: AsBindGroup
        + Reflect
        + Clone
        + TypePath
        + ShaderType
        + WriteInto
        + ShaderSize
        + Default;
}

/// Default custumization that will use the default user data and shader code.
#[derive(Clone, TypePath, Default)]
pub struct NoCustomization;

impl Customization for NoCustomization {
    const CUSTOM_SHADER_CODE: &'static str = r#"
        struct UserData {
            dummy: u32,
        };

        fn sample_tile(in: ExtractIn) -> vec4<f32> {
            return sample_tile_at(in.tile_index, in.tile_position, in.tile_offset);
        }
    "#;
    const SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(15375856360518374895);
    type UserData = DefaultUserData;
}

/// Plugin for fast tilemap.
/// Add this to you app and then spawn one or multiple maps use [`crate::map_builder::MapBuilder`].
pub type FastTileMapPlugin = CustomFastTileMapPlugin<NoCustomization>;

/// Plugin for fast tilemap.
/// Add this to you app and then spawn one or multiple maps use [`crate::map_builder::MapBuilder`].
#[derive(Default)]
pub struct CustomFastTileMapPlugin<C: Customization = NoCustomization> {
    _customization: std::marker::PhantomData<C>,
}

impl<C: Customization> Plugin for CustomFastTileMapPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<Map<C>>::default());
        let mut shaders = app.world.resource_mut::<Assets<Shader>>();

        let mut code = SHADER_CODE.to_string();

        code = code.replace("#[user_code]", C::CUSTOM_SHADER_CODE);

        shaders.insert(C::SHADER_HANDLE, Shader::from_wgsl(code, file!()));

        app.add_systems(
            Update,
            (
                (update_loading_maps::<C>, log_map_events::<C>).chain(),
                update_map_vertex_attributes::<C>,
            ),
        );
    }
}
