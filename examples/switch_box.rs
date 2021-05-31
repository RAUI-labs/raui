use raui::prelude::*;
use raui_quick_start::{
    tetra::{input::Key, Event},
    RauiQuickStartBuilder,
};

fn app(ctx: WidgetContext) -> WidgetNode {
    // we read value from process context which is stored in app data passed to the app builder.
    let active_index = ctx
        .process_context
        .get_mut::<usize>()
        .copied()
        .unwrap_or_default()
        % 3;

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
        .on_event(|_, event, app_data| match event {
            Event::KeyPressed { key: Key::Num1 } => {
                // we modify app data with value that represent active switch index.
                *app_data.downcast_mut::<usize>().unwrap() = 0;
                // we return `true` which marks RAUI app as dirty (needs to process tree).
                true
            }
            Event::KeyPressed { key: Key::Num2 } => {
                *app_data.downcast_mut::<usize>().unwrap() = 1;
                true
            }
            Event::KeyPressed { key: Key::Num3 } => {
                *app_data.downcast_mut::<usize>().unwrap() = 2;
                true
            }
            _ => false, // we return `false` to tell nothing changed.
        })
        // run app with app data that is read by RAUI and mutated by app events.
        .run_with_app_data(0usize)
        .unwrap();
}
