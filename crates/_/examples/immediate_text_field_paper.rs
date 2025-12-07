use raui_app::app::immediate::ImmediateApp;
use raui_core::widget::{
    component::{containers::size_box::SizeBoxProps, interactive::navigation::NavItemActive},
    unit::{size::SizeBoxSizeValue, text::TextBoxFont},
    utils::Rect,
};
use raui_immediate::{ImSharedProps, apply, use_state};
use raui_immediate_widgets::{
    core::containers::size_box,
    material::{containers::nav_paper, interactive::text_field_paper},
};
use raui_material::{
    component::interactive::text_field_paper::TextFieldPaperProps,
    theme::{ThemeColor, ThemeProps, ThemedTextMaterial, ThemedWidgetProps, new_dark_theme},
};

// Create a new theme with a custom text variant for input fields.
fn new_theme() -> ThemeProps {
    let mut theme = new_dark_theme();
    theme.text_variants.insert(
        "input".to_owned(),
        ThemedTextMaterial {
            font: TextBoxFont {
                name: "./demos/hello-world/resources/verdana.ttf".to_owned(),
                size: 24.0,
            },
            ..Default::default()
        },
    );
    theme
}

fn main() {
    ImmediateApp::simple("Immediate mode Text Field Paper", |_| {
        // Apply the custom theme for all UI widgets.
        apply(ImSharedProps(new_theme()), || {
            // Make navigable paper container for the text field.
            // Navigable containers are required to make interactive widgets work.
            nav_paper((), || {
                let props = SizeBoxProps {
                    width: SizeBoxSizeValue::Fill,
                    height: SizeBoxSizeValue::Exact(50.0),
                    margin: 20.0.into(),
                    ..Default::default()
                };

                size_box(props, || {
                    let props = (
                        TextFieldPaperProps {
                            hint: "> Type some text...".to_owned(),
                            paper_theme: ThemedWidgetProps {
                                color: ThemeColor::Primary,
                                ..Default::default()
                            },
                            padding: Rect {
                                left: 10.0,
                                right: 10.0,
                                top: 6.0,
                                bottom: 6.0,
                            },
                            variant: "input".to_owned(),
                            ..Default::default()
                        },
                        NavItemActive,
                    );

                    // Make state holding the text input value.
                    let text = use_state(|| "Hello!".to_owned());

                    // Make the text field paper with the text input state and
                    // override existing value on change.
                    let value = text_field_paper(&*text.read().unwrap(), props).0;
                    if let Some(value) = value {
                        *text.write().unwrap() = value;
                    }
                });
            });
        });
    });
}
