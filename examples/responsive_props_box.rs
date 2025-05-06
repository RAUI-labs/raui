// Make sure you have seen `responsive_box` code example first, because this is an evolution of that.

use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget,
    widget::{
        component::{
            containers::{
                content_box::content_box,
                responsive_box::{
                    MediaQueryExpression, MediaQueryOrientation, responsive_props_box,
                },
            },
            image_box::{ImageBoxProps, image_box},
            text_box::{TextBoxProps, text_box},
        },
        context::WidgetContext,
        node::WidgetNode,
        none_widget,
        unit::text::{TextBoxFont, TextBoxHorizontalAlign, TextBoxVerticalAlign},
        utils::Color,
    },
};

fn widget(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;

    let landscape = props.read_cloned_or_default::<bool>();
    let color = if landscape {
        Color {
            r: 0.25,
            g: 1.0,
            b: 0.25,
            a: 1.0,
        }
    } else {
        Color {
            r: 0.25,
            g: 0.25,
            b: 1.0,
            a: 1.0,
        }
    };
    let text = if landscape {
        "Landscape".to_owned()
    } else {
        "Portrait".to_owned()
    };

    make_widget!(content_box)
        .key(key)
        .listed_slot(
            make_widget!(image_box)
                .key("image")
                .with_props(ImageBoxProps::colored(color)),
        )
        .listed_slot(make_widget!(text_box).with_props(TextBoxProps {
            text,
            font: TextBoxFont {
                name: "./demos/hello-world/resources/verdana.ttf".to_owned(),
                size: 64.0,
            },
            color: Color {
                r: 0.25,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            horizontal_align: TextBoxHorizontalAlign::Center,
            vertical_align: TextBoxVerticalAlign::Middle,
            ..Default::default()
        }))
        .into()
}

fn main() {
    // responsive props box allows to select listed slot with media query, but
    // instead of selecting that slot as content, it only grabs its props and
    // applies them to named `content` slot - this is quite useful if we have
    // single kind of widget we wanna present, but its props are what's different.
    let tree = make_widget!(responsive_props_box)
        .listed_slot(
            // since because slot widget is not used, we need use `none_widget`
            // to not pollute UI with complex widgets that won't be ever used.
            make_widget!(none_widget)
                .with_props(MediaQueryExpression::ScreenOrientation(
                    MediaQueryOrientation::Portrait,
                ))
                .with_props(false),
        )
        .listed_slot(make_widget!(none_widget).with_props(true))
        .named_slot("content", make_widget!(widget));

    DeclarativeApp::simple("Responsive Props Box", tree);
}
