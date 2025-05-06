use raui_app::app::{App, AppConfig, declarative::DeclarativeApp};
use raui_core::{
    Managed, Scalar, make_widget, pre_hooks,
    view_model::{ViewModel, ViewModelValue},
    widget::{
        component::{
            containers::vertical_box::vertical_box,
            interactive::{
                button::{
                    ButtonNotifyMessage, ButtonNotifyProps, ButtonProps, use_button_notified_state,
                },
                input_field::{
                    TextInputControlNotifyMessage, TextInputControlNotifyProps, TextInputMode,
                    TextInputNotifyMessage, TextInputNotifyProps, TextInputProps, TextInputState,
                    input_field, input_text_with_cursor, use_text_input_notified_state,
                },
                navigation::{NavItemActive, use_nav_container_active},
            },
            text_box::{TextBoxProps, text_box},
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::text::{TextBoxFont, TextBoxSizeValue},
        utils::Color,
    },
};

const DATA: &str = "data";
const TEXT_INPUT: &str = "text-input";
const NUMBER_INPUT: &str = "number-input";
const INTEGER_INPUT: &str = "integer-input";
const UNSIGNED_INTEGER_INPUT: &str = "unsigned-integer-input";
const FILTER_INPUT: &str = "filter-input";

struct AppData {
    text_input: Managed<ViewModelValue<String>>,
    number_input: Managed<ViewModelValue<String>>,
    integer_input: Managed<ViewModelValue<String>>,
    unsigned_integer_input: Managed<ViewModelValue<String>>,
    filter_input: Managed<ViewModelValue<String>>,
}

fn use_app(ctx: &mut WidgetContext) {
    ctx.life_cycle.mount(|mut ctx| {
        ctx.view_models
            .bindings(DATA, TEXT_INPUT)
            .unwrap()
            .bind(ctx.id.to_owned());
        ctx.view_models
            .bindings(DATA, NUMBER_INPUT)
            .unwrap()
            .bind(ctx.id.to_owned());
        ctx.view_models
            .bindings(DATA, INTEGER_INPUT)
            .unwrap()
            .bind(ctx.id.to_owned());
        ctx.view_models
            .bindings(DATA, UNSIGNED_INTEGER_INPUT)
            .unwrap()
            .bind(ctx.id.to_owned());
        ctx.view_models
            .bindings(DATA, FILTER_INPUT)
            .unwrap()
            .bind(ctx.id.to_owned());
    });
}

// we mark root widget as navigable container to let user focus and type in text inputs.
#[pre_hooks(use_nav_container_active, use_app)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    let mut app_data = ctx
        .view_models
        .view_model_mut(DATA)
        .unwrap()
        .write::<AppData>()
        .unwrap();

    // put inputs with all different types modes.
    make_widget!(vertical_box)
        .listed_slot(
            make_widget!(input)
                .with_props(TextInputMode::Text)
                .with_props(TextInputProps {
                    allow_new_line: false,
                    text: Some(app_data.text_input.lazy().into()),
                }),
        )
        .listed_slot(
            make_widget!(input)
                .with_props(TextInputMode::Number)
                .with_props(TextInputProps {
                    allow_new_line: false,
                    text: Some(app_data.number_input.lazy().into()),
                }),
        )
        .listed_slot(
            make_widget!(input)
                .with_props(TextInputMode::Integer)
                .with_props(TextInputProps {
                    allow_new_line: false,
                    text: Some(app_data.integer_input.lazy().into()),
                }),
        )
        .listed_slot(
            make_widget!(input)
                .with_props(TextInputMode::UnsignedInteger)
                .with_props(TextInputProps {
                    allow_new_line: false,
                    text: Some(app_data.unsigned_integer_input.lazy().into()),
                }),
        )
        .listed_slot(
            make_widget!(input)
                .with_props(TextInputMode::Filter(|_, character| {
                    character.is_uppercase()
                }))
                .with_props(TextInputProps {
                    allow_new_line: false,
                    text: Some(app_data.filter_input.lazy().into()),
                }),
        )
        .into()
}

fn use_input(ctx: &mut WidgetContext) {
    ctx.life_cycle.change(|ctx| {
        for msg in ctx.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<TextInputNotifyMessage>() {
                println!("* Text input: {:#?}", msg);
            } else if let Some(msg) = msg.as_any().downcast_ref::<TextInputControlNotifyMessage>() {
                println!("* Text input control: {:#?}", msg);
            } else if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                println!("* Button: {:#?}", msg);
            }
        }
    });
}

// this component will receive and store button and input text state changes.
#[pre_hooks(use_button_notified_state, use_text_input_notified_state, use_input)]
fn input(mut ctx: WidgetContext) -> WidgetNode {
    let ButtonProps {
        selected, trigger, ..
    } = ctx.state.read_cloned_or_default();

    let TextInputState {
        cursor_position,
        focused,
    } = ctx.state.read_cloned_or_default();

    let TextInputProps {
        allow_new_line,
        text,
    } = ctx.props.read_cloned_or_default();

    let mode = ctx.props.read_cloned_or_default::<TextInputMode>();

    let value = text
        .as_ref()
        .and_then(|text| mode.process(&text.get()))
        .unwrap_or_default();

    // input field is an evolution of input text, what changes is input field can be focused
    // because it is input text plus button.
    make_widget!(input_field)
        // as usually we enable this navigation item.
        .with_props(NavItemActive)
        // pass text input mode to the input field (by default Text mode is used).
        .with_props(mode)
        // setup text input.
        .with_props(TextInputProps {
            allow_new_line,
            text,
        })
        // notify this component about input text state change.
        .with_props(TextInputNotifyProps(ctx.id.to_owned().into()))
        // notify this component about input control characters it receives.
        // useful for reacting to Tab key for example.
        .with_props(TextInputControlNotifyProps(ctx.id.to_owned().into()))
        // notify this component about button state change.
        .with_props(ButtonNotifyProps(ctx.id.to_owned().into()))
        .named_slot(
            "content",
            // input field and input text components doesn't assume any content widget for you so
            // that's why we create custom input component to make it work and look exactly as we
            // want - here we just put a text box.
            make_widget!(text_box).with_props(TextBoxProps {
                text: if focused {
                    input_text_with_cursor(&value, cursor_position, '|')
                } else if value.is_empty() {
                    match mode {
                        TextInputMode::Text => "> Type text...".to_owned(),
                        TextInputMode::Number => "> Type number...".to_owned(),
                        TextInputMode::Integer => "> Type integer...".to_owned(),
                        TextInputMode::UnsignedInteger => "> Type unsigned integer...".to_owned(),
                        TextInputMode::Filter(_) => "> Type uppercase text...".to_owned(),
                    }
                } else {
                    value
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
    let app = DeclarativeApp::default()
        .tree(make_widget!(app))
        .view_model(
            DATA,
            ViewModel::produce(|properties| AppData {
                text_input: Managed::new(ViewModelValue::new(
                    Default::default(),
                    properties.notifier(TEXT_INPUT),
                )),
                number_input: Managed::new(ViewModelValue::new(
                    Default::default(),
                    properties.notifier(NUMBER_INPUT),
                )),
                integer_input: Managed::new(ViewModelValue::new(
                    Default::default(),
                    properties.notifier(INTEGER_INPUT),
                )),
                unsigned_integer_input: Managed::new(ViewModelValue::new(
                    Default::default(),
                    properties.notifier(UNSIGNED_INTEGER_INPUT),
                )),
                filter_input: Managed::new(ViewModelValue::new(
                    Default::default(),
                    properties.notifier(FILTER_INPUT),
                )),
            }),
        );

    App::new(AppConfig::default().title("Input Field")).run(app);
}
