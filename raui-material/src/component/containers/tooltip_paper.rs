use crate::{
    component::containers::{paper::PaperProps, wrap_paper::wrap_paper},
    theme::{ThemeColor, ThemedWidgetProps},
};
use raui_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Copy, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct TooltipPaperProps {
    #[serde(default = "TooltipPaperProps::default_margin")]
    pub margin: Rect,
    #[serde(default = "TooltipPaperProps::default_frame")]
    pub frame: Option<Scalar>,
}

impl Default for TooltipPaperProps {
    fn default() -> Self {
        Self {
            margin: Self::default_margin(),
            frame: Self::default_frame(),
        }
    }
}

impl TooltipPaperProps {
    fn default_margin() -> Rect {
        Rect {
            left: 10.0,
            right: 10.0,
            top: 10.0,
            bottom: 10.0,
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn default_frame() -> Option<Scalar> {
        Some(2.0)
    }
}

pub fn tooltip_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => {content, tooltip});

    let TooltipPaperProps { margin, frame } = props.read_cloned_or_default();

    let size_props = SizeBoxProps {
        width: SizeBoxSizeValue::Content,
        height: SizeBoxSizeValue::Content,
        ..Default::default()
    };
    let themed_props = props.read_cloned_or_else(|| ThemedWidgetProps {
        color: ThemeColor::Primary,
        ..Default::default()
    });
    let paper_props = props.read_cloned_or_else(|| PaperProps {
        frame: frame.map(|v| ImageBoxFrame::from((v, true))),
        ..Default::default()
    });
    let wrap_props = props
        .clone()
        .with(themed_props)
        .with(paper_props)
        .with(WrapBoxProps {
            margin,
            fill: false,
        });

    make_widget!(portals_tooltip_box)
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .named_slot("content", content)
        .named_slot(
            "tooltip",
            make_widget!(size_box)
                .key("size")
                .with_props(size_props)
                .named_slot(
                    "content",
                    make_widget!(wrap_paper)
                        .key("wrap")
                        .merge_props(wrap_props)
                        .named_slot("content", tooltip),
                ),
        )
        .into()
}
