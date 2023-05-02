
# Fast Tilemap for Bevy

[![Crates.io](https://img.shields.io/crates/v/bevy_fast_tilemap)](https://crates.io/crates/bevy_fast_tilemap)
[![docs](https://docs.rs/bevy_fast_tilemap/badge.svg)](https://docs.rs/bevy_fast_tilemap/)

GPU-accelerated tilemap functionality for [`bevy`](https://bevyengine.org/).
Aims at rendering tilemaps with lightning speed by using just a single quad per map (layer)
and offloading the actual rendering to GPU.
This should be faster than most other bevy tilemap implementations as of this writing.

## Features

- Very high rendering performance.
- Tilemaps can be very large or have many "layers"
- Rectangular and isometric tile maps.

## Screenshots

![layers](screenshots/updates.png)
![layers](screenshots/layers.png)
![iso](screenshots/iso.png)
![iso2](screenshots/iso2.png)

## How it works

The whole tilemap (-layer) is rendered as a single quad and a shader cares for rendering the correct
tiles at the correct position.

Thus each map layer works with two textures: One with integer data type, constructed and maintained
internally for storing for each tile position which tile index should be displayed there. And a
tile atlas that contains all the tiles which should be provided by you (see [assets/](assets/)).

## Limitations

- Only tested on Windows, no WASM support
- Overlapping "tiles" can not be rendered due to how the shader is designed, this may be
changed in the future.
- No direct animation support

## Related work

If you dont require all of `bevy_fast_tilemap`s performance and are looking for 
more features and maturity, take a look at 
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
```

## Bevy Compatibility

|bevy|bevy_fast_tilemap|
|---|---|
|0.10.1|0.1.0|
|0.10.1|0.2.0|
|0.10.1|0.3.0|
