use raui_core::prelude::*;

widget_component! {
    pub app(key, named_slots) {
        unpack_named_slots!(named_slots => { title, content });

        content.remap_props(|props| props.with(FlexBoxItemLayout {
            fill: 1.0,
            grow: 1.0,
            ..Default::default()
        }));

        widget!{
            (#{key} vertical_box: {VerticalBoxProps {
                separation: 16.0,
                ..Default::default()
            }} [
                {title}
                {content}
            ])
        }
    }
}
