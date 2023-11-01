// Make sure you have seen `portal_box` code example first, because this is an evolution of that.

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
    let idref = WidgetRef::default();
    // we read value from view model created with app builder.
    let data = ctx
        .view_models
        .view_model::<(bool, bool, bool)>(DATA)
        .unwrap();

    make_widget!(content_box)
        .idref(idref.clone())
        .with_shared_props(PortalsContainer(idref.into()))
        .listed_slot(
            make_widget!(horizontal_box)
                .with_props(HorizontalBoxProps {
                    separation: 25.0,
                    ..Default::default()
                })
                .listed_slot(
                    make_widget!(icon)
                        // clear this flex box item layout (no frowing, shrinking or filling).
                        .with_props(FlexBoxItemLayout::cleared())
                        // pass context box state read from app data.
                        .with_props(data.0)
                        // set icon color.
                        .with_props(Color {
                            r: 1.0,
                            g: 0.25,
                            b: 0.25,
                            a: 1.0,
                        })
                        // tell context widget how to position it relative to the content widget.
                        .with_props(PivotBoxProps {
                            pivot: 0.0.into(),
                            align: 0.0.into(),
                        }),
                )
                .listed_slot(
                    make_widget!(icon)
                        .with_props(FlexBoxItemLayout::cleared())
                        .with_props(data.1)
                        .with_props(Color {
                            r: 0.25,
                            g: 1.0,
                            b: 0.25,
                            a: 1.0,
                        })
                        .with_props(PivotBoxProps {
                            pivot: 0.5.into(),
                            align: 0.5.into(),
                        }),
                )
                .listed_slot(
                    make_widget!(icon)
                        .with_props(FlexBoxItemLayout::cleared())
                        .with_props(data.2)
                        .with_props(Color {
                            r: 0.25,
                            g: 0.25,
                            b: 1.0,
                            a: 1.0,
                        })
                        .with_props(PivotBoxProps {
                            pivot: 1.0.into(),
                            align: 1.0.into(),
                        }),
                ),
        )
        .into()
}

// custom icon component composed out of icon image as its content and context image that we show
// when bool props value is true.
fn icon(ctx: WidgetContext) -> WidgetNode {
    // we use `portals_context_box` to allow this context box properly calculate context widget
    // relative to the portals container.
    make_widget!(portals_context_box)
        // pass pivot props to context box,
        .with_props(ctx.props.read_cloned_or_default::<PivotBoxProps>())
        .with_props(ContextBoxProps {
            // read bool props value and use it to tell if context widget is gonna be shown.
            show: ctx.props.read_cloned_or_default::<bool>(),
        })
        // put colored image box as content widget.
        .named_slot(
            "content",
            make_widget!(image_box).with_props(ImageBoxProps {
                material: ImageBoxMaterial::Color(ImageBoxColor {
                    color: ctx.props.read_cloned_or_default::<Color>(),
                    ..Default::default()
                }),
                width: ImageBoxSizeValue::Exact(100.0),
                height: ImageBoxSizeValue::Exact(100.0),
                ..Default::default()
            }),
        )
        // put gray image box as context widget.
        .named_slot(
            "context",
            make_widget!(image_box).with_props(ImageBoxProps {
                material: ImageBoxMaterial::Color(ImageBoxColor {
                    color: Color {
                        r: 0.25,
                        g: 0.25,
                        b: 0.25,
                        a: 1.0,
                    },
                    ..Default::default()
                }),
                width: ImageBoxSizeValue::Exact(150.0),
                height: ImageBoxSizeValue::Exact(50.0),
                ..Default::default()
            }),
        )
        .into()
}

fn main() {
    let app = DeclarativeApp::default()
        .tree(make_widget!(app))
        // we use tuple of 3 bools that will represent state of individual context box.
        .view_model(DATA, ViewModel::new_object((false, true, false)))
        .event(move |application, event, _| {
            let mut data = application
                .view_models
                .get_mut(DATA)
                .unwrap()
                .write_notified::<(bool, bool, bool)>()
                .unwrap();

            if let Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } = event
            {
                if input.state == ElementState::Pressed {
                    if let Some(key) = input.virtual_keycode {
                        match key {
                            VirtualKeyCode::Key1 | VirtualKeyCode::Numpad1 => {
                                // change state of given context box in app data.
                                data.0 = !data.0;
                            }
                            VirtualKeyCode::Key2 | VirtualKeyCode::Numpad2 => {
                                data.1 = !data.1;
                            }
                            VirtualKeyCode::Key3 | VirtualKeyCode::Numpad3 => {
                                data.2 = !data.2;
                            }
                            _ => {}
                        }
                    }
                }
            }
            true
        });

    App::new(AppConfig::default().title("Context Box")).run(app);
}
