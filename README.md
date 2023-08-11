
# Fast Tilemap for Bevy

[![Crates.io](https://img.shields.io/crates/v/bevy_fast_tilemap)](https://crates.io/crates/bevy_fast_tilemap)
[![docs](https://docs.rs/bevy_fast_tilemap/badge.svg)](https://docs.rs/bevy_fast_tilemap/)

Lightning fast tilemaps for [`bevy`](https://bevyengine.org/).

## Features

- Very high rendering performance (hundreds of fps, largely independent of map size)
- Tilemaps can be very large or have many "layers"
- Rectangular and isometric (axonometric) tile maps.
- Tiles can overlap either by "dominance" rule or by perspective
- Optional custom mesh for which the map serves as a texture

## Screenshots

![iso_perspective](screenshots/iso_perspective.png)
![custom_mesh](screenshots/custom_mesh.png)

Checkout ![screenshots/](screenshots/) for more.

## How it works

The whole map is rendered as a single quad and a custom shader cares for rendering the
correct tiles at the correct position.

Thus each map works with two textures: One with integer data type, constructed and maintained
internally for storing for each tile position which tile index should be displayed there. And the
other being a tile atlas that contains all the tiles. This one should be provided by you (see [assets/](assets/) for
atlas examples).

As of this writing, this should be (much) faster than most other bevy tilemap implementations out
there.

## Limitations

- Only tested on Windows, no WASM support
- No direct animation support, but you can easily update the tilemap in regular intervals
  to achieve the same (see [Animation Example](examples/animation.rs))
- Currently no support for rotating or scaling the entity holding the map (it will not look like you'd expect).
  (You can of course still zoom/rotate the camera to achieve any such effect)

## Related work

If you dont require all of `bevy_fast_tilemap`s performance and are looking for an approach that
supports some more tile shapes and allows to treat each tile as a separate entity, take a look at
[bevy_ecs_tilemap](https://github.com/StarArawn/bevy_ecs_tilemap/) which (among others) inspired
this work.

## Examples

Check out the [examples/](examples/) folder to get an overview.
You can run the examples like this:

```bash
cargo run --example updates
cargo run --example layers
cargo run --example iso
cargo run --example iso2
cargo run --example bench
...
```

## Bevy Compatibility

|bevy|bevy_fast_tilemap|
|---|---|
|0.10.1|0.1.0|
|0.10.1|0.2.0|
|0.10.1|0.3.0|
|0.10.1|0.4.0|
|0.11.0|0.5.0|
