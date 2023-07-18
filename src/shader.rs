use bevy::{prelude::*, reflect::TypeUuid};

pub const SHADER_CODE: &str = include_str!("../assets/tilemap_shader.wgsl");
pub const SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 15375856360518374895);
