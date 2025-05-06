// Make sure you have seen `button_internal` code example first, because this is an evolution of that.

use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget, pre_hooks,
    widget::{
        component::{
            image_box::{ImageBoxProps, image_box},
            interactive::{
                button::{ButtonNotifyMessage, ButtonNotifyProps, button},
                navigation::{NavItemActive, use_nav_container_active},
            },
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::image::{ImageBoxColor, ImageBoxMaterial, ImageBoxSizeValue},
        utils::Color,
    },
};

// we create app hook that just receives button state change messages and prints them.
fn use_app(ctx: &mut WidgetContext) {
    ctx.life_cycle.change(|ctx| {
        for msg in ctx.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                println!("Button message: {:#?}", msg);
            }
        }
    });
}

#[pre_hooks(use_nav_container_active, use_app)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    make_widget!(button)
        .with_props(NavItemActive)
        // we tell button to notify this component (send messages to it) whenever button state changes.
        .with_props(ButtonNotifyProps(ctx.id.to_owned().into()))
        .named_slot(
            "content",
            make_widget!(image_box).with_props(ImageBoxProps {
                material: ImageBoxMaterial::Color(ImageBoxColor {
                    color: Color {
                        r: 1.0,
                        g: 0.25,
                        b: 0.25,
                        a: 1.0,
                    },
                    ..Default::default()
                }),
                width: ImageBoxSizeValue::Exact(400.0),
                height: ImageBoxSizeValue::Exact(300.0),
                ..Default::default()
            }),
        )
        .into()
}

fn main() {
    DeclarativeApp::simple("Button - Sending state to other widget", make_widget!(app));
}
