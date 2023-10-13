// Make sure you have seen `text_box` code example first, because this is an evolution of that.

use raui::prelude::*;
use raui_quick_start::{
    tetra::{input::Key, Event},
    RauiQuickStartBuilder,
};

// Name of View-Model instance.
const DATA: &str = "data";
// Name of View-Model property notification.
const COUNTER: &str = "counter";

// View-Model data type.
struct AppData {
    // View-Model value wrapper that automatically notifies View-Model
    // about change on mutation.
    counter: ViewModelValue<usize>,
}

// We use hook to bind widget to and unbind from View-Model instance.
// This will make RAUI application automatically rebuild widgets tree
// on change in View-Model data.
// BTW. We could omit unbinding, since widgets unbind automatically
// on unmount, but this is here to showcase how to do it manually.
fn use_app(ctx: &mut WidgetContext) {
    ctx.life_cycle.mount(|mut ctx| {
        ctx.view_models
            .bindings(DATA, COUNTER)
            .unwrap()
            .bind(ctx.id.to_owned());
    });
    ctx.life_cycle.unmount(|mut ctx| {
        ctx.view_models
            .bindings(DATA, COUNTER)
            .unwrap()
            .unbind(ctx.id);
    });
}

#[pre_hooks(use_app)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    // We read app data from view model created with app builder.
    let app_data = ctx.view_models.view_model::<AppData>(DATA).unwrap();

    make_widget!(text_box)
        .with_props(TextBoxProps {
            // Use View-Model data to render widget on change.
            text: format!("Counter: {}", *app_data.counter),
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
    // Create View-Model for `AppData`.
    let mut view_model = ViewModel::produce(|properties| AppData {
        // We use View-Model properties to create notifiers for properties.
        counter: ViewModelValue::new(0, properties.notifier(COUNTER)),
    });
    // Get lazy shared reference to View-Model data for later use
    // on the host side of application.
    let app_data = view_model.lazy::<AppData>().unwrap();

    RauiQuickStartBuilder::default()
        .window_title("View-model".to_owned())
        .widget_tree(make_widget!(app).into())
        .build()
        .unwrap()
        .on_event(move |_, event, _| {
            match event {
                Event::KeyPressed { key: Key::Space } => {
                    // Here we use that shared reference to `AppData`
                    // to mutate it and notify UI.
                    *app_data.write().unwrap().counter += 1;
                }
                _ => {}
            }
            false
        })
        .view_model_raw(DATA, view_model)
        .run()
        .unwrap();
}
