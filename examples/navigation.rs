use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

#[pre_hooks(use_nav_container_active, use_nav_jump_direction_active)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    let slots_layout = FlexBoxItemLayout {
        margin: 20.0.into(),
        ..Default::default()
    };

    make_widget!(vertical_box)
        .key("vertical")
        .with_props(VerticalBoxProps {
            override_slots_layout: Some(slots_layout.clone()),
            ..Default::default()
        })
        .listed_slot(
            make_widget!(horizontal_box)
                .key("horizontal")
                .with_props(HorizontalBoxProps {
                    override_slots_layout: Some(slots_layout),
                    ..Default::default()
                })
                .listed_slot(make_widget!(button_item).key("a").with_props(NavAutoSelect))
                .listed_slot(make_widget!(button_item).key("b"))
                .listed_slot(make_widget!(button_item).key("c")),
        )
        .listed_slot(make_widget!(button_item).key("d"))
        .listed_slot(make_widget!(button_item).key("e"))
        .into()
}

fn button_item(ctx: WidgetContext) -> WidgetNode {
    make_widget!(button)
        .key("button")
        .merge_props(ctx.props.clone())
        .with_props(NavItemActive)
        .named_slot("content", make_widget!(button_content))
        .into()
}

fn button_content(ctx: WidgetContext) -> WidgetNode {
    let ButtonProps {
        selected,
        trigger,
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
        .key("image")
        .with_props(ImageBoxProps::colored(color))
        .into()
}

fn main() {
    App::new(AppConfig::default().title("Navigation")).run(
        DeclarativeApp::default()
            .tree(make_widget!(app).key("app"))
            .setup_interactions(|interactions| {
                interactions.engine.deselect_when_no_button_found = false;
            }),
    );
}
