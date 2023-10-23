use crate::{
    component::containers::wrap_paper::wrap_paper,
    theme::{ThemeColor, ThemedWidgetProps},
};
use raui_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct WindowPaperProps {
    #[serde(default)]
    pub bar_color: ThemeColor,
    #[serde(default = "WindowPaperProps::default_bar_margin")]
    pub bar_margin: Rect,
    #[serde(default = "WindowPaperProps::default_bar_height")]
    pub bar_height: Option<Scalar>,
    #[serde(default = "WindowPaperProps::default_content_margin")]
    pub content_margin: Rect,
}

impl Default for WindowPaperProps {
    fn default() -> Self {
        Self {
            bar_color: ThemeColor::Primary,
            bar_margin: Self::default_bar_margin(),
            bar_height: Self::default_bar_height(),
            content_margin: Self::default_content_margin(),
        }
    }
}

impl WindowPaperProps {
    fn default_bar_margin() -> Rect {
        Rect {
            left: 10.0,
            right: 10.0,
            top: 4.0,
            bottom: 4.0,
        }
    }

    fn default_bar_height() -> Option<Scalar> {
        Some(32.0)
    }

    fn default_content_margin() -> Rect {
        10.0.into()
    }
}

pub fn window_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => {content, bar});

    let window_props = props.read_cloned_or_default::<WindowPaperProps>();

    make_widget!(vertical_box)
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .listed_slot(
            make_widget!(wrap_paper)
                .key("bar")
                .with_props(ThemedWidgetProps {
                    color: window_props.bar_color,
                    ..Default::default()
                })
                .with_props(WrapBoxProps {
                    margin: window_props.bar_margin,
                    fill: true,
                })
                .with_props(FlexBoxItemLayout {
                    basis: window_props.bar_height,
                    grow: 0.0,
                    shrink: 0.0,
                    ..Default::default()
                })
                .named_slot("content", bar),
        )
        .listed_slot(
            make_widget!(wrap_paper)
                .key("content")
                .with_props(WrapBoxProps {
                    margin: window_props.content_margin,
                    fill: true,
                })
                .named_slot("content", content),
        )
        .into()
}

pub fn window_title_controls_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key, named_slots, ..
    } = context;
    unpack_named_slots!(named_slots => {content, title, controls});

    let mut controls = if let WidgetNode::Tuple(nodes) = controls {
        make_widget!(horizontal_box).listed_slots(nodes).into()
    } else {
        controls
    };
    controls.remap_props(|p| {
        if p.has::<FlexBoxItemLayout>() {
            p
        } else {
            p.with(FlexBoxItemLayout {
                grow: 0.0,
                shrink: 0.0,
                ..Default::default()
            })
        }
    });

    make_widget!(window_paper)
        .key(key)
        .named_slot(
            "bar",
            make_widget!(horizontal_box)
                .key("bar")
                .listed_slot(title)
                .listed_slot(controls),
        )
        .named_slot("content", content)
        .into()
}
