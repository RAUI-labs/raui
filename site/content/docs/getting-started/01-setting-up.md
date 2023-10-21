+++
title = "Setting Up"
description = "Learn how to get a window setup so RAUI can render to it."
draft = false
weight = 1
template = "docs/page.html"
slug = "setting-up"

[extra]
lead = "First we're going to get a window setup so RAUI can render to it."
toc = true
top = false
+++

## Creating the Project

Let's create a new Rust project and add our dependencies.

Create a new cargo project:

```bash
cargo new --bin my_project
```

Then add the following dependencies to the `Cargo.toml`:

{{ toml_code_snippet(path="rust/guide_01/Cargo.toml", start=7) }}

## Initializing The Window

Next we need to setup our UI window. Using the [`raui-app`] crate this is super easy!

In most cases you will probably want to integrate RAUI with a game engine or other renderer, and in that case you would not use [`raui-app`], you would use an integration crate like [`raui-tesselation-renderer`]. For now, though, we want to get right into RAUI without having to worry about integrations.

[`raui-app`]: https://docs.rs/raui-app
[`raui-tesselation-renderer`]: https://docs.rs/raui-tesselation-renderer

Go ahead and add the following to your `main.rs` file:

{{ rust_code_snippet(path="rust/guide_01/src/main.rs") }}

We don't add any widgets yet, we'll get to that in the next step. At this point you should be able to `cargo run` and have a blank window pop up!

OK, not that cool. We're not here for a blank window, so let's go put some GUI on the screen!

> **Note:** You can find the full code for this chapter [here](https://github.com/RAUI-labs/raui/tree/master/site/rust/guide_01)