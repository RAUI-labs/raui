// Make sure you have seen `immediate_mode` code example first, because this is a continuation of that.

use raui_app::app::immediate::ImmediateApp;
use raui_core::widget::{
    component::{
        containers::wrap_box::WrapBoxProps, image_box::ImageBoxProps,
        interactive::navigation::NavItemActive, text_box::TextBoxProps,
    },
    unit::text::TextBoxFont,
    utils::Color,
};
use raui_immediate::{ImmediateOnMount, ImmediateOnUnmount, use_effects, use_state};
use raui_immediate_widgets::core::{
    containers::{content_box, nav_vertical_box, wrap_box},
    image_box,
    interactive::{ImmediateButton, button},
    text_box,
};

const FONT: &str = "./demos/hello-world/resources/verdana.ttf";

pub fn app() {
    let props = WrapBoxProps {
        margin: 20.0.into(),
        ..Default::default()
    };

    wrap_box(props, || {
        nav_vertical_box((), || {
            // `use_state` allows to keep persistent state across
            // multiple frames, as long as order of calls and types
            // match between frames.
            let flag = use_state(|| false);
            let mut flag = flag.write().unwrap();

            let counter = use_state(|| 0usize);
            let counter_mount = counter.clone();

            if text_button("Toggle").trigger_start() {
                *flag = !*flag;
            }

            if *flag {
                // effects are passed as props, these are callbacks
                // that get executed whenever RAUI widget gets mounted,
                // unmounted or changed.
                // There is also `ImmediateHooks` props that allow to
                // apply RAUI hooks to rendered widget, useful for example
                // to render effects widget with any custom behavior.
                let effects = (
                    ImmediateOnMount::new(move || {
                        println!("Mounted!");
                        *counter_mount.write().unwrap() += 1;
                    }),
                    ImmediateOnUnmount::new(|| {
                        println!("Unmounted!");
                    }),
                );

                use_effects(effects, || {
                    label(format!("Mounted {} times!", *counter.read().unwrap()));
                });
            }
        });
    });
}

fn label(text: impl ToString) {
    text_box(TextBoxProps {
        text: text.to_string(),
        font: TextBoxFont {
            name: crate::FONT.to_owned(),
            size: 32.0,
        },
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        ..Default::default()
    });
}

fn text_button(text: &str) -> ImmediateButton {
    button(NavItemActive, |state| {
        content_box((), || {
            image_box(ImageBoxProps::colored(Color {
                r: if state.state.selected { 1.0 } else { 0.75 },
                g: if state.state.trigger { 1.0 } else { 0.75 },
                b: if state.state.context { 1.0 } else { 0.75 },
                a: 1.0,
            }));

            text_box(TextBoxProps {
                text: text.to_string(),
                font: TextBoxFont {
                    name: crate::FONT.to_owned(),
                    size: 32.0,
                },
                color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                ..Default::default()
            });
        });
    })
}

fn main() {
    ImmediateApp::simple("Immediate mode UI - States and Effects", move || {
        app();
    });
}
