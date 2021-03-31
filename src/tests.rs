#![cfg(test)]

use crate::prelude::*;
use std::str::FromStr;

#[test]
fn test_threadsafe() {
    fn foo<T>()
    where
        T: Send + Sync,
    {
        println!("* {} is threadsafe!", std::any::type_name::<T>());
    }

    foo::<Application>();
    foo::<WidgetRef>();
    foo::<WidgetIdOrRef>();
    foo::<DataBinding<()>>();
}

#[test]
fn test_macro() {
    fn app(_context: WidgetContext) -> WidgetNode {
        widget! {()}
    }

    fn text(_context: WidgetContext) -> WidgetNode {
        widget! {()}
    }

    println!("{:#?}", widget! {()});
    println!(
        "{:#?}",
        widget! {
            (app)
        }
    );
    println!(
        "{:#?}",
        widget! {
            (app: {()})
        }
    );
    println!(
        "{:#?}",
        widget! {
            (app {
                ass = (text: {"ass".to_owned()})
                hole = ()
            })
        }
    );
    println!(
        "{:#?}",
        widget! {
            (app: {()} {
                ass = (text: {"ass".to_owned()})
                hole = ()
            })
        }
    );
    println!(
        "{:#?}",
        widget! {
            (app [
                (text: {"hole".to_owned()})
                ()
            ])
        }
    );
    println!(
        "{:#?}",
        widget! {
            (app: {()} [
                (text: {"hole".to_owned()})
                ()
            ])
        }
    );
    println!(
        "{:#?}",
        widget! {
            (#{"app"} app {
                ass = (text: {"ass".to_owned()})
                hole = ()
            } [
                (text: {"hole".to_owned()})
                ()
            ])
        }
    );
    println!(
        "{:#?}",
        widget! {
            (#{42} app: {()} {
                ass = (text: {"ass".to_owned()})
                hole = {widget! {()}}
            } [
                (text: {"hole".to_owned()})
                {{WidgetUnitNode::None}}
                {{WidgetNode::None}}
            ])
        }
    );
}

#[test]
#[allow(dead_code)]
#[cfg(feature = "html")]
fn test_hello_world() {
    use serde::{Deserialize, Serialize};
    use std::convert::TryInto;

    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    struct AppProps {
        #[serde(default)]
        pub index: usize,
    }
    implement_props_data!(AppProps);

    let v = AppProps { index: 42 };
    let s = v.to_prefab().unwrap();
    println!("* SERIALIZED APP PROPS: {:?}", s);
    let d = AppProps::from_prefab(s).unwrap();
    println!("* DESERIALIZED APP PROPS: {:?}", d);

    // convenient macro that produces widget component processing function.
    widget_component! {
        // <component name> ( [list of context data to unpack into scope] )
        app(props, named_slots) {
            // easy way to get widgets from named slots.
            unpack_named_slots!(named_slots => { title, content });
            let index = props.read::<AppProps>().map(|p| p.index).unwrap_or(0);

            // we always return new widgets tree.
            widget! {
                // Forgive me the syntax, i'll make a JSX-like one soon using procedural macros.
                // `#{key}` - provided value gives a unique name to node. keys allows widgets
                //      to save state between render calls. here we just pass key of this widget.
                // `vertical_box` - name of widget component to use.
                // `[...]` - listed widget slots. here we just put previously unpacked named slots.
                (#{index} vertical_box [
                    {title}
                    {content}
                ])
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    struct ButtonState {
        #[serde(default)]
        pub pressed: bool,
    }
    implement_props_data!(ButtonState);

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    enum ButtonAction {
        Pressed,
        Released,
    }
    implement_message_data!(ButtonAction);

    widget_hook! {
        use_empty(life_cycle) {
            life_cycle.mount(|_| {
                println!("* EMPTY MOUNTED");
            });

            life_cycle.change(|_| {
                println!("* EMPTY CHANGED");
            });

            life_cycle.unmount(|_| {
                println!("* EMPTY UNMOUNTED");
            });
        }
    }

    // you use life cycle hooks for storing closures that will be called when widget will be
    // mounted/changed/unmounted. they exists for you to be able to reuse some common logic across
    // multiple components. each closure provides arguments such as:
    // - widget id
    // - widget state
    // - message sender (this one is used to message other widgets you know about)
    // - signal sender (this one is used to message application host)
    // although this hook uses only life cycle, you can make different hooks that use many
    // arguments, even use context you got from the component!
    widget_hook! {
        use_button(key, life_cycle) [use_empty] {
            life_cycle.mount(|context| {
                println!("* BUTTON MOUNTED: {}", context.id.key());
                drop(context.state.write(ButtonState { pressed: false }));
            });

            life_cycle.change(|context| {
                println!("* BUTTON CHANGED: {}", context.id.key());
                for msg in context.messenger.messages {
                    if let Some(msg) = msg.as_any().downcast_ref::<ButtonAction>() {
                        let pressed = match msg {
                            ButtonAction::Pressed => true,
                            ButtonAction::Released => false,
                        };
                        println!("* BUTTON ACTION: {:?}", msg);
                        drop(context.state.write(ButtonState { pressed }));
                        drop(context.signals.write(*msg));
                    }
                }
            });

            life_cycle.unmount(|context| {
                println!("* BUTTON UNMOUNTED: {}", context.id.key());
            });
        }
    }

    widget_component! {
        button(key, props) [use_button] {
            println!("* PROCESS BUTTON: {}", key);

            widget!{
                (#{key} text: {props.clone()})
            }
        }
    }

    widget_component! {
        title_bar(key, props) {
            let title = props.read_cloned_or_default::<String>();

            widget! {
                (#{key} text: {title})
            }
        }
    }

    widget_component! {
        vertical_box(id, key, listed_slots) {
            // listed slots are just widget node children.
            // here we just unwrap widget units (final atomic UI elements that renderers read).
            let items = listed_slots
                .into_iter()
                .map(|slot| FlexBoxItemNode {
                    slot: slot.try_into().expect("Cannot convert slot to WidgetUnitNode!"),
                    ..Default::default()
                })
                .collect::<Vec<_>>();

            // we use `{{{ ... }}}` to inform macro that this is widget unit.
            widget! {{{
                FlexBoxNode {
                    id: id.to_owned(),
                    items,
                    ..Default::default()
                }
            }}}
        }
    }

    widget_component! {
        text(id, key, props) {
            let text = props.read_cloned_or_default::<String>();

            widget!{{{
                TextBoxNode {
                    id: id.to_owned(),
                    text,
                    ..Default::default()
                }
            }}}
        }
    }

    let mapping = CoordsMapping::new(Rect {
        left: 0.0,
        right: 1024.0,
        top: 0.0,
        bottom: 576.0,
    });

    let mut application = Application::new();
    application.setup(setup);
    application.register_component("app", app);
    let tree = widget! {
        (app {
            // <named slot name> = ( <widget to put in a slot> )
            title = (title_bar: {"Hello".to_owned()})
            content = (vertical_box [
                (#{"hi"} button: {"Say hi!".to_owned()})
                (#{"exit"} button: {"Close".to_owned()})
            ])
        })
    };
    println!("* INPUT:\n{:#?}", tree);

    // some dummy widget tree renderer.
    // it reads widget unit tree and transforms it into target format.
    let mut renderer = HtmlRenderer::default();

    println!("* PROCESS");
    // `apply()` sets new widget tree.
    application.apply(tree);
    // `render()` calls renderer to perform transformations on processed application widget tree.
    if let Ok(output) = application.render(&mapping, &mut renderer) {
        println!("* OUTPUT:\n{}", output);
    }

    println!("* PROCESS");
    // by default application won't process widget tree if nothing was changed.
    // "change" is either any widget state change, or new message sent to any widget (messages
    // can be sent from application host, for example a mouse click, or from another widget).
    application.forced_process();
    if let Ok(output) = application.render(&mapping, &mut renderer) {
        println!("* OUTPUT:\n{}", output);
    }

    let tree = widget! {
        (app)
    };
    println!("* INPUT:\n{:#?}", tree);
    println!("* PROCESS");
    application.apply(tree);
    if let Ok(output) = application.render(&mapping, &mut HtmlRenderer::default()) {
        println!("* OUTPUT:\n{}", output);
    }

    let p = ContentBoxItemLayout {
        anchors: Rect {
            left: 0.0,
            right: 1.0,
            top: 0.0,
            bottom: 1.0,
        },
        ..Default::default()
    };
    let c = widget! { (image_box: {p})};
    let s = application.serialize_node(&c).unwrap();
    println!("* SERIALIZED COMPONENT: {:#?}", s);
    let d = application.deserialize_node(s).unwrap();
    println!("* DESERIALIZED COMPONENT: {:#?}", d);

    let p = ContentBoxItemLayout {
        anchors: Rect {
            left: 0.0,
            right: 1.0,
            top: 0.0,
            bottom: 1.0,
        },
        ..Default::default()
    };
    let c = widget! { (image_box: {p})};
    let s = application.serialize_node(&c).unwrap();
    println!("* SERIALIZED COMPONENT VALUE: {:#?}", s);
    let d = application.deserialize_node(s).unwrap();
    println!("* DESERIALIZED COMPONENT VALUE: {:#?}", d);

    let s = serde_yaml::from_str::<serde_yaml::Value>(
        r#"
    Component:
        type_name: app
        key: app
    "#,
    )
    .unwrap();
    println!(
        "* SERIALIZED COMPONENT VALUE: {}",
        serde_yaml::to_string(&s).unwrap()
    );
    let d = application.deserialize_node(s).unwrap();
    println!("* DESERIALIZED COMPONENT VALUE: {:#?}", d);
}

#[test]
fn test_layout_no_wrap() {
    let mut layout_engine = DefaultLayoutEngine::default();
    let mapping = CoordsMapping::new(Rect {
        left: 0.0,
        right: 1024.0,
        top: 0.0,
        bottom: 576.0,
    });

    let tree = widget! {{{
        FlexBoxNode {
            id: WidgetId::from_str("type:/list").unwrap(),
            direction: FlexBoxDirection::VerticalTopToBottom,
            separation: 10.0,
            items: vec![
                FlexBoxItemNode {
                    slot: SizeBoxNode {
                        id: WidgetId::from_str("type:/list/0").unwrap(),
                        width: SizeBoxSizeValue::Fill,
                        height: SizeBoxSizeValue::Exact(100.0),
                        ..Default::default()
                    }.into(),
                    layout: FlexBoxItemLayout {
                        fill: 1.0,
                        ..Default::default()
                    },
                },
                FlexBoxItemNode {
                    slot: SizeBoxNode {
                        id: WidgetId::from_str("type:/list/1").unwrap(),
                        width: SizeBoxSizeValue::Fill,
                        height: SizeBoxSizeValue::Fill,
                        ..Default::default()
                    }.into(),
                    layout: FlexBoxItemLayout {
                        fill: 1.0,
                        grow: 1.0,
                        ..Default::default()
                    },
                },
                FlexBoxItemNode {
                    slot: SizeBoxNode {
                        id: WidgetId::from_str("type:/list/2").unwrap(),
                        width: SizeBoxSizeValue::Fill,
                        height: SizeBoxSizeValue::Fill,
                        ..Default::default()
                    }.into(),
                    layout: FlexBoxItemLayout {
                        fill: 1.0,
                        grow: 2.0,
                        ..Default::default()
                    },
                },
            ],
            ..Default::default()
        }
    }}};

    let mut application = Application::new();
    application.apply(tree);
    application.forced_process();
    println!(
        "* TREE INSPECTION:\n{:#?}",
        application.rendered_tree().inspect()
    );
    if application.layout(&mapping, &mut layout_engine).is_ok() {
        println!("* LAYOUT:\n{:#?}", application.layout_data());
    }
}

#[test]
fn test_layout_wrapping() {
    let mut layout_engine = DefaultLayoutEngine::default();
    let mapping = CoordsMapping::new(Rect {
        left: 0.0,
        right: 1024.0,
        top: 0.0,
        bottom: 576.0,
    });

    let tree = widget! {{{
        FlexBoxNode {
            id: WidgetId::from_str("type:/list").unwrap(),
            direction: FlexBoxDirection::HorizontalLeftToRight,
            separation: 10.0,
            wrap: true,
            items: vec![
                FlexBoxItemNode {
                    slot: SizeBoxNode {
                        id: WidgetId::from_str("type:/list/0").unwrap(),
                        width: SizeBoxSizeValue::Fill,
                        height: SizeBoxSizeValue::Exact(100.0),
                        ..Default::default()
                    }.into(),
                    layout: FlexBoxItemLayout {
                        basis: Some(400.0),
                        fill: 1.0,
                        grow: 1.0,
                        ..Default::default()
                    },
                },
                FlexBoxItemNode {
                    slot: SizeBoxNode {
                        id: WidgetId::from_str("type:/list/1").unwrap(),
                        width: SizeBoxSizeValue::Fill,
                        height: SizeBoxSizeValue::Exact(200.0),
                        ..Default::default()
                    }.into(),
                    layout: FlexBoxItemLayout {
                        basis: Some(400.0),
                        fill: 1.0,
                        grow: 1.0,
                        ..Default::default()
                    },
                },
                FlexBoxItemNode {
                    slot: SizeBoxNode {
                        id: WidgetId::from_str("type:/list/2").unwrap(),
                        width: SizeBoxSizeValue::Fill,
                        height: SizeBoxSizeValue::Exact(50.0),
                        ..Default::default()
                    }.into(),
                    layout: FlexBoxItemLayout {
                        basis: Some(400.0),
                        fill: 1.0,
                        grow: 2.0,
                        ..Default::default()
                    },
                },
            ],
            ..Default::default()
        }
    }}};

    let mut application = Application::new();
    application.apply(tree);
    application.forced_process();
    println!(
        "* TREE INSPECTION:\n{:#?}",
        application.rendered_tree().inspect()
    );
    if application.layout(&mapping, &mut layout_engine).is_ok() {
        println!("* LAYOUT:\n{:#?}", application.layout_data());
    }
}

#[test]
fn test_components() {
    let mut layout_engine = DefaultLayoutEngine::default();
    let mapping = CoordsMapping::new(Rect {
        left: 0.0,
        right: 1024.0,
        top: 0.0,
        bottom: 576.0,
    });

    let tree = widget! {
        (#{"app"} vertical_box: {
            VerticalBoxProps {
                separation: 10.0,
                ..Default::default()
            }
        } [
            (size_box: {
                Props::new(SizeBoxProps {
                    height: SizeBoxSizeValue::Exact(100.0),
                    ..Default::default()
                }).with(FlexBoxItemLayout {
                    basis: Some(400.0),
                    fill: 1.0,
                    grow: 1.0,
                    ..Default::default()
                })
            })
            (size_box: {
                Props::new(SizeBoxProps {
                    height: SizeBoxSizeValue::Exact(200.0),
                    ..Default::default()
                }).with(FlexBoxItemLayout {
                    basis: Some(400.0),
                    fill: 1.0,
                    grow: 1.0,
                    ..Default::default()
                })
            })
            (size_box: {
                Props::new(SizeBoxProps {
                    height: SizeBoxSizeValue::Exact(50.0),
                    ..Default::default()
                }).with(FlexBoxItemLayout {
                    basis: Some(400.0),
                    fill: 1.0,
                    grow: 2.0,
                    ..Default::default()
                })
            })
        ])
    };

    let mut application = Application::new();
    application.apply(tree);
    application.forced_process();
    println!(
        "* TREE INSPECTION:\n{:#?}",
        application.rendered_tree().inspect()
    );
    if application.layout(&mapping, &mut layout_engine).is_ok() {
        println!("* LAYOUT:\n{:#?}", application.layout_data());
    }
}

#[test]
fn test_refs() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Default, Clone, Serialize, Deserialize)]
    struct AppState {
        test_ref: WidgetRef,
    }
    implement_props_data!(AppState);

    widget_hook! {
        use_test(life_cycle) {
            life_cycle.change(|context| {
                for msg in context.messenger.messages {
                    if msg.as_any().downcast_ref::<()>().is_some() {
                        println!("Test got message");
                        drop(context.signals.write(()));
                    }
                }
            });
        }
    }

    widget_component! {
        test(key) [use_test] {
            println!("Render test: {:?}", key);
            widget!{()}
        }
    }

    widget_hook! {
        use_app(life_cycle) {
            life_cycle.mount(|context| {
                println!("Register app");
                drop(context.state.write(AppState::default()));
                drop(context.signals.write(true));
            });

            life_cycle.change(|context| {
                for msg in context.messenger.messages {
                    if msg.as_any().downcast_ref::<()>().is_some() {
                        println!("App got message");
                        let state = context.state.read_cloned_or_default::<AppState>();
                        if let Some(id) = state.test_ref.read() {
                            println!("App send message to: {:?}", id);
                            context.messenger.write(id, ());
                        }
                    }
                }
            });
        }
    }

    widget_component! {
        app(key, state) [use_app] {
            println!("Render app: {:?}", key);
            let state = state.read_cloned_or_default::<AppState>();

            widget! {
                (#{key} | {state.test_ref} test)
            }
        }
    }

    let mut appid = WidgetId::default();
    let mut application = Application::new();
    application.apply(widget! { (#{"app"} app) });
    println!("* Process");
    application.forced_process();
    for (id, msg) in application.signals() {
        if let Some(msg) = msg.as_any().downcast_ref::<bool>() {
            if *msg {
                println!("Registered app: {:?}", id);
                appid = id.to_owned();
            }
        }
    }
    println!("* Process");
    application.forced_process();
    println!("* Send message to app");
    application.send_message(&appid, ());
    println!("* Process");
    application.forced_process();
    println!("* Process");
    application.forced_process();
    assert!(application
        .signals()
        .iter()
        .any(|(_, msg)| msg.as_any().downcast_ref::<()>().is_some()));
}

#[test]
fn test_scroll_box() {
    fn run<F>(
        application: &mut Application,
        mapping: &CoordsMapping,
        layout_engine: &mut DefaultLayoutEngine,
        interactions: &mut DefaultInteractionsEngine,
        actions: &[Option<Interaction>],
        mut f: F,
    ) where
        F: FnMut(WidgetId, Message),
    {
        for action in actions.iter() {
            println!("* Process");
            application.forced_process();
            application
                .layout(mapping, layout_engine)
                .expect("Failed layouting");
            if let Some(action) = action {
                println!("* Interact: {:?}", action);
                interactions.interact(action.to_owned());
            }
            application
                .interact(interactions)
                .expect("Failed integration");
            println!("* Read signals");
            for (id, msg) in application.consume_signals() {
                println!("* Signal: {:?} -> {:?}", id, msg);
                f(id, msg);
            }
        }
    }

    let mut layout_engine = DefaultLayoutEngine::default();
    let mapping = CoordsMapping::new(Rect {
        left: 0.0,
        right: 100.0,
        top: 0.0,
        bottom: 100.0,
    });

    widget_hook! {
        use_app(life_cycle) {
            life_cycle.change(|context| {
                for msg in context.messenger.messages {
                    println!("* App message: {:#?}", msg);
                }
            });
        }
    }

    widget_component! {
        app(id, key) [use_nav_container, use_app] {
            let scroll_props = Props::new(NavContainerActive)
                .with(NavItemActive)
                .with(ScrollViewNotifyProps(id.to_owned().into()))
                .with(ScrollViewRange::default());
            let size_props = SizeBoxProps {
                width: SizeBoxSizeValue::Exact(200.0),
                height: SizeBoxSizeValue::Exact(200.0),
                ..Default::default()
            };

            widget! {
                (#{key} nav_scroll_box: {scroll_props} {
                    content = (#{"button"} button: {NavItemActive} {
                        content = (#{"size"} size_box: {size_props})
                    })
                    scrollbars = (#{"scrollbars"} nav_scroll_box_side_scrollbars)
                })
            }
        }
    }

    let mut button = WidgetId::default();
    let mut application = Application::new();
    let mut interactions = DefaultInteractionsEngine::default();
    interactions.deselect_when_no_button_found = true;
    application.apply(widget! { (#{"app"} app: {NavContainerActive}) });

    run(
        &mut application,
        &mapping,
        &mut layout_engine,
        &mut interactions,
        &[None],
        |id, msg| {
            if let Some(NavSignal::Register(NavType::Button(_))) = msg.as_any().downcast_ref() {
                println!("* Button registered: {:?}", id);
                button = id.to_owned();
            }
        },
    );

    run(
        &mut application,
        &mapping,
        &mut layout_engine,
        &mut interactions,
        &[
            None,
            None,
            None,
            Some(Interaction::Navigate(NavSignal::Select(
                button.to_owned().into(),
            ))),
            Some(Interaction::Navigate(NavSignal::Jump(NavJump::Scroll(
                NavScroll::Factor(Vec2 { x: 2.0, y: 2.0 }, false),
            )))),
            Some(Interaction::Navigate(NavSignal::Jump(NavJump::Scroll(
                NavScroll::Units(Vec2 { x: -50.0, y: -50.0 }, true),
            )))),
            Some(Interaction::Navigate(NavSignal::Jump(NavJump::Scroll(
                NavScroll::Widget(button.into(), Vec2 { x: 0.5, y: 0.5 }),
            )))),
            Some(Interaction::PointerDown(
                PointerButton::Trigger,
                Vec2 { x: 95.0, y: 0.0 },
            )),
            Some(Interaction::PointerMove(Vec2 { x: 95.0, y: 45.0 })),
            Some(Interaction::PointerMove(Vec2 { x: 95.0, y: 90.0 })),
            Some(Interaction::PointerUp(
                PointerButton::Trigger,
                Vec2 { x: 95.0, y: 90.0 },
            )),
            None,
            None,
            None,
        ],
        |_, _| {},
    );
}

#[test]
fn test_immediate_mode() {
    fn use_app(context: &mut WidgetContext) {
        context.use_hook(use_nav_container);
    }

    fn app(mut context: WidgetContext) -> WidgetNode {
        context.use_hook(use_app);

        let title = context.named_slots.remove("title").unwrap_or_default();
        let content = context.named_slots.remove("content").unwrap_or_default();

        make_widget!(content_box)
            .key(&context.key)
            .listed_slot(title)
            .listed_slot(content)
            .into()
    }

    fn make_app(key: &str) -> WidgetComponent {
        make_widget!(app).key(key)
    }

    fn make_text_box(key: &str, text: &str) -> WidgetComponent {
        make_widget!(text_box).key(key).with_props(TextBoxProps {
            text: text.to_owned(),
            ..Default::default()
        })
    }

    fn make_button(key: &str, text: &str) -> WidgetComponent {
        make_widget!(button)
            .key(key)
            .with_props(NavItemActive)
            .named_slot("content", make_text_box("text", text))
    }

    let mut application = Application::new();
    application.apply(
        make_app("app")
            .with_props(NavContainerActive)
            .named_slot("title", make_text_box("text", "Hello, World!"))
            .named_slot("content", make_button("button", "Click me!"))
            .into(),
    );
}

#[test]
#[cfg(feature = "tesselate")]
fn test_tesselation() {
    let mut application = Application::new();
    let mut layout_engine = DefaultLayoutEngine::default();
    let mut renderer = TesselateRenderer::<()>::default();
    let mapping = CoordsMapping::new(Rect {
        left: 0.0,
        right: 100.0,
        top: 0.0,
        bottom: 100.0,
    });
    application.apply(
        make_widget!(grid_box)
            .props(GridBoxProps {
                cols: 2,
                rows: 2,
                ..Default::default()
            })
            .listed_slot(
                make_widget!(image_box)
                    .props(ImageBoxProps {
                        material: ImageBoxMaterial::Color(ImageBoxColor {
                            color: Color {
                                r: 0.25,
                                g: 0.5,
                                b: 0.75,
                                a: 1.0,
                            },
                            scaling: ImageBoxImageScaling::Frame((10.0, true).into()),
                        }),
                        ..Default::default()
                    })
                    .props(GridBoxItemLayout {
                        space_occupancy: IntRect {
                            left: 1,
                            right: 2,
                            top: 0,
                            bottom: 1,
                        },
                        ..Default::default()
                    }),
            )
            .listed_slot(
                make_widget!(image_box)
                    .props(ImageBoxProps {
                        material: ImageBoxMaterial::Image(ImageBoxImage {
                            id: "ass".to_owned(),
                            ..Default::default()
                        }),
                        content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                            horizontal_alignment: 0.0,
                            vertical_alignment: 0.5,
                        }),
                        ..Default::default()
                    })
                    .props(GridBoxItemLayout {
                        space_occupancy: IntRect {
                            left: 0,
                            right: 1,
                            top: 0,
                            bottom: 1,
                        },
                        ..Default::default()
                    }),
            )
            .listed_slot(
                make_widget!(image_box)
                    .props(ImageBoxProps {
                        material: ImageBoxMaterial::Image(ImageBoxImage {
                            id: "ass".to_owned(),
                            ..Default::default()
                        }),
                        content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                            horizontal_alignment: 1.0,
                            vertical_alignment: 0.5,
                        }),
                        ..Default::default()
                    })
                    .props(GridBoxItemLayout {
                        space_occupancy: IntRect {
                            left: 0,
                            right: 2,
                            top: 1,
                            bottom: 2,
                        },
                        ..Default::default()
                    }),
            )
            .listed_slot(make_widget!(text_box).props(TextBoxProps {
                text: "hello".to_owned(),
                font: TextBoxFont {
                    name: "font".to_owned(),
                    size: 16.0,
                },
                ..Default::default()
            }))
            .into(),
    );
    application.forced_process();
    application
        .layout(&mapping, &mut layout_engine)
        .expect("Failed layouting");
    let tesselation = application
        .render(&mapping, &mut renderer)
        .expect("Cannot tesselate UI tree!")
        .optimized_batches();
    println!("* Tesselation: {:#?}", tesselation);
}
