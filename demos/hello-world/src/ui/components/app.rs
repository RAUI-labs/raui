use raui::prelude::*;

pub fn app(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key, named_slots, ..
    } = context;
    unpack_named_slots!(named_slots => { title, content });

    title.remap_props(|props| {
        props.with(FlexBoxItemLayout {
            grow: 0.0,
            shrink: 0.0,
            ..Default::default()
        })
    });
    let props = Props::new(VerticalBoxProps {
        separation: 16.0,
        ..Default::default()
    })
    .with(NavJumpLooped);

    widget! {
        (#{key} nav_vertical_box: {props} [
            {title}
            {content}
        ])
    }
}
