# Your First Component

Now we are going to create our own RAUI widget component. Before we do that, though, let's explain what the different kinds of widgets are in RAUI.

## Widget Types

There are three different kinds of widgets in RAUI:

### `WidgetNode`'s

A `WidgetNode` represents any kind of widget in RAUI: `None`, `WidgetUnit`, or `Widgetcomponent`. `WidgetNode`'s can be used to represent entire _widget trees_ because the `WidgetUnit`'s and `WidgetComponent`'s inside of them can each contain other `WidgetUnit`'s or `WidgetComponents`'s as children.

### `WidgetUnit`'s

`WidgetUnit`'s are primitive widgets such as `TextBox`, `ImageBox`, or `FlexBox`. There are a small number of different kinds of `WidgetUnit`'s and it is the job of the RAUI rendering backends to be able to render each different kind of `WidgetUnit`. They are the building-blocks for more complicated components.

### `WidgetComponent`'s

`WidgetComponent`'s are  functions that process their properties and child nodes and use that information to return a new `WidgetNode`. Components can combine other components and widget units to create complicated UIs out of small, modular components.

## Writing a Component

Now let's write our first component! Let's create a new module to put our UI code in:

**`ui.rs`:**

```rust
{{#include ./rust/02_your_first_widget.rs}}
```

Then back in `main.rs` we need to include our new `ui` module:

**`main.rs`:**

```rust
// Above fn main() { ... }
mod ui;
use ui::my_first_widget;
```

And we need to add our widget to the widget tree we created in our `App::new` function:

**`main.rs`:**

```rust
impl App {
    fn new(context: &mut tetra::Context) -> Result<Self, tetra::TetraError> {
        // This is where we pass in our actual RAUI widget tree, specifying the
        // whole RAUI UI. The widget! macro is used to create widget nodes that
        // are used to represent widget trees.
        //
        // Here we create a widget node with `my_widget` as the only component in
        // it.
        let tree = widget! {
            (my_first_widget)
        };

        // ...
    }
}
```

Now when you `cargo run` you should see your very first widget saying "Hello World!".

![hello world](./img/hello_world.png)
