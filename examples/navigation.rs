use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

fn app(_: WidgetContext) -> WidgetNode {
    make_widget!(nav_vertical_box)
        .with_props(VerticalBoxProps {
            override_slots_layout: Some(FlexBoxItemLayout {
                margin: 20.0.into(),
                ..Default::default()
            }),
            ..Default::default()
        })
        .listed_slot(make_widget!(button_item))
        .listed_slot(make_widget!(button_item))
        .listed_slot(make_widget!(button_item))
        .listed_slot(make_widget!(button_item))
        .into()
}

fn button_item(_: WidgetContext) -> WidgetNode {
    make_widget!(button)
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
        .with_props(ImageBoxProps::colored(color))
        .into()
}

fn main() {
    App::new(AppConfig::default().title("Navigation")).run(
        DeclarativeApp::default()
            .tree(make_widget!(app))
            .setup_interactions(|interactions| {
                interactions.engine.deselect_when_no_button_found = false;
            }),
    );
}
