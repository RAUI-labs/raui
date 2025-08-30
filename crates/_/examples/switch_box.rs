use raui_app::{
    app::{App, AppConfig, declarative::DeclarativeApp},
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
};
use raui_core::{
    make_widget, pre_hooks,
    view_model::ViewModel,
    widget::{
        component::{
            containers::switch_box::{SwitchBoxProps, switch_box},
            image_box::{ImageBoxProps, image_box},
        },
        context::WidgetContext,
        node::WidgetNode,
        utils::Color,
    },
};

const DATA: &str = "data";

fn use_app(ctx: &mut WidgetContext) {
    ctx.life_cycle.mount(|mut ctx| {
        ctx.view_models
            .bindings(DATA, "")
            .unwrap()
            .bind(ctx.id.to_owned());
    });
}

#[pre_hooks(use_app)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    // we read value from view model created with app builder.
    let active_index = ctx
        .view_models
        .view_model(DATA)
        .unwrap()
        .read::<usize>()
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
    let app = DeclarativeApp::default()
        .tree(make_widget!(app))
        .view_model(DATA, ViewModel::new_object(0usize))
        .event(move |application, event, _, _| {
            let mut data = application
                .view_models
                .get_mut(DATA)
                .unwrap()
                .write_notified::<usize>()
                .unwrap();

            if let Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } = event
                && input.state == ElementState::Pressed
                && let Some(key) = input.virtual_keycode
            {
                match key {
                    VirtualKeyCode::Key1 | VirtualKeyCode::Numpad1 => {
                        // we modify app data with value that represent active switch index.
                        *data = 0;
                    }
                    VirtualKeyCode::Key2 | VirtualKeyCode::Numpad2 => {
                        *data = 1;
                    }
                    VirtualKeyCode::Key3 | VirtualKeyCode::Numpad3 => {
                        *data = 2;
                    }
                    _ => {}
                }
            }
            true
        });

    App::new(AppConfig::default().title("Switch Box")).run(app);
}
