// Make sure you have seen `view_model_widget` code example first, because this is an evolution of that.

use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget, pre_hooks,
    view_model::{ViewModel, ViewModelValue},
    widget::{
        component::{
            containers::vertical_box::nav_vertical_box,
            image_box::{ImageBoxProps, image_box},
            interactive::{
                button::{ButtonNotifyMessage, ButtonNotifyProps, button},
                navigation::NavItemActive,
            },
            text_box::{TextBoxProps, text_box},
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::text::TextBoxFont,
        utils::Color,
    },
};

const DATA: &str = "data";
const COUNTER: &str = "counter";

struct AppData {
    counter: ViewModelValue<usize>,
}

fn use_app(ctx: &mut WidgetContext) {
    ctx.life_cycle.mount(|mut ctx| {
        // We register View-Model to `app` widget.
        let mut view_model = ViewModel::produce(|properties| AppData {
            counter: ViewModelValue::new(0, properties.notifier(COUNTER)),
        });
        view_model
            .properties
            .bindings(COUNTER)
            .unwrap()
            .bind(ctx.id.to_owned());
        ctx.view_models.widget_register(DATA, view_model);
    });
}

#[pre_hooks(use_app)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    // We read View-Model from `app` widget.
    let counter = ctx
        .view_models
        .widget_view_model(DATA)
        .and_then(|view_model| view_model.read::<AppData>().map(|data| *data.counter))
        .unwrap_or_default();

    make_widget!(nav_vertical_box)
        .listed_slot(make_widget!(text_box).with_props(TextBoxProps {
            text: format!("Counter: {}", counter),
            font: TextBoxFont {
                name: "./demos/hello-world/resources/verdana.ttf".to_owned(),
                size: 48.0,
            },
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            ..Default::default()
        }))
        .listed_slot(make_widget!(trigger))
        .into()
}

fn use_trigger(ctx: &mut WidgetContext) {
    ctx.life_cycle.change(|mut ctx| {
        // We write to View-Model in hierarchy of `app` widget branch,
        // that happen to be parent of this `trigger` widget.
        // Useful for data storages cascading down the hierarchy tree.
        // Each level of the hierarchy can also "override" View-Models.
        let mut app_data = ctx
            .view_models
            .hierarchy_view_model_mut(DATA)
            .unwrap()
            .write::<AppData>()
            .unwrap();

        for msg in ctx.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if msg.trigger_start() {
                    *app_data.counter = app_data.counter.saturating_add(1);
                } else if msg.context_start() {
                    *app_data.counter = app_data.counter.saturating_sub(1);
                }
            }
        }
    });
}

#[pre_hooks(use_trigger)]
fn trigger(mut ctx: WidgetContext) -> WidgetNode {
    make_widget!(button)
        .with_props(NavItemActive)
        .with_props(ButtonNotifyProps(ctx.id.to_owned().into()))
        .named_slot(
            "content",
            make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                r: 0.5,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            })),
        )
        .into()
}

fn main() {
    DeclarativeApp::simple("View-Model - Hierarchy", make_widget!(app));
}
