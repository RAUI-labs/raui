use raui::prelude::*;
use raui_quick_start::{
    tetra::{input::Key, Event},
    RauiQuickStartBuilder,
};

const DATA: &str = "data";

fn app(ctx: WidgetContext) -> WidgetNode {
    // we read value from view model created with app builder.
    let active_index = ctx
        .view_models
        .view_model::<usize>(DATA)
        .map(|value| *value % 3)
        .unwrap_or_default();

    make_widget!(switch_box)
        .with_props(SwitchBoxProps {
            active_index: Some(active_index),
            ..Default::default()
        })
        .listed_slot(
            make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                r: 1.0,
                g: 0.25,
                b: 0.25,
                a: 1.0,
            })),
        )
        .listed_slot(
            make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                r: 0.25,
                g: 1.0,
                b: 0.25,
                a: 1.0,
            })),
        )
        .listed_slot(
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
        .window_title("Switch Box".to_owned())
        .widget_tree(make_widget!(app).into())
        .build()
        .unwrap()
        .on_event(|_, host, event| {
            let mut data = host
                .application
                .view_models
                .get_mut(DATA)
                .unwrap()
                .write::<usize>()
                .unwrap();

            match event {
                Event::KeyPressed { key: Key::Num1 } => {
                    // we modify app data with value that represent active switch index.
                    *data = 0;
                    // we return `true` which marks RAUI app as dirty (needs to process tree).
                    true
                }
                Event::KeyPressed { key: Key::Num2 } => {
                    *data = 1;
                    true
                }
                Event::KeyPressed { key: Key::Num3 } => {
                    *data = 2;
                    true
                }
                _ => false, // we return `false` to tell nothing changed.
            }
        })
        .view_model("data", 0usize)
        .run()
        .unwrap();
}
