use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

// we mark root widget as navigable container to let user focus and type in text inputs.
#[pre_hooks(use_nav_container_active)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    make_widget!(vertical_box)
        // put inputs with all different types modes.
        .listed_slot(make_widget!(input).with_props(TextInputMode::Text))
        .listed_slot(make_widget!(input).with_props(TextInputMode::Number))
        .listed_slot(make_widget!(input).with_props(TextInputMode::Integer))
        .listed_slot(make_widget!(input).with_props(TextInputMode::UnsignedInteger))
        .listed_slot(
            make_widget!(input).with_props(TextInputMode::Filter(|_, character| {
                character.is_uppercase()
            })),
        )
        .into()
}

// this component will receive and store button and input text state changes.
#[pre_hooks(use_button_notified_state, use_text_input_notified_state)]
fn input(mut ctx: WidgetContext) -> WidgetNode {
    let ButtonProps {
        selected, trigger, ..
    } = ctx.state.read_cloned_or_default();

    let TextInputProps {
        text,
        cursor_position,
        focused,
        ..
    } = ctx.state.read_cloned_or_default();

    let mode = ctx.props.read_cloned_or_default::<TextInputMode>();

    // input field is an evolution of input text, what changes is input field can be focused
    // because it is input text plus button.
    make_widget!(input_field)
        // as usually we enable this navigation item.
        .with_props(NavItemActive)
        // pass text input mode to the input field (by default Text mode is used).
        .with_props(mode)
        // notify this component about input text state change.
        .with_props(TextInputNotifyProps(ctx.id.to_owned().into()))
        // notify this component about button state change.
        .with_props(ButtonNotifyProps(ctx.id.to_owned().into()))
        .named_slot(
            "content",
            // input field and input text components doesn't assume any content widget for you so
            // that's why we create custom input component to make it work and look exactly as we
            // want - here we just put a text box.
            make_widget!(text_box).with_props(TextBoxProps {
                text: if text.is_empty() {
                    match mode {
                        TextInputMode::Text => "> Type text...".to_owned(),
                        TextInputMode::Number => "> Type number...".to_owned(),
                        TextInputMode::Integer => "> Type integer...".to_owned(),
                        TextInputMode::UnsignedInteger => "> Type unsigned integer...".to_owned(),
                        TextInputMode::Filter(_) => "> Type uppercase text...".to_owned(),
                    }
                } else if focused {
                    input_text_with_cursor(&text, cursor_position, '|')
                } else {
                    text
                },
                width: TextBoxSizeValue::Fill,
                height: TextBoxSizeValue::Exact(48.0),
                font: TextBoxFont {
                    name: "./demos/hello-world/resources/verdana.ttf".to_owned(),
                    size: 32.0,
                },
                color: Color {
                    r: Scalar::from(trigger),
                    g: Scalar::from(selected),
                    b: Scalar::from(focused),
                    a: 1.0,
                },
                ..Default::default()
            }),
        )
        .into()
}

fn main() {
    DeclarativeApp::simple("Input Field", make_widget!(app));
}
