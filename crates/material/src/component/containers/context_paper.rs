use crate::{
    component::containers::{paper::PaperProps, wrap_paper::wrap_paper},
    theme::{ThemeColor, ThemedWidgetProps},
};
use raui_core::{
    PropsData, Scalar, make_widget, unpack_named_slots,
    widget::{
        WidgetIdOrRef,
        component::{
            containers::{
                context_box::portals_context_box,
                size_box::{SizeBoxProps, size_box},
                wrap_box::WrapBoxProps,
            },
            interactive::button::{ButtonNotifyProps, button},
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::{image::ImageBoxFrame, size::SizeBoxSizeValue},
        utils::Rect,
    },
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct ContextPaperProps {
    #[serde(default = "ContextPaperProps::default_margin")]
    pub margin: Rect,
    #[serde(default = "ContextPaperProps::default_frame")]
    pub frame: Option<Scalar>,
    #[serde(default)]
    pub notify_backdrop_accept: WidgetIdOrRef,
}

impl Default for ContextPaperProps {
    fn default() -> Self {
        Self {
            margin: Self::default_margin(),
            frame: Self::default_frame(),
            notify_backdrop_accept: Default::default(),
        }
    }
}

impl ContextPaperProps {
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

pub fn context_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => {content, context});

    let ContextPaperProps {
        margin,
        frame,
        notify_backdrop_accept,
    } = props.read_cloned_or_default();

    let context_size_props = SizeBoxProps {
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
    let backdrop_size_props = SizeBoxProps {
        width: SizeBoxSizeValue::Fill,
        height: SizeBoxSizeValue::Fill,
        ..Default::default()
    };

    make_widget!(portals_context_box)
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .named_slot("content", content)
        .named_slot(
            "context",
            make_widget!(size_box)
                .key("size")
                .with_props(context_size_props)
                .named_slot(
                    "content",
                    make_widget!(wrap_paper)
                        .key("wrap")
                        .merge_props(wrap_props)
                        .named_slot("content", context),
                ),
        )
        .named_slot(
            "backdrop",
            make_widget!(button)
                .key("button")
                .with_props(ButtonNotifyProps(notify_backdrop_accept))
                .named_slot(
                    "content",
                    make_widget!(size_box)
                        .key("size")
                        .with_props(backdrop_size_props),
                ),
        )
        .into()
}
