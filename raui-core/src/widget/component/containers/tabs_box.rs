use crate::{
    pre_hooks,
    props::Props,
    widget,
    widget::{
        component::{
            containers::{
                flex_box::{flex_box, FlexBoxProps},
                switch_box::{switch_box, SwitchBoxProps},
            },
            interactive::{
                button::{button, ButtonNotifyMessage, ButtonNotifyProps},
                navigation::{use_nav_container_active, use_nav_item, NavItemActive},
            },
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::flex::{FlexBoxDirection, FlexBoxItemLayout},
        utils::Transform,
    },
    PropsData, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TabsBoxTabsLocation {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct TabsBoxProps {
    #[serde(default)]
    pub tabs_location: TabsBoxTabsLocation,
    #[serde(default)]
    pub tabs_and_content_separation: Scalar,
    #[serde(default)]
    pub tabs_basis: Option<Scalar>,
    #[serde(default)]
    pub contents_clipping: bool,
    #[serde(default)]
    pub start_index: usize,
    #[serde(default)]
    pub transform: Transform,
}

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct TabsState {
    #[serde(default)]
    pub active_index: usize,
}

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct TabPlateProps {
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub index: usize,
}

impl TabsBoxProps {
    fn to_main_props(&self) -> FlexBoxProps {
        FlexBoxProps {
            direction: match self.tabs_location {
                TabsBoxTabsLocation::Top => FlexBoxDirection::VerticalTopToBottom,
                TabsBoxTabsLocation::Bottom => FlexBoxDirection::VerticalBottomToTop,
                TabsBoxTabsLocation::Left => FlexBoxDirection::HorizontalLeftToRight,
                TabsBoxTabsLocation::Right => FlexBoxDirection::HorizontalRightToLeft,
            },
            separation: self.tabs_and_content_separation,
            wrap: false,
            transform: self.transform.to_owned(),
        }
    }

    fn to_tabs_props(&self) -> FlexBoxProps {
        FlexBoxProps {
            direction: match self.tabs_location {
                TabsBoxTabsLocation::Top => FlexBoxDirection::HorizontalLeftToRight,
                TabsBoxTabsLocation::Bottom => FlexBoxDirection::HorizontalLeftToRight,
                TabsBoxTabsLocation::Left => FlexBoxDirection::VerticalTopToBottom,
                TabsBoxTabsLocation::Right => FlexBoxDirection::VerticalTopToBottom,
            },
            ..Default::default()
        }
    }
}

pub fn use_nav_tabs_box(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        let _ = context.state.write(TabsState {
            active_index: context
                .props
                .map_or_default::<TabsBoxProps, _, _>(|p| p.start_index),
        });
    });

    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if msg.trigger_start() {
                    if let Ok(index) = msg.sender.key().parse::<usize>() {
                        let _ = context.state.write(TabsState {
                            active_index: index,
                        });
                    }
                }
            }
        }
    })
}

#[pre_hooks(use_nav_container_active, use_nav_item, use_nav_tabs_box)]
pub fn nav_tabs_box(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        key,
        props,
        state,
        listed_slots,
        ..
    } = context;

    let main_props = props.read_cloned_or_default::<TabsBoxProps>();
    let props = props.clone().with(main_props.to_main_props());
    let tabs_props = Props::new(main_props.to_tabs_props()).with(FlexBoxItemLayout {
        basis: main_props.tabs_basis,
        grow: 0.0,
        shrink: 0.0,
        ..Default::default()
    });
    let TabsState { active_index } = state.read_cloned_or_default();
    let switch_props = SwitchBoxProps {
        active_index: if active_index < listed_slots.len() {
            Some(active_index)
        } else {
            None
        },
        clipping: main_props.contents_clipping,
        ..Default::default()
    };
    let mut tabs = Vec::with_capacity(listed_slots.len());
    let mut contents = Vec::with_capacity(listed_slots.len());

    for (index, item) in listed_slots.into_iter().enumerate() {
        let [mut tab, content] = item.unpack_tuple();
        tab.remap_props(|props| {
            props.with(TabPlateProps {
                active: active_index == index,
                index,
            })
        });
        let props = Props::new(NavItemActive).with(ButtonNotifyProps(id.to_owned().into()));
        tabs.push(widget! {
            (#{index} button: {props} {
                content = {tab}
            })
        });
        contents.push(content);
    }

    widget! {
        (#{key} flex_box: {props} [
            (#{"tabs"} flex_box: {tabs_props} |[ tabs ]|)
            (#{"contents"} switch_box: {switch_props} |[ contents ]|)
        ])
    }
}
