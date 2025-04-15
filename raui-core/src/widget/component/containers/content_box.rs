//! A generic container for content with optional clipping and transforms

use crate::{
    PropsData, make_widget, pre_hooks,
    widget::{
        component::interactive::navigation::{
            NavContainerActive, NavItemActive, NavJumpActive, use_nav_container_active,
            use_nav_item, use_nav_jump_direction_active,
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::content::{ContentBoxItemLayout, ContentBoxItemNode, ContentBoxNode},
        utils::Transform,
    },
};
use serde::{Deserialize, Serialize};

/// The properties of a [`content_box`] component
#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct ContentBoxProps {
    /// Whether or not to clip the parts of items that overflow outside of the box bounds
    #[serde(default)]
    pub clipping: bool,
    /// The transform to apply to the box and it's contents
    #[serde(default)]
    pub transform: Transform,
}

#[pre_hooks(use_nav_container_active, use_nav_jump_direction_active, use_nav_item)]
pub fn nav_content_box(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        listed_slots,
        ..
    } = context;

    let props = props
        .clone()
        .without::<NavContainerActive>()
        .without::<NavJumpActive>()
        .without::<NavItemActive>();

    make_widget!(content_box)
        .key(key)
        .merge_props(props)
        .listed_slots(listed_slots)
        .into()
}

/// A generic container for other widgets
///
/// [`content_box`]'s serve two basic purposes: allowing you to apply transformations and clipping
/// to all contained widgets and giving contained widgets more control over their layout inside of
/// the box.
///
/// # Transform & Clipping
///
/// The transformation and clipping options on the [`content_box`] can be set by setting the
/// [`ContentBoxProps`] on the component.
///
/// # Child Widget Layout
///
/// With a [`content_box`] you can get more control over the layout of it's children by adding the
/// [`ContentBoxItemLayout`] properties to any of it's children.
pub fn content_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        listed_slots,
        ..
    } = context;

    let ContentBoxProps {
        clipping,
        transform,
    } = props.read_cloned_or_default();

    let items = listed_slots
        .into_iter()
        .filter_map(|slot| {
            if let Some(props) = slot.props() {
                let layout = props.read_cloned_or_default::<ContentBoxItemLayout>();
                Some(ContentBoxItemNode { slot, layout })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    ContentBoxNode {
        id: id.to_owned(),
        props: props.clone(),
        items,
        clipping,
        transform,
    }
    .into()
}
