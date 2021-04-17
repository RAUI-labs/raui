use raui_core::prelude::*;

widget_component!(
    pub fn app(key: Key, (title, content): NamedSlots) {
        dbg!(&title);
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
);
