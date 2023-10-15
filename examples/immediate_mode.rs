// Example of immediate mode UI on top of RAUI.
// It's goal is to bring more ergonomics to RAUI by hiding
// declarative interface under simple nested function calls.
// As with retained mode, immediate mode UI can be mixed with
// declarative mode and retained mode widgets.

use raui_immediate::{make_widgets, ImmediateContext};
use raui_quick_start::RauiQuickStartBuilder;

const FONT: &str = "./demos/hello-world/resources/verdana.ttf";

mod gui {
    use raui_core::Scalar;
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
                            *value += 1;
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
        let props = (
            NavItemActive,
            TextInputMode::UnsignedInteger,
            // if we want to override `input_field` text value,
            // do it with String props.
            value.to_string(),
        );

        let result = input_field(props, |state, button| {
            text_box(TextBoxProps {
                text: if state.text.trim().is_empty() {
                    "...".to_owned()
                } else if state.focused {
                    if state.cursor_position < state.text.len() {
                        format!(
                            "{}|{}",
                            &state.text[..state.cursor_position],
                            &state.text[state.cursor_position..]
                        )
                    } else {
                        format!("{}|", state.text)
                    }
                } else {
                    state.text.to_owned()
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

        if let Ok(result) = result.0.text.parse() {
            *value = result;
        }
    }
}

fn main() {
    use raui_core::prelude::*;

    // immediate mode context is backend of `use_state` hook
    // that tracks persistent state in function calls.
    let context = ImmediateContext::default();

    // some applciation state.
    let mut counter = 0usize;

    RauiQuickStartBuilder::default()
        .window_title("Immediate mode UI".to_owned())
        .build()
        .unwrap()
        .on_update(move |_, ui| {
            // resets immediate mode UI builder to ensure we start
            // building current frame UI from scratch.
            raui_immediate::reset();

            // `make_widgets` function is a shorthand for:
            // - activate immediate context.
            // - begin widgets group.
            // - execute closure.
            // - end widgets group.
            // - deactivate imemdiate context.
            let widgets = make_widgets(&context, || gui::app(&mut counter));

            // once that's done, we get list of RAUI widgets produced
            // and we can embed them in some RAUI container, preferably
            // content box to simulate layers on the screen.
            ui.application.apply(
                make_widget!(content_box)
                    .listed_slots(widgets.into_iter())
                    .into(),
            );
            true
        })
        .run()
        .unwrap();
}
