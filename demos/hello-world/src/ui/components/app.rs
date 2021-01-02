use raui_core::prelude::*;

widget_component! {
    pub app(key, named_slots) {
        unpack_named_slots!(named_slots => { title, content });

        title.remap_props(|props| props.with(FlexBoxItemLayout {
            grow: 0.0,
            shrink: 0.0,
            ..Default::default()
        }));
        let props = VerticalBoxProps {
            separation: 16.0,
            ..Default::default()
        };

        widget!{
            (#{key} vertical_box: {props} [
                {title}
                {content}
            ])
        }
    }
}
