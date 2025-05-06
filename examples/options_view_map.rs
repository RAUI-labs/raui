// Make sure you have seen `options_view` code example first, because this is an evolution of that.

use raui_app::app::{App, AppConfig, declarative::DeclarativeApp};
use raui_core::{
    Managed, Scalar, make_widget, pre_hooks,
    view_model::{ViewModel, ViewModelValue},
    widget::{
        WidgetRef,
        component::{
            containers::{
                anchor_box::PivotBoxProps, content_box::content_box, portal_box::PortalsContainer,
                size_box::SizeBoxProps,
            },
            image_box::{ImageBoxProps, image_box},
            interactive::{
                button::ButtonProps,
                navigation::{NavItemActive, use_nav_container_active},
                options_view::{OptionsViewMode, OptionsViewProps, options_view},
            },
            text_box::{TextBoxProps, text_box},
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::{
            content::ContentBoxItemLayout,
            size::SizeBoxSizeValue,
            text::{TextBoxFont, TextBoxHorizontalAlign, TextBoxVerticalAlign},
        },
        utils::{Color, Rect},
    },
};

const DATA: &str = "data";
const INDEX: &str = "index";

struct AppData {
    index: Managed<ViewModelValue<usize>>,
}

fn use_app(ctx: &mut WidgetContext) {
    ctx.life_cycle.mount(|mut ctx| {
        ctx.view_models
            .bindings(DATA, INDEX)
            .unwrap()
            .bind(ctx.id.to_owned());
    });
}

#[pre_hooks(use_nav_container_active, use_app)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    let idref = WidgetRef::default();
    let mut app_data = ctx
        .view_models
        .view_model_mut(DATA)
        .unwrap()
        .write::<AppData>()
        .unwrap();

    make_widget!(content_box)
        .idref(idref.clone())
        .with_shared_props(PortalsContainer(idref))
        .listed_slot(
            make_widget!(options_view)
                .with_props(ContentBoxItemLayout {
                    anchors: 0.1.into(),
                    margin: [-200.0, -40.0].into(),
                    ..Default::default()
                })
                .with_props(OptionsViewProps {
                    input: Some(app_data.index.lazy().into()),
                })
                .with_props(NavItemActive)
                .with_props(PivotBoxProps {
                    pivot: [0.0, 1.0].into(),
                    align: 0.0.into(),
                })
                .with_props(SizeBoxProps {
                    width: SizeBoxSizeValue::Exact(500.0),
                    height: SizeBoxSizeValue::Exact(500.0),
                    ..Default::default()
                })
                .named_slot(
                    "content",
                    make_widget!(content_box)
                        .with_props(ContentBoxItemLayout {
                            keep_in_bounds: true.into(),
                            ..Default::default()
                        })
                        .listed_slot(
                            make_widget!(image_box)
                                .with_props(ImageBoxProps::image("./examples/resources/map.png")),
                        ),
                )
                .listed_slot(
                    make_widget!(option)
                        .with_props("Vidence".to_owned())
                        .with_props(NavItemActive)
                        .with_props(marker_content_layout(0.1, 0.3)),
                )
                .listed_slot(
                    make_widget!(option)
                        .with_props("Yrale".to_owned())
                        .with_props(NavItemActive)
                        .with_props(marker_content_layout(0.6, 0.2)),
                )
                .listed_slot(
                    make_widget!(option)
                        .with_props("Qock".to_owned())
                        .with_props(NavItemActive)
                        .with_props(marker_content_layout(0.9, 0.6)),
                )
                .listed_slot(
                    make_widget!(option)
                        .with_props("Eryphia".to_owned())
                        .with_props(NavItemActive)
                        .with_props(marker_content_layout(0.3, 0.7)),
                ),
        )
        .into()
}

fn marker_content_layout(x: Scalar, y: Scalar) -> ContentBoxItemLayout {
    ContentBoxItemLayout {
        anchors: Rect {
            left: x,
            right: x,
            top: y,
            bottom: y,
        },
        margin: Rect {
            left: -50.0,
            right: -50.0,
            top: -10.0,
            bottom: -10.0,
        },
        align: 0.5.into(),
        ..Default::default()
    }
}

fn option(ctx: WidgetContext) -> WidgetNode {
    match ctx.props.read_cloned_or_default::<OptionsViewMode>() {
        OptionsViewMode::Selected => option_selected(ctx),
        OptionsViewMode::Option => option_marker(ctx),
    }
}

fn option_selected(ctx: WidgetContext) -> WidgetNode {
    let ButtonProps {
        selected, trigger, ..
    } = ctx.props.read_cloned_or_default();
    let color = if trigger {
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    } else if selected {
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    } else {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    };
    let text = ctx.props.read_cloned_or_default::<String>();

    make_widget!(content_box)
        .listed_slot(make_widget!(image_box).with_props(ImageBoxProps::colored(color)))
        .listed_slot(make_widget!(text_box).with_props(TextBoxProps {
            text,
            font: TextBoxFont {
                name: "./demos/hello-world/resources/verdana.ttf".to_owned(),
                size: 32.0,
            },
            horizontal_align: TextBoxHorizontalAlign::Center,
            vertical_align: TextBoxVerticalAlign::Middle,
            color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            ..Default::default()
        }))
        .into()
}

fn option_marker(ctx: WidgetContext) -> WidgetNode {
    let ButtonProps {
        selected, trigger, ..
    } = ctx.props.read_cloned_or_default();
    let color = if trigger {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    } else if selected {
        Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        }
    } else {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    };
    let text = ctx.props.read_cloned_or_default::<String>();

    make_widget!(text_box)
        .with_props(TextBoxProps {
            text,
            font: TextBoxFont {
                name: "./demos/hello-world/resources/verdana.ttf".to_owned(),
                size: 20.0,
            },
            horizontal_align: TextBoxHorizontalAlign::Center,
            vertical_align: TextBoxVerticalAlign::Middle,
            color,
            ..Default::default()
        })
        .into()
}

fn main() {
    let app = DeclarativeApp::default()
        .tree(make_widget!(app))
        .view_model(
            DATA,
            ViewModel::produce(|properties| AppData {
                index: Managed::new(ViewModelValue::new(
                    Default::default(),
                    properties.notifier(INDEX),
                )),
            }),
        );

    App::new(AppConfig::default().title("Options View")).run(app);
}
