use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget, pre_hooks,
    widget::{
        component::{
            containers::tabs_box::{
                TabPlateProps, TabsBoxProps, TabsBoxTabsLocation, nav_tabs_box,
            },
            image_box::{ImageBoxProps, image_box},
            interactive::navigation::{NavItemActive, use_nav_container_active},
        },
        context::WidgetContext,
        node::WidgetNode,
        utils::Color,
    },
};

#[pre_hooks(use_nav_container_active)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    make_widget!(nav_tabs_box)
        .with_props(NavItemActive)
        .with_props(TabsBoxProps {
            // top tabs location is default one but we can change tabs bar to be on either side of
            // the tabs box area.
            tabs_location: TabsBoxTabsLocation::Top,
            // we set tabs basis to let tabs itself fill into the area that tabs bar gives to layout.
            tabs_basis: Some(50.0),
            ..Default::default()
        })
        // we pack pairs of tab plate and its content using tuples and then put them in listed slots.
        .listed_slot(WidgetNode::pack_tuple([
            // first tiple item is always the tab plate that's gonna be put on tabs bar (it's gonna
            // be wrapped with button component so it's better to not put other buttons in tab plate
            // widget tree).
            make_widget!(tab_plate)
                .with_props(Color {
                    r: 1.0,
                    g: 0.25,
                    b: 0.25,
                    a: 1.0,
                })
                .into(),
            // second tuple item is always the tab contents (all tabs contents are put into inner
            // switch box so we make sure there is always only one tab content present at a time).
            make_widget!(image_box)
                .with_props(ImageBoxProps::colored(Color {
                    r: 0.75,
                    g: 0.25,
                    b: 0.25,
                    a: 1.0,
                }))
                .into(),
        ]))
        .listed_slot(WidgetNode::pack_tuple([
            make_widget!(tab_plate)
                .with_props(Color {
                    r: 0.25,
                    g: 1.0,
                    b: 0.25,
                    a: 1.0,
                })
                .into(),
            make_widget!(image_box)
                .with_props(ImageBoxProps::colored(Color {
                    r: 0.25,
                    g: 0.75,
                    b: 0.25,
                    a: 1.0,
                }))
                .into(),
        ]))
        .listed_slot(WidgetNode::pack_tuple([
            make_widget!(tab_plate)
                .with_props(Color {
                    r: 0.25,
                    g: 0.25,
                    b: 1.0,
                    a: 1.0,
                })
                .into(),
            make_widget!(image_box)
                .with_props(ImageBoxProps::colored(Color {
                    r: 0.25,
                    g: 0.25,
                    b: 0.75,
                    a: 1.0,
                }))
                .into(),
        ]))
        .into()
}

fn tab_plate(ctx: WidgetContext) -> WidgetNode {
    let mut color = ctx.props.read_cloned_or_default::<Color>();
    if !ctx.props.read_cloned_or_default::<TabPlateProps>().active {
        color.r *= 0.5;
        color.g *= 0.5;
        color.b *= 0.5;
    }

    make_widget!(image_box)
        .with_props(ImageBoxProps::colored(color))
        .into()
}

fn main() {
    DeclarativeApp::simple("Tabs Box", make_widget!(app));
}
