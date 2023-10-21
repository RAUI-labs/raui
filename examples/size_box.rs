use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

fn main() {
    let tree = make_widget!(size_box)
        .with_props(SizeBoxProps {
            // takes the layout box size from its children size. content size is the default one.
            width: SizeBoxSizeValue::Content,
            height: SizeBoxSizeValue::Content,
            ..Default::default()
        })
        .named_slot(
            "content",
            make_widget!(size_box)
                .with_props(SizeBoxProps {
                    // exact size resets layout available size into size defined here.
                    // it simply ignores available size and uses this one down the widget tree.
                    width: SizeBoxSizeValue::Exact(400.0),
                    height: SizeBoxSizeValue::Exact(300.0),
                    ..Default::default()
                })
                .named_slot(
                    "content",
                    make_widget!(size_box)
                        .with_props(SizeBoxProps {
                            // uses layout available size defined by this widget parent node.
                            width: SizeBoxSizeValue::Fill,
                            height: SizeBoxSizeValue::Fill,
                            // we can additionally set margin.
                            margin: 50.0.into(),
                            ..Default::default()
                        })
                        .named_slot(
                            "content",
                            make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                                r: 1.0,
                                g: 0.25,
                                b: 0.25,
                                a: 1.0,
                            })),
                        ),
                ),
        );

    DeclarativeApp::simple("Size Box", tree);
}
