use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

fn use_app(ctx: &mut WidgetContext) {
    // whenever we receive tracking message, we store it's horizontal
    // component in state for rendering to use.
    ctx.life_cycle.change(|ctx| {
        for msg in ctx.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<NavTrackingNotifyMessage>() {
                let _ = ctx.state.write_with(msg.state.0.x);
            }
        }
    });
}

#[pre_hooks(use_nav_container_active, use_app)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    // possibly read stored horizontal tracking value.
    let factor = ctx.state.read_cloned::<f32>().unwrap_or(0.5);

    // we use `self_tracking` wrapper widget to allow it to automatically
    // track pointer position relative to itself.
    make_widget!(self_tracking)
        // we tell widget to notify app widget about tracking changes.
        .with_props(NavTrackingNotifyProps(ctx.id.to_owned().into()))
        .named_slot(
            "content",
            // we make horizontal box items have weights proportional to
            // horizontal tracking value.
            make_widget!(horizontal_box)
                .listed_slot(
                    make_widget!(image_box)
                        .with_props(FlexBoxItemLayout {
                            grow: factor,
                            shrink: factor,
                            ..Default::default()
                        })
                        .with_props(ImageBoxProps::colored(Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        })),
                )
                .listed_slot(
                    make_widget!(image_box)
                        .with_props(FlexBoxItemLayout {
                            grow: 1.0 - factor,
                            shrink: 1.0 - factor,
                            ..Default::default()
                        })
                        .with_props(ImageBoxProps::colored(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 1.0,
                            a: 1.0,
                        })),
                ),
        )
        .into()
}

fn main() {
    DeclarativeApp::simple("Tracking", make_widget!(app));
}
