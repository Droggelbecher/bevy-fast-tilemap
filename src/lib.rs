//! GPU-accelerated tilemap functionality for bevy.
//! Aims at rendering tilemaps with lightning speed by using just a single quad per map (layer)
//! and offloading the actual rendering to GPU.
//! This should be faster than most other bevy tilemap implementations as of this writing.
//!
//! ## Features
//!
//! - Very high rendering performance (hundreds of fps, largely independent of map size)
//! - Tilemaps can be very large or have many "layers"
//! - Rectangular and isometric (axonometric) tile maps.
//! - Tile overlaps either by "dominance" rule or by perspective
//! - Optional custom mesh for which the map serves as a texture
//!
//! ## How it works
//!
//! The principle is probably not new but nonetheless quite helpful: The whole tilemap (-layer) is
//! rendered as a single quad and a shader cares for rendering the correct tiles at the correct
//! position.

pub mod bundle;
pub mod map;
pub mod map_builder;
pub mod map_uniform;
pub mod plugin;
pub mod shader;
pub mod tile_projection;

pub mod prelude {
    pub use super::bundle::*;
    pub use super::map::*;
    pub use super::map_builder::*;
    pub use super::map_uniform::*;
    pub use super::plugin::*;
    pub use super::tile_projection::*;

}
