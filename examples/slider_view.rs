use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

const DATA: &str = "data";
const FLOAT_INPUT: &str = "float-input";
const INTEGER_INPUT: &str = "integer-input";
const UNSIGNED_INTEGER_INPUT: &str = "unsigned-integer-input";

struct AppData {
    float_input: Managed<ViewModelValue<f32>>,
    integer_input: Managed<ViewModelValue<i32>>,
    unsigned_integer_input: Managed<ViewModelValue<u8>>,
}

fn use_app(ctx: &mut WidgetContext) {
    ctx.life_cycle.mount(|mut ctx| {
        ctx.view_models
            .bindings(DATA, FLOAT_INPUT)
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
    });
}

#[pre_hooks(use_nav_container_active, use_app)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    let mut app_data = ctx
        .view_models
        .view_model_mut(DATA)
        .unwrap()
        .write::<AppData>()
        .unwrap();

    make_widget!(horizontal_box)
        .listed_slot(
            make_widget!(input)
                .with_props(FlexBoxItemLayout {
                    margin: 50.0.into(),
                    ..Default::default()
                })
                .with_props(SliderViewProps {
                    input: Some(app_data.float_input.lazy().into()),
                    from: -10.0,
                    to: 10.0,
                    direction: SliderViewDirection::BottomToTop,
                }),
        )
        .listed_slot(
            make_widget!(vertical_box)
                .with_props(VerticalBoxProps {
                    override_slots_layout: Some(FlexBoxItemLayout {
                        margin: 50.0.into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .listed_slot(make_widget!(input).with_props(SliderViewProps {
                    input: Some(app_data.integer_input.lazy().into()),
                    from: -2.0,
                    to: 2.0,
                    ..Default::default()
                }))
                .listed_slot(make_widget!(input).with_props(SliderViewProps {
                    input: Some(app_data.unsigned_integer_input.lazy().into()),
                    from: -3.0,
                    to: 7.0,
                    direction: SliderViewDirection::RightToLeft,
                })),
        )
        .into()
}

fn input(ctx: WidgetContext) -> WidgetNode {
    let props = ctx.props.read_cloned_or_default::<SliderViewProps>();
    let percentage = props.get_percentage();
    let value = props.get_value();
    let anchors = match props.direction {
        SliderViewDirection::LeftToRight => Rect {
            left: 0.0,
            right: percentage,
            top: 0.0,
            bottom: 1.0,
        },
        SliderViewDirection::RightToLeft => Rect {
            left: 1.0 - percentage,
            right: 1.0,
            top: 0.0,
            bottom: 1.0,
        },
        SliderViewDirection::TopToBottom => Rect {
            left: 0.0,
            right: 1.0,
            top: 0.0,
            bottom: percentage,
        },
        SliderViewDirection::BottomToTop => Rect {
            left: 0.0,
            right: 1.0,
            top: 1.0 - percentage,
            bottom: 1.0,
        },
    };

    make_widget!(slider_view)
        .with_props(NavItemActive)
        .with_props(props)
        .named_slot(
            "content",
            make_widget!(content_box)
                .listed_slot(
                    make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                        r: 0.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    })),
                )
                .listed_slot(
                    make_widget!(image_box)
                        .with_props(ContentBoxItemLayout {
                            anchors,
                            ..Default::default()
                        })
                        .with_props(ImageBoxProps::colored(Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        })),
                )
                .listed_slot(make_widget!(text_box).with_props(TextBoxProps {
                    text: value.to_string(),
                    horizontal_align: TextBoxHorizontalAlign::Center,
                    vertical_align: TextBoxVerticalAlign::Middle,
                    font: TextBoxFont {
                        name: "./demos/hello-world/resources/verdana.ttf".to_owned(),
                        size: 64.0,
                    },
                    color: Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    },
                    ..Default::default()
                })),
        )
        .into()
}

fn main() {
    let app = DeclarativeApp::default()
        .tree(make_widget!(app))
        .view_model(
            DATA,
            ViewModel::produce(|properties| AppData {
                float_input: Managed::new(ViewModelValue::new(
                    Default::default(),
                    properties.notifier(FLOAT_INPUT),
                )),
                integer_input: Managed::new(ViewModelValue::new(
                    Default::default(),
                    properties.notifier(INTEGER_INPUT),
                )),
                unsigned_integer_input: Managed::new(ViewModelValue::new(
                    Default::default(),
                    properties.notifier(UNSIGNED_INTEGER_INPUT),
                )),
            }),
        );

    App::new(AppConfig::default().title("Slider View")).run(app);
}
