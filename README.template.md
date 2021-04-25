# RAUI [![Crates.io](https://img.shields.io/crates/v/raui.svg)](https://crates.io/crates/raui)[![Docs.rs](https://docs.rs/raui/badge.svg)](https://docs.rs/raui)
### Renderer Agnostic User Interface

## Table of contents
1. [About](#about)
1. [Architecture](#architecture)
    1. [Application](#application)
    1. [Widget](#widget)
    1. [Component Function](#component-function)
    1. [Hooks](#hooks)
    1. [Layouting](#layouting)
    1. [Interactivity](#interactivity)
1. [Media](#media)
1. [Installation](#installation)
1. [Contribute](#contribute)
1. [Milestones](#milestones)

## About
RAUI (_spelled as **"ra"** (Egiptian god) + **"oui"** (french for "yes")_) is heavely inspired by **React** declarative UI composition and **UE4 Slate** widget components system.

The main idea behind RAUI architecture is to treat UI as another data that you transform into target renderable data format used by your rendering engine of choice.

## Architecture

### Application
It is the central point of user interrest. It performs whole UI processing logic. There you apply widget tree that wil be processed, send messages from host application to widgets and receive signals sent from widgets to host application.
```rust: source @ application
./src/tests.rs
```

### Widget
Widgets are divided into three categories:
- **Widget Node** - used as source UI trees (variant that can be either a component, unit or none)
  ```rust: source @ widget-node
  ./src/tests.rs
  ```
- **Widget Component** - you can think of them as Virtual DOM nodes, they store:
  - pointer to _component function_ (that process their data)
  - unique _key_ (that is a part of widget ID and will be used to tell the system if it should carry its _state_ to next processing run)
  - boxed cloneable _properties_ data (if component is a function, then properties are function arguments)
  - _listed slots_ (simply: widget children)
  - _named slots_ (similar to listed slots: widget children, but these ones have names assigned to them, so you can access them by name instead of by index)
- **Widget Unit** - an atomic element that renderers use to convert into target renderable data format for rendering engine of choice.
  ```rust: source @ widget-unit
  ./src/tests.rs
  ```

### Component Function
Component functions are static functions that transforms input data (properties, state or neither of them) into output widget tree (usually used to simply wrap another components tree under one simple component, where at some point the simplest components returns final _widget units_).
They work together as a chain of transforms - root component applies some properties into children components using data from its own properties or state.
```rust: source @ component-function
./src/tests.rs
```
This may bring up a question: _**"If i use only functions and no objects to tell how to visualize UI, how do i keep some data between each render run?"**_.
For that you use _states_. State is a data that is stored between each processing calls as long as given widget is alive (that means: as long as widget id stays the same between two processing calls, to make sure your widget stays the same, you use keys - if no key is assigned, system will generate one for your widget but that will make it possible to die at any time if for example number of widget children changes in your common parent, your widget will change its id when key wasn't assigned).
Some additional notes: While you use _properties_ to send information down the tree and _states_ to store widget data between processing cals, you can communicate with another widgets and host application using messages and signals!
More than that, you can use hooks to listen for widget life cycle and perform actions there.
It's worth noting that state uses _properties_ to hold its data, so by that you can for example attach multiple hooks that each of them uses different data type as widget state, this opens the doors to be very creative when combining different hooks that operate on the same widget.
```rust: source @ state
./src/tests.rs
```

### Hooks
Hooks are used to put common widget logic into separate functions that can be chained in widgets and another hooks (you can build a reusable dependency chain of logic with that).
Usually it is used to listen for life cycle events such as mount, change and unmount, additionally you can chain hooks to be processed sequentially in order they are chained in widgets and other hooks.
```rust: source @ hooks
./src/tests.rs
```
What happens under the hood:
- Application calls `button` on a node
    - `button` calls `use_button` hook
        - `use_button` calls `use_empty` hook
    - `use_button` logic is executed
- `button` logic is executed

### Layouting
_**TODO**_

RAUI exposes API (`Application::layout()`) to allow use of virtual-to-real coords mapping and custom layout engines to perform widget tree positioning data, which is later used by custom UI renderers to specify boxes where given widgets should be placed.
Every call to perform layouting will store a layout data inside Application, you can always access that data at any time.
There is a `DefaultLayoutEngine` that does this in a generic way.
If you find some part of its pipeline working different than what you've expected, feel free to create your custom layout engine!
```rust: source @ layouting
./src/tests.rs
```

### Interactivity
_**TODO**_

RAUI allows you to ease and automate interactions with UI by use of Interactions Engine - this is just a struct that implements `perform_interactions` method with reference to Application, and all you should do there is to send user input related messages to widgets.
There is `DefaultInteractionsEngine` that covers widget navigation, button and input field - actions sent from input devices such as mouse (or any single pointer), keyboard and gamepad. When it comes to UI navigation you can send raw `NavSignal` messages to the default interactions engine and despite being able to select/unselect widgets at will, you have typical navigation actions available: up, down, left, right, previous tab/screen, next tab/screen, also being able to focus text inputs and send text input changes to focused input widget. All interactive widget components that are provided by RAUI handle all `NavSignal` actions in their hooks, so all user has to do is to just activate navigation features for them (using `NavItemActive` unit props).
RAUI integrations that want to just use use default interactions engine should make use of this struct composed in them and call its `interact` method with information about what input change was made.
There is an example of that feature covered in Tetra integration crate (`TetraInteractionsEngine` struct).

**NOTE: Interactions engines should use layout for pointer events so make sure that you rebuild layout before you perform interactions!**
```rust: source @ interactivity
./src/tests.rs
```

## Media
- [`RAUI + Tetra In-Game`](https://github.com/PsichiX/raui/tree/master/demos/in-game)
  An example of an In-Game integration of RAUI with custom Material theme, using Tetra as a renderer.

  ![RAUI + Tetra In-Game](https://github.com/PsichiX/raui/blob/master/media/raui-tetra-in-game-material-ui.gif?raw=true)

- [`RAUI + Tetra todo app`](https://github.com/PsichiX/raui/tree/master/demos/todo-app)
  An example of TODO app with Tetra renderer and dark theme Material component library.

  ![RAUI + Tetra todo app](https://github.com/PsichiX/raui/blob/master/media/raui-tetra-todo-app-material-ui.gif?raw=true)

## Installation
There is a main `raui` crate that contains all of the project sub-crates to allow easy access to all features needed at any time, each enabled using Cargo `feature` flags (by default only `raui-core` subcrate is enabled).
```toml
[dependencies]
raui = { version = "*", features = ["all"] }
```

- `raui-core` - Core module that contains all RAUI essential logic.
  ```toml
  [dependencies]
  raui-core = "*"
  ```
- `raui-material` - Material Library module that contains themeable Material components for RAUI (`material` feature).
  ```toml
  [dependencies]
  raui-material = "*"
  ```
- `raui-tesselate-renderer` - Renders RAUI widget tree into Vertex + Index + Batch buffers (`tesselate` feature).
  ```toml
  [dependencies]
  raui-tesselate-renderer = "*"
  ```
- `raui-binary-renderer` - Renders RAUI widget tree into binary format (`binary` feature).
  ```toml
  [dependencies]
  raui-binary-renderer = "*"
  ```
- `raui-html-renderer` - Renders RAUI widget tree into simple HTML format (`html` feature).
  ```toml
  [dependencies]
  raui-html-renderer = "*"
  ```
- `raui-json-renderer` - Renders RAUI widget tree into JSON format (`json` feature).
  ```toml
  [dependencies]
  raui-json-renderer = "*"
  ```
- `raui-ron-renderer` - Renders RAUI widget tree into RON format (`ron` feature).
  ```toml
  [dependencies]
  raui-ron-renderer = "*"
  ```
- `raui-yaml-renderer` - Renders RAUI widget tree into YAML format (`yaml` feature).
  ```toml
  [dependencies]
  raui-yaml-renderer = "*"
  ```
- `raui-tetra-renderer` - Renders RAUI widget tree with Tetra renderer.
  ```toml
  [dependencies]
  raui-tetra-renderer = "*"
  ```
- `raui-ggez-renderer` - Renders RAUI widget tree with GGEZ renderer.
  NOTE: Author of GGEZ crate is looking for new maintainer - until that happen, new updates are on hold.
  ```toml
  [dependencies]
  raui-ggez-renderer = "*"
  ```

## Contribute
Any contribution that improves quality of the RAUI toolset is highly appreciated.
- If you have a feature request, create an Issue post and explain the goal of the feature along with the reason why it is needed and its pros and cons.
- Whenever you would like to create na PR, please create your feature branch from `next` branch so when it gets approved it can be simply merged using GitHub merge button
- All changes are staged into `next` branch and new versions are made out of its commits, master is considered stable/release branch.
- Changes should pass tests, you run tests with: `cargo test --all --features all`.
- This readme file is generated from `README.template.md` file using [MD Bakery CLI tool](https://github.com/PsichiX/md-bakery) that allows us to inject snippets of code from `/src/tests.rs` file to make sure all code examples are up-to-date, formatted, compiling and tested before new version gets published. To bake `README.md` you simply run: `mdbakery -i ./README.template.md -o ./README.md`.

## Milestones
RAUI is still in early development phase, so prepare for these changes until v1.0:
- [ ] Integrate RAUI into one public open source Rust game.
- [ ] Write documentation.
- [ ] Write MD book about how to use RAUI properly and make UI efficient.
- [ ] Props feature starts to look more like a micro ECS - make use of that and make custom allocator for them that would optimize frequent props creation/cloning (to clarify: it is not about using an actual ECS to store props, it is more about to figure out how ECS principles might help design efficient props allocator to avoid frequent props cloning).
- [ ] Implement VDOM diffing algorithm for tree rebuilding optimizations.
- [ ] Find a solution (or make it a feature) for moving from trait objects data into strongly typed data for properties and states.
- [ ] Make WASM/JS API bindings.
- [ ] Make C API bindings.

Things that now are done:
- [x] Add suport for layouting.
- [x] Add suport for interactions (user input).
- [x] Create renderer for GGEZ game framework.
- [x] Create basic user components.
- [x] Create basic Hello World example application.
- [x] Decouple shared props from props (don't merge them, put shared props in context).
- [x] Create TODO app as an example.
- [x] Create In-Game app as an example.
- [x] Create renderer for Oxygengine game engine.
- [x] Add complex navigation system.
- [x] Create scroll box widget.
- [x] Add "immediate mode UI" builder to give alternative to macros-based declarative mode UI building (with zero overhead, it is an equivalent to declarative macros used by default, immediate mode and declarative mode widgets can talk to each other without a hassle).
- [x] Add data binding property type to easily mutate data from outside of the application.
- [x] Create tesselation renderer that produces Vertex + Index + Batch buffers ready for mesh renderers.
- [x] Create renderer for Tetra game framework.
- [x] Move from `widget_component!` and `widget_hook!` macro rules to `pre_hooks` and `post_hooks` function attributes.
- [x] Add derive `PropsData` and `MessageData` procedural macros to gradually replace the need to call `implement_props_data!` and `implement_message_data!` macros.
- [x] Add support for portals - an easy way to "teleport" sub-tree into another tree node (useful for modals and drag & drop).
