// Make sure you have seen `context_box` code example first, because this is an evolution of that.

use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

// we mark app as an active navigable container to let all buttons down the tree register to the
// navigation system so they can react on mouse hovering for example.
#[pre_hooks(use_nav_container_active)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    let idref = WidgetRef::default();

    make_widget!(content_box)
        .idref(idref.clone())
        .with_shared_props(PortalsContainer(idref.into()))
        .listed_slot(
            make_widget!(horizontal_box)
                .with_props(HorizontalBoxProps {
                    separation: 25.0,
                    ..Default::default()
                })
                .listed_slot(
                    make_widget!(icon)
                        .with_props(FlexBoxItemLayout::cleared())
                        .with_props(Color {
                            r: 1.0,
                            g: 0.25,
                            b: 0.25,
                            a: 1.0,
                        }),
                )
                .listed_slot(
                    make_widget!(icon)
                        .with_props(FlexBoxItemLayout::cleared())
                        .with_props(Color {
                            r: 0.25,
                            g: 1.0,
                            b: 0.25,
                            a: 1.0,
                        })
                        .with_props(PivotBoxProps {
                            pivot: Vec2 { x: 0.5, y: 1.0 },
                            align: Vec2 { x: 0.5, y: 0.0 },
                        }),
                )
                .listed_slot(
                    make_widget!(icon)
                        .with_props(FlexBoxItemLayout::cleared())
                        .with_props(Color {
                            r: 0.25,
                            g: 0.25,
                            b: 1.0,
                            a: 1.0,
                        })
                        .with_props(PivotBoxProps {
                            pivot: Vec2 { x: 1.0, y: 1.0 },
                            align: Vec2 { x: 1.0, y: 0.0 },
                        }),
                ),
        )
        .into()
}

fn icon(ctx: WidgetContext) -> WidgetNode {
    // tooltip box is basically an evolution of context box - what changes is tooltip box is shown
    // only if this its content gets selected by navigation system (and since buttons can be
    // selected for example by mouse hover, this tooltip is shown whenever mouse gets over the
    // widget it wraps).
    make_widget!(portals_tooltip_box)
        .with_props(ctx.props.read_cloned_or_default::<PivotBoxProps>())
        // put colored image box as content widget.
        .named_slot(
            "content",
            // we wrap content with button to allow automated widget selection that will show tooltip,
            make_widget!(button)
                // remember that buttons has to be activated to make them receive selection
                // navigation messages - they are inactive by default.
                .with_props(NavItemActive)
                .named_slot(
                    "content",
                    make_widget!(image_box).with_props(ImageBoxProps {
                        material: ImageBoxMaterial::Color(ImageBoxColor {
                            color: ctx.props.read_cloned_or_default::<Color>(),
                            ..Default::default()
                        }),
                        width: ImageBoxSizeValue::Exact(100.0),
                        height: ImageBoxSizeValue::Exact(100.0),
                        ..Default::default()
                    }),
                ),
        )
        // put gray image box as tooltip widget.
        .named_slot(
            "tooltip",
            make_widget!(image_box).with_props(ImageBoxProps {
                material: ImageBoxMaterial::Color(ImageBoxColor {
                    color: Color {
                        r: 0.25,
                        g: 0.25,
                        b: 0.25,
                        a: 1.0,
                    },
                    ..Default::default()
                }),
                width: ImageBoxSizeValue::Exact(150.0),
                height: ImageBoxSizeValue::Exact(50.0),
                ..Default::default()
            }),
        )
        .into()
}

fn main() {
    DeclarativeApp::simple("Tooltip Box", make_widget!(app));
}
