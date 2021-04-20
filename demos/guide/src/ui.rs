// Include the RAUI prelude
use raui::prelude::*;

/// We create our own widget by making a function that takes a `WidgetContext`
/// and that returns `WidgetNode`.
pub fn my_first_widget(_ctx: WidgetContext) -> WidgetNode {
    // We may do any amount of processing in the body of the function.

    // For now we will simply be creating a text box properties struct that we
    // will use to configure the `text_box` component.
    let text_box_props = TextBoxProps {
        text: "Hello world!".to_owned(),
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        font: TextBoxFont {
            name: "verdana".to_owned(),
            size: 32.0,
        },
        ..Default::default()
    };

    // And at the end of the function we return a `WidgetNode` which we can
    // conveniently create with the `widget!` macro.
    widget! {
        (text_box: {text_box_props})
        // ^          ^
        // |          |
        // |          ---- After the name of the widget component we pass in
        // |               the component properties.
        // |
        // --- This is the name of the `text_box` widget component, which is a
        //     part of the RAUI prelude
    }
}
