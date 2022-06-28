use bevy::{
    math::{vec2, ivec2},
    prelude::*, reflect::TypeUuid
};
use std::ops::{Index, IndexMut};


/// Map, of size `size` tiles.
/// The actual tile data is stored in MapLayer components and Images in the asset system
/// and connected to this Map via bevys parent/child relationships for entities.
#[derive(Debug, Component, Clone)]
pub struct Map {
    pub(crate) size: IVec2,
    pub(crate) tile_size: Vec2,
    pub(crate) layers: Vec<Entity>,
}

impl Default for Map {
    fn default() -> Map {
        Map {
            size: ivec2(1024, 1024),
            tile_size: vec2(16., 16.),
            layers: vec![],
        }
    }
}

impl Map {

    /// Size of this map in tiles.
    pub fn size(&self) -> IVec2 {
        self.size
    }

    pub fn layers(&self) -> Vec<Entity> {
        self.layers.clone()
    }

    pub fn world_to_index(&self, world: Vec2) -> Vec2 {
        // TODO: Where to get tile size, should we maybe just set it on map level?
        world / self.tile_size * vec2(1., -1.) + self.size.as_vec2() / 2.
    }

    /// Get mutable access to map layers via a `MapIndexer`.
    /// For this needs to mutably borrow the contained `MapLayer`s
    /// and the associated `Image`s.
    pub fn get_mut<'a>(
        &self,
        layers: &mut Query<&mut MapLayer>,
        materials: &Assets<MapLayerMaterial>,
        images: &'a mut Assets<Image>,
    ) -> Result<MapIndexer<'a>, &'a str> {
        let mut layer_indexers: Vec<LayerIndexer> = vec![];

        unsafe { // TODO: Reduce the amount of unsafe code
            for layer_entity in self.layers.iter() {
                let layer = layers
                    .get_mut(*layer_entity)
                    .or(Err("Could not load layer from query"))?;
                let material = materials
                    .get(layer.material.clone())
                    .ok_or("Layer material not yet loaded")?;

                let image = images
                    .get_mut(material.map_texture.clone())
                    .ok_or("Map texture not yet loaded")? as *mut _;

                let tint_image = images
                    .get_mut(material.tint_texture.clone())
                    .ok_or("Tint texture not yet loaded")? as *mut _;

                layer_indexers.push(LayerIndexer::<'a> {
                    image: &mut *image,
                    tint_image: &mut *tint_image,
                    size: self.size,
                })
            }

            Ok(MapIndexer {
                layers: layer_indexers,
            })
        } // unsafe
    } // get_mut()

} // impl Map

/// Single Map layer, wrapper around material that stores the per-layer data.
/// TODO: Can we remove this and just store handles to MapLayerMaterials instead?
#[derive(Component)]
pub struct MapLayer {
    pub material: Handle<MapLayerMaterial>,
}

/// Map indexer, groups together the layer indexers for a particular map
pub struct MapIndexer<'a> {
    layers: Vec<LayerIndexer<'a>>,
}

impl<'a> Index<usize> for MapIndexer<'a> {
    type Output = LayerIndexer<'a>;

    fn index(&self, i: usize) -> &Self::Output {
        &self.layers[i]
    }
}

impl<'a> IndexMut<usize> for MapIndexer<'a> {
    fn index_mut(&mut self, i: usize) -> &mut LayerIndexer<'a> {
        &mut self.layers[i]
    }
}

/// Holds mut refs to image and tint image for this layer
pub struct LayerIndexer<'a> {
    image: &'a mut Image,
    tint_image: &'a mut Image,
    size: IVec2,
}

impl<'a> LayerIndexer<'a> {
    pub fn set_tint(&self, p: IVec2, color: [f32; 4]) {
        let idx = p.y as isize * self.size.x as isize + p.x as isize;
        unsafe {
            let ptr = self.tint_image.data.as_ptr() as *mut u8;
            *ptr.offset(idx * 4) = (color[0] * 255.) as u8;
            *ptr.offset(idx * 4 + 1) = (color[1] * 255.) as u8;
            *ptr.offset(idx * 4 + 2) = (color[2] * 255.) as u8;
            *ptr.offset(idx * 4 + 3) = (color[3] * 255.) as u8;
        }
    }
}

impl<'a> Index<IVec2> for LayerIndexer<'a> {
    type Output = u16;
    fn index(&self, i: IVec2) -> &Self::Output {
        let idx = i.y as isize * self.size.x as isize + i.x as isize;
        unsafe {
            let ptr = self.image.data.as_ptr() as *const u16;
            &*ptr.offset(idx)
        }
    }
}

impl<'a> IndexMut<IVec2> for LayerIndexer<'a> {
    fn index_mut(&mut self, i: IVec2) -> &mut u16 {
        let idx = i.y as isize * self.size.x as isize + i.x as isize;
        unsafe {
            let ptr = self.image.data.as_ptr() as *mut u16;
            &mut *ptr.offset(idx)
        }
    }
}


#[derive(Debug, Clone, TypeUuid)]
#[uuid = "0797341e-ddc5-11ec-ac9f-00155d74ab53"]
pub struct MapLayerMaterial {
    pub map_texture: Handle<Image>,
    pub tint_texture: Handle<Image>,
    pub tiles_texture: Handle<Image>,
    pub tile_ids: u32,
    pub ready: bool,
    pub(crate) tile_size: Vec2,
}

/// Signals that `self.map` has been fully loaded (materials & images),
/// so all layer's .get and .set methods can be used.
#[derive(Debug)]
pub struct MapReadyEvent {
    pub map: Entity,
}

/// Check if all images and materials of a `Map` are loaded,
/// when thats the case, send out a `MapReadyEvent`.
pub fn check_map_ready_events(
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut materials: ResMut<Assets<MapLayerMaterial>>,
    images: ResMut<Assets<Image>>,
    mut send_map_ready_event: EventWriter<MapReadyEvent>,
    mut maps: Query<(&Map, Entity, &Children)>,
    mut map_layers: Query<&mut MapLayer>,
) {
    for (map, map_entity, children) in maps.iter_mut() {
        let mut all_layers_ready = true;
        let mut something_changed = false;

        for ev in ev_asset.iter() {
            for layer_entity in children.iter() {
                let layer = match map_layers.get_mut(*layer_entity) {
                    Err(_) => continue,
                    Ok(x) => x,
                };

                let mut material = match materials.get_mut(layer.material.clone()) {
                    None => continue,
                    Some(x) => x,
                };

                match ev {
                    AssetEvent::Created { handle }
                        if handle.id == material.map_texture.id
                            || handle.id == material.tint_texture.id
                            || handle.id == material.tiles_texture.id =>
                    {
                        if images.get(material.map_texture.clone()).is_some()
                            && images.get(material.tint_texture.clone()).is_some()
                            && images.get(material.tiles_texture.clone()).is_some()
                        {
                            let tiles_texture = images.get(material.tiles_texture.clone()).unwrap();
                            let s = tiles_texture.size() / map.tile_size;
                            material.tile_ids = s.x as u32 * s.y as u32;

                            material.ready = true;
                            something_changed = true;
                        } // if
                    } // AssetEvent::Created
                    _ => {}
                } // match
                all_layers_ready = all_layers_ready && material.ready;
            } // for ev
        } // layer

        if something_changed && all_layers_ready {
            send_map_ready_event.send(MapReadyEvent { map: map_entity })
        }
    } // map
} // handle_tilemap_events()
