use crate::map::{
    apply_map_transforms, configure_loaded_assets, log_map_events, update_loading_maps,
    update_map_vertex_attributes,
};
use bevy::{
    prelude::*,
    render::render_resource::{encase::internal::WriteInto, AsBindGroup, ShaderSize, ShaderType},
    sprite::Material2dPlugin,
};

use crate::{
    map::{DefaultUserData, Map},
    shader::{SHADER_CODE, SHADER_HANDLE},
};

/// Plugin for fast tilemap.
/// Add this to you app and then spawn one or multiple maps use [`crate::map_builder::MapBuilder`].
pub type FastTileMapPlugin = CustomFastTileMapPlugin<DefaultUserData>;

/// Plugin for fast tilemap.
/// Add this to you app and then spawn one or multiple maps use [`crate::map_builder::MapBuilder`].
#[derive(Default)]
pub struct CustomFastTileMapPlugin<UserData = DefaultUserData> {
    pub pre_sample_code: Option<String>,
    pub post_sample_code: Option<String>,
    pub user_data_struct: Option<String>,
    pub _user_data: std::marker::PhantomData<UserData>,
}

impl<UserData> Plugin for CustomFastTileMapPlugin<UserData>
where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<Map<UserData>>::default());
        let mut shaders = app.world.resource_mut::<Assets<Shader>>();

        let mut code = SHADER_CODE.to_string();
        let user_data_struct = self
            .user_data_struct
            .clone()
            .unwrap_or("x: u32,".to_string());
        code = code.replace("//#[user_data_struct]", &user_data_struct);

        if let Some(pre_sample_code) = &self.pre_sample_code {
            code = code.replace("//#[pre_sample_code]", pre_sample_code);
        }
        if let Some(post_sample_code) = &self.post_sample_code {
            code = code.replace("//#[post_sample_code]", post_sample_code);
        }
        shaders.insert(SHADER_HANDLE, Shader::from_wgsl(code, file!()));

        app.add_systems(
            Update,
            (
                (
                    configure_loaded_assets::<UserData>,
                    update_loading_maps::<UserData>,
                    log_map_events::<UserData>,
                )
                    .chain(),
                update_map_vertex_attributes::<UserData>,
            ),
        );
        app.add_systems(Update, apply_map_transforms::<UserData>);
    }
}
