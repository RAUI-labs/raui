# Setting Up

We're going to get setup with a blank Tetra game project with RAUI hooked up to render to it.

## Creating the Project

First we need to create a new Rust project and get our dependencies setup.

Create a new cargo project:

```bash
cargo new --bin guide
```

Then add the following dependencies to the `Cargo.toml`:

```toml
[dependencies]
# The sdl2_bundled feature tells the crate to download and install SDL2 so you
# don't have to have it installed
tetra = { version = "0.6", features = [] }
raui = "0.29"
raui-tetra-renderer = "0.29"
```

## Initializing Tetra

Next we need to setup our Tetra game window with the RAUI renderer hooked up to it.

```rust
{{#include ./rust/01_setting_up.rs}}
```

## Summary

At this point you should be able to `cargo run` and have a blank window pop up, but we're not here for a blank window, so let's go put some GUI on the screen!
