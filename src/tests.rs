#![cfg(test)]

use crate::prelude::*;
use std::convert::TryInto;

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
                {{WidgetUnit::None}}
                {{WidgetNode::None}}
            ])
        }
    );
}

#[test]
#[allow(dead_code)]
fn test_hello_world() {
    #[derive(Debug, Default, Copy, Clone)]
    struct AppProps {
        pub index: usize,
    }
    implement_props_data!(AppProps);

    #[derive(Debug, Default, Copy, Clone)]
    struct ButtonState {
        pub pressed: bool,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    enum ButtonAction {
        Pressed,
        Released,
    }

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

    widget_component! {
        button(key, props, unmounter, phase, state, messenger, signals) {
            println!("=== PROCESS BUTTON: {} | PHASE: {:?}", key, phase);
            // buttons use string as props data.
            let label = props.read_cloned_or_default::<String>();

            if phase == WidgetPhase::Mount {
                drop(state.write(ButtonState { pressed: false }));
            }
            while let Some(msg) = messenger.read() {
                if let Some(msg) = msg.downcast_ref::<ButtonAction>() {
                    let pressed = match msg {
                        ButtonAction::Pressed => true,
                        ButtonAction::Released => false,
                    };
                    println!("=== BUTTON ACTION: {:?}", msg);
                    drop(state.write(ButtonState { pressed }));
                    drop(signals.write(Box::new(*msg)));
                }
            }

            let k = key.to_string();
            // you use unmounter for storing closures that will be called when widget will be
            // unmounted from the widget tree.
            // closure provides arguments such as:
            // - widget id
            // - widget state
            // - message sender (this one is used to message other widgets you know about)
            // - signal sender (this one is used to message application host)
            unmounter.listen(move |_, _, _, _| {
                println!("=== BUTTON UNMOUNTED: {}", k);
            });

            widget!{
                (#{key} text: {label})
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
        vertical_box(key, listed_slots) {
            // listed slots are just widget node children.
            // here we just unwrap widget units (final atomic UI elements that renderers read).
            let items = listed_slots
                .into_iter()
                .map(|slot| ListBoxItem {
                    slot: slot.try_into().expect("Cannot convert slot to WidgetUnit!"),
                    ..Default::default()
                })
                .collect::<Vec<_>>();

            // we use `{{{ ... }}}` to inform macro that this is widget unit.
            widget! {{{
                ListBox {
                    items,
                    ..Default::default()
                }
            }}}
        }
    }

    widget_component! {
        text(key, props) {
            let text = props.read_cloned_or_default::<String>();

            widget!{{{
                TextBox {
                    text,
                    ..Default::default()
                }
            }}}
        }
    }

    let mut application = Application::new();
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
    println!("=== INPUT:\n{:#?}", tree);

    // some dummy widget tree renderer.
    // it reads widget unit tree and transforms it into target format.
    let mut renderer = HtmlRenderer::default();

    println!("=== PROCESS");
    // `apply()` sets new widget tree.
    application.apply(tree);
    // `render()` calls renderer to perform transformations on processed application widget tree.
    if let Ok(output) = application.render(&mut renderer) {
        println!("=== OUTPUT:\n{}", output);
    }

    println!("=== PROCESS");
    // by default application won't process widget tree if nothing was changed.
    // "change" is either any widget state change, or new message sent to any widget (messages
    // can be sent from application host, for example a mouse click, or from another widget).
    application.forced_process();
    if let Ok(output) = application.render(&mut renderer) {
        println!("=== OUTPUT:\n{}", output);
    }

    let tree = widget! {
        (app)
    };
    println!("=== INPUT:\n{:#?}", tree);
    println!("=== PROCESS");
    application.apply(tree);
    if let Ok(output) = application.render(&mut HtmlRenderer::default()) {
        println!("=== OUTPUT:\n{}", output);
    }
}
