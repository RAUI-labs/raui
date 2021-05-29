use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;

fn main() {
    // Create our quick start builder
    RauiQuickStartBuilder::default()
        // Set our window title
        .window_title("RAUI Guide".into())
        // Set the RAUI widget tree for our app
        .widget_tree(widget! {
            // Our app is the only widget we insert here. All other widgets
            // will go inside of our app
            (app)
        })
        // Build the app
        .build()
        .expect("Error building quick start")
        // And run it! 🚀
        .run()
        .expect("Error running RAUI app");
}

// Our app widget from earlier
pub fn app(_ctx: WidgetContext) -> WidgetNode {
    // Create our text box properties
    let text_box_props = Props::new(TextBoxProps {
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
        // Use the defaults for the rest of the text box settings
        ..Default::default()
    })
    // Notice that we now use a `FlexBoxItemLayout` instead of a `ContentBoxItemLayout`
    // because we are putting it in a flex box instead of a content box
    .with(FlexBoxItemLayout {
        margin: Rect {
            // Let's just set a left margin this time
            left: 30.,
            ..Default::default()
        },
        ..Default::default()
    });

    // Create the props for our image
    let image_box_props = Props::new(ImageBoxProps {
        // The material defines what image or color to use for the box
        material: ImageBoxMaterial::Image(ImageBoxImage {
            // The path to our image
            id: "resources/cats.jpg".to_owned(),
            ..Default::default()
        }),
        ..Default::default()
    });

    widget! {
        // Use a vertical_box instead of a content_box
        (vertical_box [
            // Now because the text and image won't overlap, let's put
            // the text above the image
            (text_box: {text_box_props})
            (image_box: {image_box_props})
        ])
    }
}
