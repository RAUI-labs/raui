use raui::prelude::*;

fn main() {
    DeclarativeApp::simple("RAUI Guide", make_widget!(app));
}

// Our app widget from earlier
pub fn app(_ctx: WidgetContext) -> WidgetNode {
    make_widget!(vertical_box)
        .listed_slot(
            make_widget!(text_box)
                .with_props(TextBoxProps {
                    text: "Hello World!".into(),
                    color: Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                    font: TextBoxFont {
                        // We specify the path to our font
                        name: "resources/verdana.ttf".to_owned(),
                        size: 60.0,
                    },
                    horizontal_align: TextBoxHorizontalAlign::Center,
                    // Use the defaults for the rest of the text box settings
                    ..Default::default()
                })
                // Notice that we now use a `FlexBoxItemLayout` instead of a `ContentBoxItemLayout`
                // because we are putting it in a flex box instead of a content box
                .with_props(FlexBoxItemLayout {
                    basis: Some(80.0),
                    grow: 0.0,
                    shrink: 0.0,
                    margin: Rect {
                        // Let's just set a left margin this time
                        left: 30.,
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
        .listed_slot(make_widget!(image_box).with_props(ImageBoxProps {
            // The material defines what image or color to use for the box
            material: ImageBoxMaterial::Image(ImageBoxImage {
                // The path to our image
                id: "resources/cats.jpg".to_owned(),
                ..Default::default()
            }),
            // this allows image content to not stretch to fill its container.
            content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                horizontal_alignment: 0.5,
                vertical_alignment: 0.5,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .into()
}
