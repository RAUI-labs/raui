use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget,
    widget::{
        component::{
            containers::{
                content_box::content_box,
                responsive_box::{MediaQueryExpression, MediaQueryOrientation, responsive_box},
            },
            image_box::{ImageBoxProps, image_box},
            text_box::{TextBoxProps, text_box},
        },
        unit::text::{TextBoxFont, TextBoxHorizontalAlign, TextBoxVerticalAlign},
        utils::Color,
    },
};

fn main() {
    let tree = make_widget!(content_box)
        .listed_slot(
            // responsive box allows to select one of listed slot widgets to
            // present, depending on which slot widget's media query expression
            // passes. ordering of listed slots is important, the first one that
            // passes will be used. media query expressions can be combined with
            // logical operator expressions such as `and`, `or` and `not`.
            // in case of default case, use `any` expression.
            make_widget!(responsive_box)
                .listed_slot(
                    make_widget!(image_box)
                        .key("landscape")
                        .with_props(MediaQueryExpression::ScreenOrientation(
                            MediaQueryOrientation::Landscape,
                        ))
                        .with_props(ImageBoxProps::colored(Color {
                            r: 0.25,
                            g: 1.0,
                            b: 0.25,
                            a: 1.0,
                        })),
                )
                .listed_slot(make_widget!(image_box).key("portrait").with_props(
                    ImageBoxProps::colored(Color {
                        r: 0.25,
                        g: 0.25,
                        b: 1.0,
                        a: 1.0,
                    }),
                )),
        )
        .listed_slot(make_widget!(text_box).with_props(TextBoxProps {
            text: "Change window size to observe responsiveness".to_owned(),
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
        }));

    DeclarativeApp::simple("Responsive Box", tree);
}
