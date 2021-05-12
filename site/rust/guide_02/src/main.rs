use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;

fn main() {
    // Create our quick start builder
    RauiQuickStartBuilder::default()
        // Set our window title
        .window_title("RAUI Guide".into())
        // Set the RAUI widget tree for our app
        .widget_tree(widget! {
            // UPDATE HERE
            // 
            // Our app is the only widget we insert here. All other widgets
            // will go inside of our app
            (app)
        })
        // Build the app
        .build()
        .expect("Error building quick start")
        // And run it! 🚀
        .run()
        .expect("Error running RAUI app");
}


/// NEW
/// 
/// We create our own widget by making a function that takes a `WidgetContext`
/// and that returns `WidgetNode`.
pub fn app(_ctx: WidgetContext) -> WidgetNode {
    // Our _ctx variable starts with an underscore so rust doesn't complain 
    // that it is unused. We will be using the context later in the guide.

    // We may do any amount of processing in the body of the function.

    // For now we will simply be creating a text box properties struct that we
    // will use to configure the `text_box` component.
    let text_box_props = Props::new(TextBoxProps {
        text: "Hello world!".to_owned(),
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        font: TextBoxFont {
            // We specify the path to our font
            name: "resources/verdana.ttf".to_owned(),
            size: 60.0,
        },
        // Use the defaults for the rest of the text box settings
        ..Default::default()
    });

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
