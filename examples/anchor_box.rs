use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;

fn preview(ctx: WidgetContext) -> WidgetNode {
    // we print this widget props to show how AnchorProps values change relative to window resize.
    println!("Preview props: {:#?}", ctx.props);

    // we create simple colored image that fills available space just to make you see the values.
    make_widget!(image_box)
        .with_props(ImageBoxProps::colored(Color {
            r: 1.0,
            g: 0.25,
            b: 0.25,
            a: 1.0,
        }))
        .into()
}

fn main() {
    // we create widget reference first so we can apply it to some widget and and reference
    //that widget in another place - basically what widget reference is, it is a way to read
    // some other widget ID in some other place outside the referenced widget scope.
    let idref = WidgetRef::new();

    let tree = make_widget!(content_box)
        // we apply widget reference to the root content box so we can reference that root widget
        // later in anchor box to enable it to calculate how anchor box content is lay out relative
        // to the root widget - this is the most important thing to setup, because if we won't do
        // that, anchor box would not be able to give its content a proper data about its layout
        // relative to the referenced widget. Note that, you can reference ANY widget in the widget
        // tree - it will always give you a relative location to any widget you provide.
        .idref(idref.clone())
        .listed_slot(
            make_widget!(anchor_box)
                // we pass widget reference to anchor box via RelativeLayoutProps, because anchor
                // uses relative layout hook to perform calculations of relative layout box.
                .with_props(RelativeLayoutProps {
                    relative_to: idref.into(),
                })
                // we apply margin to anchor box just to make it not fill entire space by default.
                .with_props(ContentBoxItemLayout {
                    margin: 100.0.into(),
                    ..Default::default()
                })
                .named_slot("content", make_widget!(preview)),
        );

    RauiQuickStartBuilder::default()
        .window_title("Anchor Box".to_owned())
        .widget_tree(tree.into())
        .build()
        .unwrap()
        .run()
        .unwrap();
}
