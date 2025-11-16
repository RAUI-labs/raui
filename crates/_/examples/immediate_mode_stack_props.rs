use raui_app::app::immediate::ImmediateApp;
use raui_core::widget::{
    component::text_box::TextBoxProps,
    unit::{
        flex::FlexBoxItemLayout,
        text::{TextBoxFont, TextBoxHorizontalAlign},
    },
    utils::Color,
};
use raui_immediate::{ImProps, ImStackProps, apply, use_stack_props};
use raui_immediate_widgets::core::{containers::nav_vertical_box, text_box};

pub fn app() {
    let props = TextBoxProps {
        font: TextBoxFont {
            name: "./demos/hello-world/resources/verdana.ttf".to_owned(),
            size: 96.0,
        },
        color: Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        ..Default::default()
    };

    // We can create cascaded styling with stack props.
    // The difference between stack props and applied props
    // is that applied props are applied directly do its
    // children nodes, while stack props are stacked so any
    // widget in hierarchy can access the top of the props
    // stack - we can easily share style down the hierarchy!
    apply(ImStackProps::new(props), || {
        nav_vertical_box((), || {
            let layout = FlexBoxItemLayout {
                basis: Some(100.0),
                grow: 0.0,
                shrink: 0.0,
                margin: 32.0.into(),
                ..Default::default()
            };

            // These props apply only to label widgets.
            apply(ImProps(layout), || {
                label("Hey!");
                label("Hi!");

                let props = TextBoxProps {
                    font: TextBoxFont {
                        name: "./demos/in-game/resources/fonts/MiKrollFantasy.ttf".to_owned(),
                        size: 100.0,
                    },
                    color: Color {
                        r: 0.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    },
                    horizontal_align: TextBoxHorizontalAlign::Center,
                    ..Default::default()
                };

                // By pushing new props on stack props we override
                // what's gonna be used in all chidren in hierarchy.
                apply(ImStackProps::new(props), || {
                    label("Hello!");
                    label("Ohayo?");
                });
            });
        });
    });
}

pub fn label(text: impl ToString) {
    // Accessing props from the stack to achieve cascading styles.
    let mut props = use_stack_props::<TextBoxProps>().unwrap_or_default();
    props.text = text.to_string();

    text_box(props);
}

fn main() {
    ImmediateApp::simple("Immediate mode UI - Stack props", |_| app());
}
