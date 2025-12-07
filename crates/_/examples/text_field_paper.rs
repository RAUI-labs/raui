use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    ManagedBox, make_widget, pre_hooks,
    view_model::ViewModel,
    widget::{
        component::{
            containers::size_box::{SizeBoxProps, size_box},
            interactive::{
                button::ButtonNotifyProps,
                input_field::{TextInput, TextInputProps},
                navigation::{NavItemActive, use_nav_container_active},
            },
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::{size::SizeBoxSizeValue, text::TextBoxFont},
        utils::Rect,
    },
};
use raui_material::{
    component::{
        containers::paper::paper,
        interactive::text_field_paper::{TextFieldPaperProps, text_field_paper},
    },
    theme::{ThemeColor, ThemeProps, ThemedTextMaterial, ThemedWidgetProps, new_dark_theme},
};

const TEXT_INPUT: &str = "text-input";

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

fn use_app(ctx: &mut WidgetContext) {
    ctx.life_cycle.mount(|mut ctx| {
        // Initialize the view model for the text input field.
        let mut view_model = ViewModel::produce(|_| ManagedBox::new("Hello!".to_owned()));
        view_model
            .properties
            .bindings(TEXT_INPUT)
            .unwrap()
            .bind(ctx.id.to_owned());
        ctx.view_models.widget_register(TEXT_INPUT, view_model);
    });
}

#[pre_hooks(use_nav_container_active, use_app)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        mut view_models,
        ..
    } = ctx;

    // Turn the view model into a lazy TextInput for the text field props.
    let text = view_models.widget_view_model_mut(TEXT_INPUT).and_then(|v| {
        v.write::<ManagedBox<String>>()
            .map(|mut v| TextInput::new(v.lazy()))
    });

    make_widget!(paper)
        .with_shared_props(new_theme())
        .listed_slot(
            make_widget!(size_box)
                .with_props(SizeBoxProps {
                    width: SizeBoxSizeValue::Fill,
                    height: SizeBoxSizeValue::Exact(50.0),
                    margin: 20.0.into(),
                    ..Default::default()
                })
                .named_slot(
                    "content",
                    make_widget!(text_field_paper)
                        .key("name")
                        .with_props(TextFieldPaperProps {
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
                        })
                        // Make input text editable.
                        .with_props(NavItemActive)
                        // Notify this widget about changes made by input text.
                        .with_props(ButtonNotifyProps(id.to_owned().into()))
                        // Pass the lazy TextInput to the text field paper to edit.
                        .with_props(TextInputProps {
                            text,
                            ..Default::default()
                        }),
                ),
        )
        .into()
}

fn main() {
    DeclarativeApp::simple("Text Field Paper", make_widget!(app));
}
