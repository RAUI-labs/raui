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
1. [Media](#media)
1. [Installation](#installation)
1. [TODO](#todo)

## About
RAUI is heavely inspired by **React** declarative UI composition and **UE4 Slate** widget components system.

The main idea behind RAUI architecture is to treat UI as another data that you transform into target renderable data format used by your rendering engine of choice.

## Architecture

### Application
It is the central point of user interrest. It performs whole UI processing logic. There you apply widget tree that wil be processed, send messages from host application to widgets and receive signals sent from widgets to host application.
```rust
let mut application = Application::new();
let tree = widget! {
    (app {
        title = (title_bar: {"Hello".to_owned()})
        content = (vertical_box [
            (#{"hi"} button: {"Say hi!".to_owned()})
            (#{"exit"} button: {"Close".to_owned()})
        ])
    })
};
let mut renderer = HtmlRenderer::default();
application.apply(tree);
application.layout(view, &mut DefaultLayoutEngine);
if let Ok(output) = application.render(&mut renderer) {
    println!("OUTPUT: {}", output);
}
```

### Widget
Widgets are divided into three categories:
- **Widget Node** - used as source UI trees (variant that can be either a component, unit or none)
  ```rust
  widget! {
      (app {
          title = (title_bar: {"Hello".to_owned()})
          content = (vertical_box [
              (#{"hi"} button: {"Say hi!".to_owned()})
              (#{"exit"} button: {"Close".to_owned()})
          ])
      })
  }
  ```
- **Widget Component** - you can think of them as Virtual DOM nodes, they store:
  - pointer to _component function_ (that process their data)
  - unique _key_ (that is a part of widget ID and will be used to tell the system if it should carry its _state_ to next processing run)
  - boxed cloneable _properties_ data (if component is a function, then properties are function arguments)
  - _listed slots_ (simply: widget children)
  - _named slots_ (similar to listed slots: widget children, but these ones have names assigned to them, so you can access them by name instead of by index)
- **Widget Unit** - an atomic element that renderers use to convert into target renderable data format for rendering engine of choice.
  ```rust
  widget!{{{
      TextBox {
          text: "Hello World".to_owned(),
          ..Default::default()
      }
  }}}
  ```

### Component Function
Component functions are static functions that transforms input data (properties, state or neither of them) into output widget tree (usually used to simply wrap another components tree under one simple component, where at some point the simplest components returns final _widget units_).
They work together as a chain of transforms - root component applies some properties into children components using data from its own properties or state.
```rust
#[derive(Debug, Default, Copy, Clone)]
struct AppProps {
    pub index: usize,
}
implement_props_data!(AppProps);

widget_component! {
    app(props, named_slots) {
        unpack_named_slots!(named_slots => { title, content });
        let index = props.read::<AppProps>().map(|p| p.index).unwrap_or(0);

        widget! {
            (#{index} vertical_box [
                {title}
                {content}
            ])
        }
    }
}

widget_component! {
    vertical_box(key, listed_slots) {
        let items = listed_slots
            .into_iter()
            .map(|slot| FlexBoxItem {
                slot: slot.try_into().expect("Cannot convert slot to WidgetUnit!"),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        widget! {{{
            FlexBox {
                items,
                ..Default::default()
            }
        }}}
    }
}
```
This may bring up a question: _**"If i use only functions and no objects to tell how to visualize UI, how do i keep some data between each render run?"**_.
For that you use _states_. State is a data that is stored between each processing call as long as given widget is alive (that means: as long as widget id stays the same between two processing calls, to make sure your widget stays the same, you use keys - if no key is assigned, system will generate one for your widget but that will make it possible to die at any time if for example number of widget children changes in your common parent, your widget will change its id when key wasn't assigned).
Some additional notes: While you use _properties_ to send information down the tree and _states_ to store widget data between processing cals, you can communicate with another widgets and host application using messages and signals!
More than that, you can use hooks to listen for widget life cycle and perform actions there.
```rust
#[derive(Debug, Default, Copy, Clone)]
struct ButtonState {
    pub pressed: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ButtonAction {
    Pressed,
    Released,
}

widget_hook! {
    use_button(key, life_cycle) {
        let key_ = key.to_owned();
        life_cycle.mount(move |_, state, _, _| {
            drop(state.write(ButtonState { pressed: false }));
        });

        let key_ = key.to_owned();
        life_cycle.change(move |_, _, state, messenger, signals| {
            for msg in messenger.messages {
                if let Some(msg) = msg.downcast_ref::<ButtonAction>() {
                    let pressed = match msg {
                        ButtonAction::Pressed => true,
                        ButtonAction::Released => false,
                    };
                    drop(state.write(ButtonState { pressed }));
                    drop(signals.write(Box::new(*msg)));
                }
            }
        });
    }
}

widget_component! {
    button(key, props, state, messenger, signals) [use_button] {
        let label = props.read_cloned_or_default::<String>();

        widget!{
            (#{key} text: {label})
        }
    }
}
```

### Hooks
Hooks are used to put common widget logic into separate functions that can be chained in widgets and another hooks (you can build a reusable dependency chain of logic with that).
Usually it is used to listen for life cycle events such as mount, change and unmount, additionally you can chain hooks to be processed sequentially in order they are chained in widgets and other hooks.
```rust
widget_hook! {
    use_empty {}
}

widget_hook! {
    use_button(key, life_cycle) [use_empty] {
        let key_ = key.to_owned();
        life_cycle.mount(move |_, _, state, _, _| {
            drop(state.write(ButtonState { pressed: false }));
        });

        let key_ = key.to_owned();
        life_cycle.change(move |_, _, state, messenger, signals| {
            for msg in messenger.messages {
                if let Some(msg) = msg.downcast_ref::<ButtonAction>() {
                    let pressed = match msg {
                        ButtonAction::Pressed => true,
                        ButtonAction::Released => false,
                    };
                    drop(state.write(ButtonState { pressed }));
                    drop(signals.write(Box::new(*msg)));
                }
            }
        });
    }
}

widget_component! {
    button(key, props) [use_button] {
        widget!{
            (#{key} text: {props})
        }
    }
}
```
What happens under the hood:
- Application calls `button` on a node
    - `button` calls `use_button` hook
        - `use_button` calls `use_empty` hook
    - `use_button` logic is executed
- `button` logic is executed

### Layouting
_**TODO**_

RAUI exposes API (`Application::layout()`) to allow use of custom layout engines to perform widget tree positioning data, which is later used by custom UI renderers to specify boxes where given widgets should be placed.
Every call to perform layouting will store a layout data inside Application, you can always access that data at any time.
There is a `DefaultLayoutEngine` that does this in a generic way.
If you find some part of its pipeline working different than what you've expected, feel free to create your custom layout engine!
```rust
let mut application = Application::new();
application.apply(tree);
application.process();
if application.layout(view, &mut DefaultLayoutEngine).is_ok() {
    println!("LAYOUT:\n{:#?}", application.layout_data());
}
```

## Media
- `GGEZ Hello World` with vertical flex box, text box, grid box and image box.
  ![GGEZ Hello World](https://github.com/PsichiX/raui/blob/master/media/ggez-hello-world.png?raw=true)

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
- `raui-ggez-renderer` - Renders RAUI widget tree with GGEZ renderer.
  ```toml
  [dependencies]
  raui-ggez-renderer = "*"
  ```

## TODO
RAUI is still in early development phase, so prepare for these changes until v1.0:
- Create renderer for at least one popular Rust graphics engine.
- Create TODO app as an example.
- Reduce unnecessary allocations in processing pipeline.
- Find a solution (or make it a feature) for moving from trait objects data into strongly typed data for properties and states.
- Make C API bindings.
