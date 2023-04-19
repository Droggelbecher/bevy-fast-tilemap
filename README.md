
Simple and fast tilemap functionality for bevy.

# Project status

As of this writing, this code runs with bevy 0.10.1 on my Windows machine to my satisfaction.

That being said, I can currently not afford the time to ensure it works for everyone else or other
platforms or with your favourite features.

You're very welcome to use this in whatever way you want, but be warned the maturity is rather
experimental.

If you have ideas to extend this, you're very welcome to shoot me a PR, even better lets chat about
it first.

# How it works

The principle is probably not new but nonetheless quite helpful:
The whole tilemap (-layer) is rendered as a single quad and a shader cares for rendering the correct
tiles at the correct position.

# Limitions

Currently this has only been tested on Windows, so dont expect other platforms to work.
There is currently no support for non-square tiles (isometric/hex/etc...).
It should for many cases be possible to simulate these by having more square tiles (& some logic) that together express your shapes.
