// Make sure you have seen `view_model` code example first, because this is an evolution of that.

use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

// animation message name used to trigger counter change.
const TICK: &str = "tick";
// View-Model name.
const DATA: &str = "data";
// View-Model proeprty name.
const COUNTER: &str = "counter";

struct AppData {
    counter: ViewModelValue<usize>,
}

fn use_app(ctx: &mut WidgetContext) {
    ctx.life_cycle.mount(|mut ctx| {
        // First we register View-Model owned by this widget.
        // Widget View-Models are replacements for widget state useful
        // in situations where widget state should not be serializable props.
        let mut view_model = ViewModel::produce(|properties| AppData {
            counter: ViewModelValue::new(0, properties.notifier(COUNTER)),
        });
        view_model
            .properties
            .bindings(COUNTER)
            .unwrap()
            .bind(ctx.id.to_owned());
        ctx.view_models.widget_register(DATA, view_model);

        // Then we register new looped tick animation to trigger counter ticks.
        let _ = ctx.animator.change(
            TICK,
            Some(Animation::Looped(Box::new(Animation::Sequence(vec![
                Animation::Value(AnimatedValue {
                    duration: 1.0,
                    ..Default::default()
                }),
                Animation::Message(TICK.to_owned()),
            ])))),
        );
    });

    ctx.life_cycle.change(|mut ctx| {
        // We get View-Model of this widget.
        let mut app_data = ctx
            .view_models
            .widget_view_model_mut::<AppData>(DATA)
            .unwrap();

        // And then we react for tick messages from animation.
        for msg in ctx.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<AnimationMessage>() {
                if msg.0 == TICK {
                    *app_data.counter += 1;
                }
            }
        }
    });
}

#[pre_hooks(use_app)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    // Because widget rendering can happen before widget mount,
    // we need to fallback to some default value in case View-Model
    // is not yet available.
    let counter = ctx
        .view_models
        .widget_view_model::<AppData>(DATA)
        .map(|data| *data.counter)
        .unwrap_or_default();

    make_widget!(text_box)
        .with_props(TextBoxProps {
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
        })
        .into()
}

fn main() {
    DeclarativeApp::simple("View-Model - Widget", make_widget!(app));
}
