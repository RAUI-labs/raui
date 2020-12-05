# RAUI [![Crates.io](https://img.shields.io/crates/v/raui.svg)](https://crates.io/crates/raui)[![Docs.rs](https://docs.rs/raui/badge.svg)](https://docs.rs/raui)
### Renderer Agnostic User Interface

## Table of contents
1. [About](#about)
1. [Architecture](#architecture)
    1. [Application](#application)
    1. [Widget](#widget)
    1. [Component Function](#component-function)
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
if let Ok(output) = application.render(&mut renderer) {
    println!("{}", output);
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
            .map(|slot| ListBoxItem {
                slot: slot.try_into().expect("Cannot convert slot to WidgetUnit!"),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        widget! {{{
            ListBox {
                items,
                ..Default::default()
            }
        }}}
    }
}
```
This may bring up a question: _**"If i use only functions and no objects to tell how to visualize UI, how do i keep some data between each render run?"**_.
For that you use _states_. State is a data that is stored between each processing call as long as given widget is alive (that means: as long as widget id stays the same between two processing calls, to make sure your widget stays the same, you use keys - if no key is assigned, system will generate one for your widget but that will make it possible to die at any time if for example number of widget children changes in your common parent, your widget will change its id when key wasn't assigned).
Some additional notes: While you use _properties_ to send information down the tree and _states_ to store widget data between processing cals, you can communicate with another widgets and host application using messages and signals! More than that, you can assign a closure that will be called when your widget gets unmounted from the tree.
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

widget_component! {
    // here we tell what data we want to unpack from
    // WidgetContext object passed to this function.
    button(key, props, unmounter, phase, state, messenger) {
        let label = props.read_cloned_or_default::<String>();

        if phase == WidgetPhase::Mount {
            drop(state.write(ButtonState { pressed: false }));
        }

        while let Some(msg) = messenger.read() {
            if let Some(msg) = msg.downcast_ref::<ButtonAction>() {
                let pressed = match msg {
                    ButtonAction::Pressed => true,
                    ButtonAction::Released => false,
                };
                println!("=== BUTTON ACTION: {:?}", msg);
                drop(state.write(ButtonState { pressed }));
                drop(signals.write(Box::new(*msg)));
            }
        }

        unmounter.listen(move |id, _state, _messages, _signals| {
            println!("=== BUTTON UNMOUNTED: {}", id.to_string());
        });

        widget!{
            (#{key} text: {label})
        }
    }
}
```

## Installation
There is a main `raui` crate that contains all of the project sub-crates to allow easy access to all features needed at any time, each enabled using Cargo `feature` flags (by default only `raui-core` subcrate is enabled).
```toml
[dependencies]
raui = { version = "0.5", features = ["all"] }
```

- `raui-core` - Core module that contains all RAUI essential logic.
  ```toml
  [dependencies]
  raui-core = "0.5"
  ```
- `raui-binary-renderer` - Renders RAUI widget tree into binary format.
  ```toml
  [dependencies]
  raui-binary-renderer = "0.5"
  ```
- `raui-html-renderer` - Renders RAUI widget tree into simple HTML format.
  ```toml
  [dependencies]
  raui-html-renderer = "0.5"
  ```
- `raui-json-renderer` - Renders RAUI widget tree into JSON format.
  ```toml
  [dependencies]
  raui-json-renderer = "0.5"
  ```
- `raui-ron-renderer` - Renders RAUI widget tree into RON format.
  ```toml
  [dependencies]
  raui-ron-renderer = "0.5"
  ```
- `raui-yaml-renderer` - Renders RAUI widget tree into YAML format.
  ```toml
  [dependencies]
  raui-yaml-renderer = "0.5"
  ```

## TODO
RAUI is still in early development phase, so prepare for these changes until v1.0:
- Reduce unnecessary allocations in processing pipeline.
- Find a solution (or make it a feature) for moving from trait objects data into strongly typed data for properties and states.
- Create renderer for at least one popular Rust graphics engine.
- Make C API bindings.
