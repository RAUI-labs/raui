// Make sure you have seen `anchor_box` code example first, because this is an evolution of that.

use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;

// we use this hook that receives anchor box state change and store that in this component state.
#[pre_hooks(use_anchor_box_notified_state)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    let idref = WidgetRef::new();

    make_widget!(content_box)
        .idref(idref.clone())
        // widget rederence marked as portals container and put into root shared props for any
        // portal box down the widget tree. More about how portal box works later.
        .with_shared_props(PortalsContainer(idref.to_owned().into()))
        .listed_slot(
            make_widget!(anchor_box)
                .with_props(RelativeLayoutProps {
                    relative_to: idref.to_owned().into(),
                })
                // we make this anchor box notify this component about anchor box state change.
                .with_props(AnchorNotifyProps(ctx.id.to_owned().into()))
                .with_props(ContentBoxItemLayout {
                    margin: 100.0.into(),
                    ..Default::default()
                })
                .named_slot(
                    "content",
                    make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                        r: 0.25,
                        g: 0.25,
                        b: 0.25,
                        a: 1.0,
                    })),
                ),
        )
        .listed_slot(
            // pivot box is used to calculate ContentBoxItemLayout that is later passed to its
            // content so it works best with things like portal box which then uses that layout to
            // position its content in portals container - in other words pivot box and portal box
            // works best together.
            make_widget!(pivot_box)
                // pivot box uses AnchorProps for PivotBoxProps data to calculate a place to
                // position the content relative to that area.
                .with_props(ctx.state.read_cloned_or_default::<AnchorProps>())
                .with_props(PivotBoxProps {
                    // percentage of the anchored area to position at.
                    pivot: 0.0.into(),
                    // percentage of content area to align relative to pivot position.
                    align: 0.75.into(),
                })
                .named_slot(
                    "content",
                    // portal box reads PortalsContainer from shared props and use its widget
                    // reference to "teleport" portal box content into referenced container widget
                    // (best container to use is content box) - what actually happen, RAUI sees
                    // portal box, unwraps it, find referenced container and injects that unwrapped
                    // content widget there.
                    make_widget!(portal_box).named_slot(
                        "content",
                        make_widget!(image_box).with_props(ImageBoxProps {
                            material: ImageBoxMaterial::Color(ImageBoxColor {
                                color: Color {
                                    r: 1.0,
                                    g: 0.25,
                                    b: 0.25,
                                    a: 1.0,
                                },
                                ..Default::default()
                            }),
                            width: ImageBoxSizeValue::Exact(100.0),
                            height: ImageBoxSizeValue::Exact(100.0),
                            ..Default::default()
                        }),
                    ),
                ),
        )
        .listed_slot(
            make_widget!(pivot_box)
                .with_props(ctx.state.read_cloned_or_default::<AnchorProps>())
                .with_props(PivotBoxProps {
                    pivot: 0.5.into(),
                    align: 0.5.into(),
                })
                .named_slot(
                    "content",
                    make_widget!(portal_box).named_slot(
                        "content",
                        make_widget!(image_box).with_props(ImageBoxProps {
                            material: ImageBoxMaterial::Color(ImageBoxColor {
                                color: Color {
                                    r: 0.25,
                                    g: 1.0,
                                    b: 0.25,
                                    a: 1.0,
                                },
                                ..Default::default()
                            }),
                            width: ImageBoxSizeValue::Exact(200.0),
                            height: ImageBoxSizeValue::Exact(200.0),
                            ..Default::default()
                        }),
                    ),
                ),
        )
        .listed_slot(
            make_widget!(pivot_box)
                .with_props(ctx.state.read_cloned_or_default::<AnchorProps>())
                .with_props(PivotBoxProps {
                    pivot: 1.0.into(),
                    align: 0.25.into(),
                })
                .named_slot(
                    "content",
                    make_widget!(portal_box).named_slot(
                        "content",
                        make_widget!(image_box).with_props(ImageBoxProps {
                            material: ImageBoxMaterial::Color(ImageBoxColor {
                                color: Color {
                                    r: 0.25,
                                    g: 0.25,
                                    b: 1.0,
                                    a: 1.0,
                                },
                                ..Default::default()
                            }),
                            width: ImageBoxSizeValue::Exact(100.0),
                            height: ImageBoxSizeValue::Exact(100.0),
                            ..Default::default()
                        }),
                    ),
                ),
        )
        .into()
}

fn main() {
    RauiQuickStartBuilder::default()
        .window_title("Portal Box".to_owned())
        .widget_tree(make_widget!(app).into())
        .build()
        .unwrap()
        .run()
        .unwrap();
}
