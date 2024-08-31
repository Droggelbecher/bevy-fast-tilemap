
# Undesired horizontal and/or vertical lines / "screen door" effect

## Background

`bevy_fast_tilemap` relies heavily on shader logic to render your tilemap as fast as possible.
As such, however it also relies heavily on GPU floating point math which is usually
optimized a little more for speed rather than precision.
Depending on the precise numerics involved, due to float rounding maths and/or computational "shortcuts" like `-ffast-math` GPU (-compiler)s are taking to speed up floating point maths,
this can sometimes lead to pixels of the padding area being sampled undesirably, leading to horizontal and/or vertical lines between your tiles.

For in-depth technical discussion of some of the causes (and the mitigations that bevy_fast_tilemap already includes), see
[Issue 21](https://github.com/Droggelbecher/bevy-fast-tilemap/issues/21) and
[Issue 49](https://github.com/Droggelbecher/bevy-fast-tilemap/issues/49).

## Implemented Mitigations in `bevy_fast_tilemap`

* We aim to reduce matrix multiplications in shader code.
  Instead, we rely on vertex interpolation for obtaining the map position of each fragment.
* The default mesh is a single triangle, as we found a rectangle (two triangles) can
  lead to slight numerical offsets for specific camera positions.

## What you can do

Float computations are generally more accurate when dealing with powers of two (most of the time perfectly accurate), and less accurate for odd numbers that have no finite representation in binary such as 1/3.
Thus, your tile atlas in particular can have a huge influence on whether this issue appears.
This list is roughly in priority order. You may need to adhere to multiple of these; if you are torn between two options, prefer the earlier one.

### Make your tile atlas power-of-two sized

Or at least reduce the number of odd prime factors in its size at least in x-dimension.
Often this is as easy as reformatting your atlas a bit to not be eg. 6x8 but rather 8x8.
Since 0.8.1 your atlas doesn't need to fit a whole number of tiles anymore,
you can use `.with_n_tiles()` on the map builder to specificy how much of the space you actually want to use.

### Avoid custom meshes

Each triangle in the mesh may be found to
round slightly differently.
The default mesh is a single triangle since 0.8.1.

### Put tile boundaries on "nice" values (with few odd prime factors)

If you use a padding of 1, and a tile size of 64, tile 0 will be at 0, but tile 1 will be at 65
which is less accurate. If you don't work with overhang, chances are you don't actually need
padding so you can have your tiles at 0, 64, 128, etc..

### Stay clear of weird camera zoom values

More precisely, "weird" values for the scale factors of the camera2d transform.
Zoom factors like 3.129 seem to be one cause of these issues.
Again, stick to "nice" numbers if you can such as powers of two or at least numbers that
have few odd prime factors.

### Use unobtrusive padding

As a last resort, make your padding unobtrusive so the effect won't be noticable.

