use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

fn main() {
    // responsive box allows to select one of listed slot widgets to present,
    // depending on which slot widget's media query expression passes.
    // ordering of listed slots is important, the first one that passes will be used.
    // media query expressions can be combined with logical operator expressions
    // such as `and`, `or` and `not`. in case of default case, use `any` expression.
    let tree =
        make_widget!(responsive_box)
            .listed_slot(
                make_widget!(image_box)
                    .key("landscape")
                    .with_props(MediaQueryExpression::WidgetOrientation(
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
            ));

    DeclarativeApp::simple("Responsive Box", tree);
}
