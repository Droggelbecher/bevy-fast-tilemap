
//! GPU-accelerated tilemap functionality for bevy.
//! Aims at rendering tilemaps with lightning speed by using just a single quad per map (layer)
//! and offloading the actual rendering to GPU.
//! This should be faster than most other bevy tilemap implementations as of this writing.
//!
//! ## Features
//! 
//! - Very high rendering performance.
//! - Tilemaps can be very large or have many "layers"
//! - Rectangular and isometric tile maps.
//! 
//! ## How it works
//! 
//! The principle is probably not new but nonetheless quite helpful: The whole tilemap (-layer) is
//! rendered as a single quad and a shader cares for rendering the correct tiles at the correct
//! position.

pub mod map;
pub mod map_uniform;
pub mod map_builder;
pub mod bundle;
pub mod plugin;
pub mod pipeline;
pub mod shader;
pub mod extract;
pub mod prepare;
pub mod queue;
pub mod tile_projection;

pub mod prelude {
    pub use crate::bundle::MapBundle;
    pub use crate::map::{Map, MapIndexer, MapReadyEvent, MeshManagedByMap};
    pub use crate::plugin::FastTileMapPlugin;
    pub use crate::tile_projection::{TileProjection, IDENTITY, AXONOMETRIC};
}

pub use crate::bundle::MapBundle;
pub use crate::map::{Map, MapIndexer, MapReadyEvent, MeshManagedByMap};
pub use crate::plugin::FastTileMapPlugin;
pub use crate::tile_projection::{TileProjection, IDENTITY, AXONOMETRIC};
