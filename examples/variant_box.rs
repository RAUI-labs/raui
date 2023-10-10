use raui::prelude::*;
use raui_quick_start::{
    tetra::{input::Key, Event},
    RauiQuickStartBuilder,
};

const DATA: &str = "data";

fn app(ctx: WidgetContext) -> WidgetNode {
    // we read value from view model created with app builder.
    let variant_name = ctx
        .view_models
        .get(DATA)
        .unwrap()
        .read::<String>()
        .map(|value| value.to_owned());

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
        .on_event(|_, event, view_models| {
            let mut data = view_models
                .get_mut(DATA)
                .unwrap()
                .write::<String>()
                .unwrap();

            match event {
                Event::KeyPressed { key: Key::A } => {
                    // we modify app data with value that represent active variant name.
                    *data = "A".to_owned();
                    // we return `true` which marks RAUI app as dirty (needs to process tree).
                    true
                }
                Event::KeyPressed { key: Key::B } => {
                    *data = "B".to_owned();
                    true
                }
                Event::KeyPressed { key: Key::C } => {
                    *data = "C".to_owned();
                    true
                }
                _ => false,
            }
        })
        .view_model("data", "A".to_owned())
        .run()
        .unwrap();
}
