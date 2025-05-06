use raui::{
    app::app::declarative::DeclarativeApp,
    core::{
        make_widget,
        widget::{
            component::text_box::{TextBoxProps, text_box},
            context::WidgetContext,
            node::WidgetNode,
            unit::text::TextBoxFont,
            utils::Color,
        },
    },
};

fn main() {
    DeclarativeApp::simple("RAUI Guide", make_widget!(app));
}

/// We create our own widget by making a function that takes a `WidgetContext`
/// and that returns `WidgetNode`.
pub fn app(_ctx: WidgetContext) -> WidgetNode {
    // Our _ctx variable starts with an underscore so rust doesn't complain
    // that it is unused. We will be using the context later in the guide.

    // We may do any amount of processing in the body of the function.
    // For now we will simply be creating a text box properties struct that we
    // will use to configure the `text_box` component.
    make_widget!(text_box)
        .with_props(TextBoxProps {
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
        })
        .into()
}
