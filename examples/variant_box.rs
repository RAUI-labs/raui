use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

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
    let variant_name = ctx
        .view_models
        .view_model::<String>(DATA)
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
            "S",
            make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                r: 0.25,
                g: 1.0,
                b: 0.25,
                a: 1.0,
            })),
        )
        .named_slot(
            "D",
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
        .view_model(DATA, ViewModel::new_object("A".to_owned()))
        .event(move |application, event, _, _| {
            let mut data = application
                .view_models
                .get_mut(DATA)
                .unwrap()
                .write_notified::<String>()
                .unwrap();

            if let Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } = event
            {
                if input.state == ElementState::Pressed {
                    if let Some(key) = input.virtual_keycode {
                        match key {
                            VirtualKeyCode::A => {
                                // we modify app data with value that represent active variant name.
                                *data = "A".to_owned();
                            }
                            VirtualKeyCode::S => {
                                *data = "S".to_owned();
                            }
                            VirtualKeyCode::D => {
                                *data = "D".to_owned();
                            }
                            _ => {}
                        }
                    }
                }
            }
            true
        });

    App::new(AppConfig::default().title("Variant Box")).run(app);
}
