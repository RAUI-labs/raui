use raui_app::app::{App, AppConfig, declarative::DeclarativeApp};
use raui_core::{
    make_widget, pre_hooks,
    view_model::{ViewModel, ViewModelValue},
    widget::{
        component::{
            containers::float_box::{
                FloatBoxChange, FloatBoxChangeMessage, FloatBoxNotifyProps, FloatBoxProps,
                FloatBoxState, float_box,
            },
            image_box::{ImageBoxProps, image_box},
            interactive::{
                float_view::float_view_control,
                navigation::{NavItemActive, use_nav_container_active},
            },
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::{
            content::ContentBoxItemLayout,
            image::{ImageBoxColor, ImageBoxMaterial, ImageBoxSizeValue},
        },
        utils::{Color, Rect, Vec2},
    },
};

const DATA: &str = "data";
const PANELS: &str = "panels";

// AppData holds list of floating panels positions and their color.
struct AppData {
    panels: ViewModelValue<Vec<(Vec2, Color)>>,
}

fn use_app(ctx: &mut WidgetContext) {
    ctx.life_cycle.mount(|mut ctx| {
        ctx.view_models
            .bindings(DATA, PANELS)
            .unwrap()
            .bind(ctx.id.to_owned());
    });

    ctx.life_cycle.unmount(|mut ctx| {
        ctx.view_models
            .bindings(DATA, PANELS)
            .unwrap()
            .unbind(ctx.id);
    });

    ctx.life_cycle.change(|mut ctx| {
        let mut view_model = ctx
            .view_models
            .view_model_mut(DATA)
            .unwrap()
            .write::<AppData>()
            .unwrap();

        for msg in ctx.messenger.messages {
            // We listen for float box change messages sent from `float_view_control`
            // widgets and move sender panel by delta of change.
            if let Some(msg) = msg.as_any().downcast_ref::<FloatBoxChangeMessage>() {
                if let Ok(index) = msg.sender.key().parse::<usize>() {
                    if let FloatBoxChange::RelativePosition(delta) = msg.change {
                        if let Some((position, _)) = view_model.panels.get_mut(index) {
                            position.x += delta.x;
                            position.y += delta.y;
                        }
                    }
                }
            }
        }
    });
}

#[pre_hooks(use_nav_container_active, use_app)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    let view_model = ctx
        .view_models
        .view_model(DATA)
        .unwrap()
        .read::<AppData>()
        .unwrap();

    make_widget!(float_box)
        .with_props(FloatBoxProps {
            bounds_left: Some(-300.0),
            bounds_right: Some(600.0),
            bounds_top: Some(-300.0),
            bounds_bottom: Some(400.0),
        })
        .with_props(FloatBoxState {
            position: Vec2 { x: 0.0, y: 0.0 },
            zoom: 2.0,
        })
        .listed_slot(
            // `float_view_control` widget reacts to dragging action and sends
            // that dragging movement delta to widget that wants to be notified.
            // In this case, we want to notify `float_box` widget so it will
            // reposition its content panels.
            make_widget!(float_view_control)
                .key("panning")
                .with_props(NavItemActive)
                // we make sure panning control fills entire area and stays
                // in its bounds no matter how content gets repositioned.
                .with_props(ContentBoxItemLayout {
                    anchors: Rect {
                        left: 0.0,
                        top: 0.0,
                        right: 1.0,
                        bottom: 1.0,
                    },
                    keep_in_bounds: true.into(),
                    ..Default::default()
                })
                .named_slot(
                    "content",
                    make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                        r: 0.3,
                        g: 0.3,
                        b: 0.3,
                        a: 1.0,
                    })),
                ),
        )
        .listed_slots(
            view_model
                .panels
                .iter()
                .enumerate()
                .map(|(index, (position, color))| {
                    // we also use `float_view_control` widget for panels so
                    // they can be dragged around float box.
                    make_widget!(float_view_control)
                        .key(index)
                        .with_props(NavItemActive)
                        .with_props(FloatBoxNotifyProps(ctx.id.to_owned().into()))
                        .with_props(ContentBoxItemLayout {
                            offset: *position,
                            ..Default::default()
                        })
                        .named_slot(
                            "content",
                            make_widget!(image_box).with_props(ImageBoxProps {
                                width: ImageBoxSizeValue::Exact(200.0),
                                height: ImageBoxSizeValue::Exact(150.0),
                                material: ImageBoxMaterial::Color(ImageBoxColor {
                                    color: *color,
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }),
                        )
                }),
        )
        .into()
}

fn main() {
    let panels = vec![
        (
            Vec2 { x: 0.0, y: 0.0 },
            Color {
                r: 1.0,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            },
        ),
        (
            Vec2 { x: 100.0, y: 100.0 },
            Color {
                r: 0.5,
                g: 1.0,
                b: 0.5,
                a: 1.0,
            },
        ),
        (
            Vec2 { x: 200.0, y: 200.0 },
            Color {
                r: 0.5,
                g: 0.5,
                b: 1.0,
                a: 1.0,
            },
        ),
    ];

    let app = DeclarativeApp::default()
        .tree(make_widget!(app))
        .view_model(
            DATA,
            ViewModel::produce(|properties| AppData {
                panels: ViewModelValue::new(panels, properties.notifier(PANELS)),
            }),
        );

    App::new(AppConfig::default().title("Float View")).run(app);
}
