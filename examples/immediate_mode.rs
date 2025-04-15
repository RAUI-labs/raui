// Example of immediate mode UI on top of RAUI.
// It's goal is to bring more ergonomics to RAUI by hiding
// declarative interface under simple nested function calls.
// As with retained mode, immediate mode UI can be mixed with
// declarative mode and retained mode widgets.

#[allow(unused_imports)]
use raui_app::prelude::*;

const FONT: &str = "./demos/hello-world/resources/verdana.ttf";

mod gui {
    use raui_core::{Scalar, widget::component::interactive::input_field::input_text_with_cursor};
    use raui_immediate::*;
    use raui_immediate_widgets::prelude::*;

    // app function widget, we pass application state there.
    pub fn app(value: &mut usize) {
        let props = WrapBoxProps {
            margin: 20.0.into(),
            fill: true,
        };

        wrap_box(props, || {
            // we can use any "immedietified" RAUI widget we want.
            // we can pass Props to parameterize RAUI widget in first param.
            // BTW. we should make sure to use any `nav_*` container widget
            // somewhere in the app root to make app interactive.
            nav_vertical_box((), || {
                let layout = FlexBoxItemLayout {
                    basis: Some(48.0),
                    grow: 0.0,
                    shrink: 0.0,
                    ..Default::default()
                };

                // we can also apply props on all produced widgets in the scope.
                apply_props(layout, || {
                    counter(value);

                    let props = HorizontalBoxProps {
                        separation: 50.0,
                        ..Default::default()
                    };

                    horizontal_box(props, || {
                        // we can react to button-like behavior by reading what
                        // button-like widgets return of their tracked state.
                        if text_button("Increment").trigger_start() {
                            *value = value.saturating_add(1);
                        }

                        if text_button("Decrement").trigger_start() {
                            *value = value.saturating_sub(1);
                        }
                    });
                });
            });
        });
    }

    fn text_button(text: &str) -> ImmediateButton {
        // buttons use `use_state` hook under the hood to track
        // declarative mode button state, that's copy of being
        // returned from button function and passed into its
        // group closure for children widgets to use.
        // BTW. don't forget to apply `NavItemActive` props on
        // button if you want to have it enabled for navigation.
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

    fn counter(value: &mut usize) {
        // counter widget is a text box wrapped in an input field.
        // it works like combination of button (can be focused by
        // selection/navigation) and text field (collects keyboard
        // text characters when focused).
        let props = (NavItemActive, TextInputMode::UnsignedInteger);

        let (result, ..) = input_field(value, props, |text, state, button| {
            text_box(TextBoxProps {
                text: if state.focused {
                    input_text_with_cursor(text, state.cursor_position, '|')
                } else if text.is_empty() {
                    "...".to_owned()
                } else {
                    text.to_owned()
                },
                font: TextBoxFont {
                    name: crate::FONT.to_owned(),
                    size: 32.0,
                },
                color: Color {
                    r: Scalar::from(button.state.trigger),
                    g: Scalar::from(button.state.selected),
                    b: Scalar::from(state.focused),
                    a: 1.0,
                },
                ..Default::default()
            });
        });

        if let Some(result) = result {
            *value = result;
        }
    }
}

fn main() {
    // some applciation state.
    let mut counter = 0usize;

    ImmediateApp::simple("Immediate mode UI", move || {
        gui::app(&mut counter);
    });
}
