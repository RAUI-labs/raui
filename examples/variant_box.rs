use raui::prelude::*;
use raui_quick_start::{
    tetra::{input::Key, Event},
    RauiQuickStartBuilder,
};

fn app(ctx: WidgetContext) -> WidgetNode {
    // we read value from process context which is stored in app data passed to the app builder.
    let variant_name = ctx.process_context.get_mut::<String>().cloned();

    make_widget!(variant_box)
        .with_props(VariantBoxProps { variant_name })
        .named_slot(
            "A",
            make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                r: 1.0,
                g: 0.25,
                b: 0.25,
                a: 1.0,
            })),
        )
        .named_slot(
            "B",
            make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                r: 0.25,
                g: 1.0,
                b: 0.25,
                a: 1.0,
            })),
        )
        .named_slot(
            "C",
            make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                r: 0.25,
                g: 0.25,
                b: 1.0,
                a: 1.0,
            })),
        )
        .into()
}

fn main() {
    RauiQuickStartBuilder::default()
        .window_title("Variant Box".to_owned())
        .widget_tree(make_widget!(app).into())
        .build()
        .unwrap()
        .on_event(|_, event, app_data| match event {
            Event::KeyPressed { key: Key::A } => {
                // we modify app data with value that represent active variant name.
                *app_data.downcast_mut::<String>().unwrap() = "A".to_owned();
                // we return `true` which marks RAUI app as dirty (needs to process tree).
                true
            }
            Event::KeyPressed { key: Key::B } => {
                *app_data.downcast_mut::<String>().unwrap() = "B".to_owned();
                true
            }
            Event::KeyPressed { key: Key::C } => {
                *app_data.downcast_mut::<String>().unwrap() = "C".to_owned();
                true
            }
            _ => false,
        })
        // run app with app data that is read by RAUI and mutated by app events.
        .run_with_app_data("A".to_owned())
        .unwrap();
}
