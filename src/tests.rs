#![cfg(test)]

use crate::prelude::*;
use std::str::FromStr;

#[test]
fn test_app_threadsafe() {
    fn foo<T>()
    where
        T: Send + Sync,
    {
        println!("* {} is threadsafe!", std::any::type_name::<T>());
    }

    foo::<Application>();
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
    // mounted/changed/unmounted. they exists for you to be able to resuse some common logic across
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
                    if let Some(msg) = msg.downcast_ref::<ButtonAction>() {
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
                (#{key} text: {props})
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
