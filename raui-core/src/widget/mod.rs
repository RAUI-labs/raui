pub mod component;
pub mod context;
pub mod node;
pub mod unit;
pub mod utils;

use crate::{
    application::Application,
    widget::{
        context::{WidgetContext, WidgetMountOrChangeContext, WidgetUnmountContext},
        node::WidgetNode,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    convert::TryFrom,
    hash::{Hash, Hasher},
    ops::Deref,
    str::FromStr,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetIdDef(pub String);

impl From<WidgetId> for WidgetIdDef {
    fn from(data: WidgetId) -> Self {
        Self(data.to_string())
    }
}

#[derive(Default, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(try_from = "WidgetIdDef")]
#[serde(into = "WidgetIdDef")]
pub struct WidgetId {
    id: String,
    type_name_len: u8,
    key_len: u8,
    depth: usize,
}

impl WidgetId {
    pub fn new(type_name: String, path: Vec<String>) -> Self {
        if type_name.len() >= 256 {
            panic!(
                "WidgetId `type_name` (\"{}\") cannot be longer than 255 characters!",
                type_name
            );
        }
        let type_name_len = type_name.len() as u8;
        let key_len = path.last().map(|p| p.len()).unwrap_or_default() as u8;
        let depth = path.len();
        let count = type_name.len() + 1 + path.iter().map(|part| 1 + part.len()).sum::<usize>();
        let mut id = String::with_capacity(count);
        id.push_str(&type_name);
        id.push(':');
        for (i, part) in path.into_iter().enumerate() {
            if part.len() >= 256 {
                panic!(
                    "WidgetId `path[{}]` (\"{}\") cannot be longer than 255 characters!",
                    i, part
                );
            }
            id.push('/');
            id.push_str(&part);
        }
        Self {
            id,
            type_name_len,
            key_len,
            depth,
        }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.id.is_empty()
    }

    #[inline]
    pub fn depth(&self) -> usize {
        self.depth
    }

    #[inline]
    pub fn type_name(&self) -> &str {
        &self.id.as_str()[0..self.type_name_len as usize]
    }

    #[inline]
    pub fn path(&self) -> &str {
        &self.id.as_str()[(self.type_name_len as usize + 2)..]
    }

    #[inline]
    pub fn key(&self) -> &str {
        &self.id[(self.id.len() - self.key_len as usize)..]
    }

    #[inline]
    pub fn parts(&self) -> impl Iterator<Item = &str> {
        self.id[(self.type_name_len as usize + 2)..].split('/')
    }

    pub fn hashed_value(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn common_parts<'a>(a: &'a Self, b: &'a Self) -> impl Iterator<Item = &'a str> {
        a.parts()
            .zip(b.parts())
            .take_while(|(a, b)| a == b)
            .map(|(a, _)| a)
    }

    pub fn common_path(a: &Self, b: &Self) -> String {
        let mut result = String::with_capacity(a.path().len().max(b.path().len()));
        for (a, b) in a.parts().zip(b.parts()) {
            if a != b {
                break;
            }
            result.push('/');
            result.push_str(a);
        }
        result
    }
}

impl Deref for WidgetId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl AsRef<str> for WidgetId {
    fn as_ref(&self) -> &str {
        &self.id
    }
}

impl FromStr for WidgetId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(index) = s.find(':') {
            let type_name = s[..index].to_owned();
            let rest = &s[(index + 2)..];
            let path = rest.split('/').map(|p| p.to_owned()).collect::<Vec<_>>();
            Ok(Self::new(type_name, path))
        } else {
            Err(())
        }
    }
}

impl std::fmt::Debug for WidgetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl TryFrom<WidgetIdDef> for WidgetId {
    type Error = String;

    fn try_from(id: WidgetIdDef) -> Result<Self, Self::Error> {
        match Self::from_str(&id.0) {
            Ok(id) => Ok(id),
            Err(_) => Err(format!("Could not parse id: `{}`", id.0)),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WidgetRefDef(pub Option<WidgetId>);

impl From<WidgetRef> for WidgetRefDef {
    fn from(data: WidgetRef) -> Self {
        match data.0.read() {
            Ok(data) => Self(data.clone()),
            Err(_) => Default::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(from = "WidgetRefDef")]
#[serde(into = "WidgetRefDef")]
pub struct WidgetRef(#[serde(skip)] Arc<RwLock<Option<WidgetId>>>);

impl WidgetRef {
    pub(crate) fn write(&mut self, id: WidgetId) {
        if let Ok(mut data) = self.0.write() {
            *data = Some(id);
        }
    }

    pub fn read(&self) -> Option<WidgetId> {
        if let Ok(data) = self.0.read() {
            data.clone()
        } else {
            None
        }
    }
}

impl From<WidgetRefDef> for WidgetRef {
    fn from(data: WidgetRefDef) -> Self {
        WidgetRef(Arc::new(RwLock::new(data.0)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetIdOrRef {
    None,
    Id(WidgetId),
    Ref(WidgetRef),
}

impl WidgetIdOrRef {
    #[inline]
    pub fn new_ref() -> Self {
        Self::Ref(WidgetRef::default())
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    #[inline]
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn read(&self) -> Option<WidgetId> {
        match self {
            Self::None => None,
            Self::Id(id) => Some(id.to_owned()),
            Self::Ref(idref) => idref.read(),
        }
    }
}

impl Default for WidgetIdOrRef {
    fn default() -> Self {
        Self::None
    }
}

impl From<()> for WidgetIdOrRef {
    fn from(_: ()) -> Self {
        Self::None
    }
}

impl From<WidgetId> for WidgetIdOrRef {
    fn from(v: WidgetId) -> Self {
        Self::Id(v)
    }
}

impl From<WidgetRef> for WidgetIdOrRef {
    fn from(v: WidgetRef) -> Self {
        Self::Ref(v)
    }
}

pub type FnWidget = fn(WidgetContext) -> WidgetNode;

#[derive(Default)]
pub struct WidgetLifeCycle {
    mount: Vec<Box<dyn FnMut(WidgetMountOrChangeContext) + Send + Sync>>,
    change: Vec<Box<dyn FnMut(WidgetMountOrChangeContext) + Send + Sync>>,
    unmount: Vec<Box<dyn FnMut(WidgetUnmountContext) + Send + Sync>>,
}

impl WidgetLifeCycle {
    pub fn mount<F>(&mut self, f: F)
    where
        F: 'static + FnMut(WidgetMountOrChangeContext) + Send + Sync,
    {
        self.mount.push(Box::new(f));
    }

    pub fn change<F>(&mut self, f: F)
    where
        F: 'static + FnMut(WidgetMountOrChangeContext) + Send + Sync,
    {
        self.change.push(Box::new(f));
    }

    pub fn unmount<F>(&mut self, f: F)
    where
        F: 'static + FnMut(WidgetUnmountContext) + Send + Sync,
    {
        self.unmount.push(Box::new(f));
    }

    #[allow(clippy::type_complexity)]
    pub fn unwrap(
        self,
    ) -> (
        Vec<Box<dyn FnMut(WidgetMountOrChangeContext) + Send + Sync>>,
        Vec<Box<dyn FnMut(WidgetMountOrChangeContext) + Send + Sync>>,
        Vec<Box<dyn FnMut(WidgetUnmountContext) + Send + Sync>>,
    ) {
        let Self {
            mount,
            change,
            unmount,
        } = self;
        (mount, change, unmount)
    }
}

pub fn setup(app: &mut Application) {
    app.register_props::<()>("()");
    app.register_props::<i8>("i8");
    app.register_props::<i16>("i16");
    app.register_props::<i32>("i32");
    app.register_props::<i64>("i64");
    app.register_props::<i128>("i128");
    app.register_props::<u8>("u8");
    app.register_props::<u16>("u16");
    app.register_props::<u32>("u32");
    app.register_props::<u64>("u64");
    app.register_props::<u128>("u128");
    app.register_props::<f32>("f32");
    app.register_props::<f64>("f64");
    app.register_props::<bool>("bool");
    app.register_props::<String>("String");
    app.register_props::<component::containers::content_box::ContentBoxProps>("ContentBoxProps");
    app.register_props::<component::containers::flex_box::FlexBoxProps>("FlexBoxProps");
    app.register_props::<component::containers::grid_box::GridBoxProps>("GridBoxProps");
    app.register_props::<component::containers::horizontal_box::HorizontalBoxProps>(
        "HorizontalBoxProps",
    );
    app.register_props::<component::containers::scroll_box::ScrollBoxOwner>("ScrollBoxOwner");
    app.register_props::<component::containers::scroll_box::SideScrollbarsProps>(
        "SideScrollbarsProps",
    );
    app.register_props::<component::containers::scroll_box::SideScrollbarsState>(
        "SideScrollbarsState",
    );
    app.register_props::<component::containers::size_box::SizeBoxProps>("SizeBoxProps");
    app.register_props::<component::containers::switch_box::SwitchBoxProps>("SwitchBoxProps");
    app.register_props::<component::containers::variant_box::VariantBoxProps>("VariantBoxProps");
    app.register_props::<component::containers::vertical_box::VerticalBoxProps>("VerticalBoxProps");
    app.register_props::<component::containers::wrap_box::WrapBoxProps>("WrapBoxProps");
    app.register_props::<component::image_box::ImageBoxProps>("ImageBoxProps");
    app.register_props::<component::interactive::button::ButtonProps>("ButtonProps");
    app.register_props::<component::interactive::button::ButtonNotifyProps>("ButtonNotifyProps");
    app.register_props::<component::interactive::input_field::TextInputProps>("TextInputProps");
    app.register_props::<component::interactive::input_field::TextInputNotifyProps>(
        "TextInputNotifyProps",
    );
    app.register_props::<component::interactive::navigation::NavItemActive>("NavItemActive");
    app.register_props::<component::interactive::navigation::NavButtonTrackingActive>(
        "NavButtonTrackingActive",
    );
    app.register_props::<component::interactive::navigation::NavContainerActive>(
        "NavContainerActive",
    );
    app.register_props::<component::interactive::navigation::NavJumpLooped>("NavJumpLooped");
    app.register_props::<component::interactive::navigation::NavJumpMapProps>("NavJumpMapProps");
    app.register_props::<component::interactive::scroll_view::ScrollViewState>("ScrollViewState");
    app.register_props::<component::interactive::scroll_view::ScrollViewRange>("ScrollViewRange");
    app.register_props::<component::interactive::scroll_view::ScrollViewNotifyProps>(
        "ScrollViewNotifyProps",
    );
    app.register_props::<component::MessageForwardProps>("MessageForwardProps");
    app.register_props::<component::WidgetAlpha>("WidgetAlpha");
    app.register_props::<component::space_box::SpaceBoxProps>("SpaceBoxProps");
    app.register_props::<component::text_box::TextBoxProps>("TextBoxProps");
    app.register_props::<unit::content::ContentBoxItemLayout>("ContentBoxItemLayout");
    app.register_props::<unit::flex::FlexBoxItemLayout>("FlexBoxItemLayout");
    app.register_props::<unit::grid::GridBoxItemLayout>("GridBoxItemLayout");

    app.register_component(
        "nav_content_box",
        component::containers::content_box::nav_content_box,
    );
    app.register_component(
        "content_box",
        component::containers::content_box::content_box,
    );
    app.register_component(
        "nav_flex_box",
        component::containers::flex_box::nav_flex_box,
    );
    app.register_component("flex_box", component::containers::flex_box::flex_box);
    app.register_component(
        "nav_grid_box",
        component::containers::grid_box::nav_grid_box,
    );
    app.register_component("grid_box", component::containers::grid_box::grid_box);
    app.register_component(
        "nav_horizontal_box",
        component::containers::horizontal_box::nav_horizontal_box,
    );
    app.register_component(
        "horizontal_box",
        component::containers::horizontal_box::horizontal_box,
    );
    app.register_component(
        "nav_scroll_box",
        component::containers::scroll_box::nav_scroll_box,
    );
    app.register_component(
        "nav_scroll_box_side_scrollbars",
        component::containers::scroll_box::nav_scroll_box_side_scrollbars,
    );
    app.register_component("size_box", component::containers::size_box::size_box);
    app.register_component(
        "nav_switch_box",
        component::containers::switch_box::nav_switch_box,
    );
    app.register_component("switch_box", component::containers::switch_box::switch_box);
    app.register_component(
        "variant_box",
        component::containers::variant_box::variant_box,
    );
    app.register_component(
        "nav_vertical_box",
        component::containers::vertical_box::nav_vertical_box,
    );
    app.register_component(
        "vertical_box",
        component::containers::vertical_box::vertical_box,
    );
    app.register_component("wrap_box", component::containers::wrap_box::wrap_box);
    app.register_component("image_box", component::image_box::image_box);
    app.register_component("button", component::interactive::button::button);
    app.register_component(
        "text_input",
        component::interactive::input_field::text_input,
    );
    app.register_component(
        "input_field",
        component::interactive::input_field::input_field,
    );
    app.register_component("space_box", component::space_box::space_box);
    app.register_component("text_box", component::text_box::text_box);
}

#[macro_export]
macro_rules! make_widget {
    ($type_id:path) => {{
        let processor = $type_id;
        let type_name = stringify!($type_id);
        $crate::widget::component::WidgetComponent::new(processor, type_name)
    }};
}

#[macro_export]
macro_rules! widget {
    {()} => ($crate::widget::node::WidgetNode::None);
    {[]} => ($crate::widget::node::WidgetNode::Unit(Default::default()));
    ({{$expr:expr}}) => {
        $crate::widget::node::WidgetNode::Unit($crate::widget::unit::WidgetUnitNode::from($expr))
    };
    ({$expr:expr}) => {
        $crate::widget::node::WidgetNode::from($expr)
    };
    {
        (
            $(
                #{ $key:expr }
            )?
            $(
                | { $idref:expr }
            )?
            $type_id:path
            $(
                : {$props:expr}
            )?
            $(
                | {$shared_props:expr}
            )?
            $(
                {
                    $($named_slot_name:ident = $named_slot_widget:tt)+
                }
            )?
            $(
                |[ $listed_slot_widgets:expr ]|
            )?
            $(
                [
                    $($listed_slot_widget:tt)+
                ]
            )?
        )
    } => {
        {
            #[allow(unused_assignments)]
            #[allow(unused_mut)]
            let mut key = None;
            $(
                key = Some($key.to_string());
            )?
            #[allow(unused_assignments)]
            #[allow(unused_mut)]
            let mut idref = None;
            $(
                idref = Some($crate::widget::WidgetRef::from($idref));
            )?
            let processor = $type_id;
            let type_name = stringify!($type_id).to_owned();
            #[allow(unused_assignments)]
            #[allow(unused_mut)]
            let mut props = $crate::props::Props::default();
            $(
                props = $crate::props::Props::from($props);
            )?
            #[allow(unused_assignments)]
            #[allow(unused_mut)]
            let mut shared_props = None;
            $(
                shared_props = Some($crate::props::Props::from($shared_props));
            )?
            #[allow(unused_mut)]
            let mut named_slots = std::collections::HashMap::new();
            $(
                $(
                    let widget = $crate::widget_wrap!{$named_slot_widget};
                    if widget.is_some() {
                        let name = stringify!($named_slot_name).to_owned();
                        named_slots.insert(name, widget);
                    }
                )+
            )?
            #[allow(unused_mut)]
            let mut listed_slots = vec![];
            $(
                listed_slots.extend($listed_slot_widgets);
            )?
            $(
                $(
                    let widget = $crate::widget_wrap!{$listed_slot_widget};
                    if widget.is_some() {
                        listed_slots.push(widget);
                    }
                )*
            )?
            let component = $crate::widget::component::WidgetComponent {
                processor,
                type_name,
                key,
                idref,
                props,
                shared_props,
                named_slots,
                listed_slots,
            };
            $crate::widget::node::WidgetNode::Component(component)
        }
    };
}

#[macro_export]
macro_rules! widget_wrap {
    ({$expr:expr}) => {
        $crate::widget::node::WidgetNode::from($expr)
    };
    ($tree:tt) => {
        $crate::widget!($tree)
    };
}

#[macro_export]
macro_rules! destruct {
    {$type_id:path { $($prop:ident),+ } ($value:expr) => $code:block} => {
        match $value {
            $type_id { $( $prop ),+ , .. } => $code
        }
    };
}

#[macro_export]
macro_rules! unpack_named_slots {
    ($map:expr => $name:ident) => {
        #[allow(unused_mut)]
        let mut $name = {
            let mut map = $map;
            match map.remove(stringify!($name)) {
                Some(widget) => widget,
                None => $crate::widget::node::WidgetNode::None,
            }
        };
    };
    ($map:expr => { $($name:ident),+ }) => {
        #[allow(unused_mut)]
        let ( $( mut $name ),+ ) = {
            let mut map = $map;
            (
                $(
                    {
                        match map.remove(stringify!($name)) {
                            Some(widget) => widget,
                            None => $crate::widget::node::WidgetNode::None,
                        }
                    }
                ),+
            )
        };
    };
}

#[macro_export]
macro_rules! widget_component {
    {
        $vis:vis $name:ident
        $( ( $( $param:ident ),+ ) )?
        $([ $( $hook_pre:path ),+ $(,)? ])?
        $(|[ $( $hook_post:path ),+ $(,)? ])?
        $code:block
    } => {
        #[allow(unused_mut)]
        $vis fn $name(
            mut context: $crate::widget::context::WidgetContext
        ) -> $crate::widget::node::WidgetNode {
            {
                $(
                    $(
                        context.use_hook($hook_pre);
                    )+
                )?
            }
            let result = {
                $(
                    #[allow(unused_mut)]
                    let $crate::widget::context::WidgetContext {
                        $( mut $param ),+ , ..
                    } = context;
                )?
                $code
            };
            {
                $(
                    $(
                        context.use_hook($hook_post);
                    )+
                )?
            }
            result
        }
    };
}

#[macro_export]
macro_rules! widget_hook {
    {
        $vis:vis $name:ident
        $( ( $( $param:ident ),+ ) )?
        $([ $( $hook_pre:path ),+ $(,)? ])?
        $(|[ $( $hook_post:path ),+ $(,)? ])?
        $code:block
    } => {
        #[allow(unused_mut)]
        $vis fn $name(
            context: &mut $crate::widget::context::WidgetContext
        ) {
            {
                $(
                    $(
                        context.use_hook($hook_pre);
                    )+
                )?
            }
            {
                $(
                    #[allow(unused_mut)]
                    let $crate::widget::context::WidgetContext {
                        $( $param ),+ , ..
                    } = context;
                )?
                $code
            }
            {
                $(
                    $(
                        context.use_hook($hook_post);
                    )+
                )?
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_id() {
        let id = WidgetId::new(
            "type".to_owned(),
            vec!["parent".to_owned(), "me".to_owned()],
        );
        assert_eq!(id.to_string(), "type:/parent/me".to_owned());
        assert_eq!(id.type_name(), "type");
        assert_eq!(id.parts().next().unwrap(), "parent");
        assert_eq!(id.key(), "me");
        assert_eq!(id.clone(), id);
    }
}
