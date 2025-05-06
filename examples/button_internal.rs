use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget, pre_hooks,
    widget::{
        component::{
            image_box::{ImageBoxProps, image_box},
            interactive::{
                button::{ButtonProps, button},
                navigation::{NavItemActive, use_nav_container_active},
            },
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::image::{ImageBoxColor, ImageBoxMaterial, ImageBoxSizeValue},
        utils::Color,
    },
};

// mark the root widget as navigable container to allow button to subscribe to navigation system.
#[pre_hooks(use_nav_container_active)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    // button is the simplest and the most common in use navigable item that can react to user input.
    make_widget!(button)
        // enable button navigation (it is disabled by default).
        .with_props(NavItemActive)
        // by default button state of the button is passed to the content widget with
        // `ButtonProps` props data, so content widget can read it and change its appearance.
        .named_slot("content", make_widget!(internal))
        .into()
}

fn internal(ctx: WidgetContext) -> WidgetNode {
    // first we unpack button state from button props.
    let ButtonProps {
        // selected state means, well..widget has got selected. selection in navigation is more
        // complex than that and it deserves separate deeper explanation, but in essence: whenever
        // user navigate over the UI, RAUI performs selection on navigable items, navigable items
        // may be nested and whenever some widget gets selected, all of its navigable parents
        // receive selection event too, so there is not only one widget that might be selected at
        // a time, but there might be a chain of selected items, as long as they are on the way
        // toward actually selected navigable item in the widget tree.
        selected,
        // trigger state means navigable item got Accept event, which in context of the button
        // means: button is selected and user performed "left mouse button click".
        trigger,
        // context state is similar to trigger state, in this case it means user performed "right
        // mouse button click".
        context,
        ..
    } = ctx.props.read_cloned_or_default();

    let color = if trigger {
        Color {
            r: 1.0,
            g: 0.25,
            b: 0.25,
            a: 1.0,
        }
    } else if context {
        Color {
            r: 0.25,
            g: 1.0,
            b: 0.25,
            a: 1.0,
        }
    } else if selected {
        Color {
            r: 0.25,
            g: 0.25,
            b: 1.0,
            a: 1.0,
        }
    } else {
        Color {
            r: 0.25,
            g: 0.25,
            b: 0.25,
            a: 1.0,
        }
    };

    make_widget!(image_box)
        .with_props(ImageBoxProps {
            material: ImageBoxMaterial::Color(ImageBoxColor {
                color,
                ..Default::default()
            }),
            width: ImageBoxSizeValue::Exact(400.0),
            height: ImageBoxSizeValue::Exact(300.0),
            ..Default::default()
        })
        .into()
}

fn main() {
    DeclarativeApp::simple("Button - Pass state to its child", make_widget!(app));
}
