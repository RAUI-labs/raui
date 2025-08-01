//! Widget types and the core component collection

pub mod component;
pub mod context;
pub mod node;
pub mod unit;
pub mod utils;

use crate::{
    Prefab, PropsData,
    application::Application,
    props::PropsData,
    widget::{
        context::{WidgetContext, WidgetMountOrChangeContext, WidgetUnmountContext},
        node::WidgetNode,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::hash_map::DefaultHasher,
    convert::TryFrom,
    hash::{Hash, Hasher},
    ops::{Deref, Range},
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

#[derive(Debug, Clone, Copy)]
pub struct WidgetIdMetaParam<'a> {
    pub name: &'a str,
    pub value: Option<&'a str>,
}

impl WidgetIdMetaParam<'_> {
    pub fn is_flag(&self) -> bool {
        self.value.is_none()
    }

    pub fn has_value(&self) -> bool {
        self.value.is_some()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WidgetIdMetaParams<'a>(&'a str);

impl<'a> WidgetIdMetaParams<'a> {
    pub fn new(meta: &'a str) -> Self {
        Self(meta)
    }

    pub fn iter(&self) -> impl Iterator<Item = WidgetIdMetaParam> {
        self.0.split('&').filter_map(|part| {
            if let Some(index) = part.find('=') {
                let name = &part[0..index];
                let value = &part[(index + b"=".len())..];
                if name.is_empty() {
                    None
                } else {
                    Some(WidgetIdMetaParam {
                        name,
                        value: Some(value),
                    })
                }
            } else if part.is_empty() {
                None
            } else {
                Some(WidgetIdMetaParam {
                    name: part,
                    value: None,
                })
            }
        })
    }

    pub fn find(&self, name: &str) -> Option<WidgetIdMetaParam> {
        self.iter().find(|param| param.name == name)
    }

    pub fn has_flag(&self, name: &str) -> bool {
        self.iter()
            .any(|param| param.name == name && param.is_flag())
    }

    pub fn find_value(&self, name: &str) -> Option<&str> {
        self.iter().find_map(|param| {
            if param.name == name {
                param.value
            } else {
                None
            }
        })
    }
}

#[derive(PropsData, Default, Clone, Serialize, Deserialize)]
#[serde(try_from = "WidgetIdDef")]
#[serde(into = "WidgetIdDef")]
pub struct WidgetId {
    id: String,
    type_name: Range<usize>,
    /// [(key range, meta range)]
    parts: Vec<(Range<usize>, Range<usize>)>,
}

impl WidgetId {
    pub fn empty() -> Self {
        Self {
            id: ":".to_owned(),
            type_name: 0..0,
            parts: Default::default(),
        }
    }

    pub fn new(type_name: &str, path: &[Cow<'_, str>]) -> Self {
        if path.is_empty() {
            return Self {
                id: format!("{type_name}:"),
                type_name: 0..type_name.len(),
                parts: Default::default(),
            };
        }
        let count = type_name.len()
            + b":".len()
            + path.iter().map(|part| part.len()).sum::<usize>()
            + path.len().saturating_sub(1) * b"/".len();
        let mut result = String::with_capacity(count);
        let mut position = result.len();
        result.push_str(type_name);
        let type_name = 0..result.len();
        result.push(':');
        let parts = path
            .iter()
            .enumerate()
            .map(|(index, part)| {
                if index > 0 {
                    result.push('/');
                }
                position = result.len();
                result.push_str(part);
                let range = position..result.len();
                if let Some(index) = part.find('?') {
                    let key = range.start..(range.start + index);
                    let meta = (range.start + index + b"?".len())..range.end;
                    (key, meta)
                } else {
                    let meta = range.end..range.end;
                    (range, meta)
                }
            })
            .collect::<Vec<_>>();
        Self {
            id: result,
            type_name,
            parts,
        }
    }

    pub fn push(&self, part: &str) -> Self {
        let count = self.id.len() + b"/".len();
        let mut result = String::with_capacity(count);
        result.push_str(&self.id);
        if self.depth() > 0 {
            result.push('/');
        }
        let position = result.len();
        result.push_str(part);
        let range = position..result.len();
        let (key, meta) = if let Some(index) = part.find('?') {
            let key = range.start..(range.start + index);
            let meta = (range.start + index + b"?".len())..range.end;
            (key, meta)
        } else {
            let meta = range.end..range.end;
            (range, meta)
        };
        let parts = self
            .parts
            .iter()
            .cloned()
            .chain(std::iter::once((key, meta)))
            .collect();
        Self {
            id: result,
            type_name: self.type_name.to_owned(),
            parts,
        }
    }

    pub fn pop(&self) -> Self {
        let parts = self.parts[0..(self.parts.len().saturating_sub(1))].to_owned();
        let result = if let Some(range) = parts.last() {
            self.id[0..range.1.end].to_owned()
        } else {
            format!("{}:", self.type_name())
        };
        Self {
            id: result,
            type_name: self.type_name.to_owned(),
            parts,
        }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.id.is_empty()
    }

    #[inline]
    pub fn depth(&self) -> usize {
        self.parts.len()
    }

    #[inline]
    pub fn type_name(&self) -> &str {
        &self.id.as_str()[self.type_name.clone()]
    }

    #[inline]
    pub fn path(&self) -> &str {
        if self.parts.is_empty() {
            &self.id.as_str()[0..0]
        } else {
            &self.id.as_str()[self.parts.first().unwrap().0.start..self.parts.last().unwrap().1.end]
        }
    }

    #[inline]
    pub fn key(&self) -> &str {
        if self.parts.is_empty() {
            &self.id.as_str()[0..0]
        } else {
            &self.id[self.parts.last().cloned().unwrap().0]
        }
    }

    #[inline]
    pub fn meta(&self) -> &str {
        if self.parts.is_empty() {
            &self.id.as_str()[0..0]
        } else {
            &self.id[self.parts.last().cloned().unwrap().1]
        }
    }

    #[inline]
    pub fn part(&self, index: usize) -> Option<&str> {
        self.parts
            .get(index)
            .cloned()
            .map(|(key, meta)| &self.id[key.start..meta.end])
    }

    #[inline]
    pub fn part_key_meta(&self, index: usize) -> Option<(&str, &str)> {
        self.parts
            .get(index)
            .cloned()
            .map(|(key, meta)| (&self.id[key], &self.id[meta]))
    }

    pub fn range(&self, from_inclusive: usize, to_exclusive: usize) -> &str {
        if self.parts.is_empty() {
            return &self.id[0..0];
        }
        let start = from_inclusive.min(self.parts.len().saturating_sub(1));
        let end = to_exclusive
            .saturating_sub(1)
            .max(start)
            .min(self.parts.len().saturating_sub(1));
        let start = self.parts[start].0.start;
        let end = self.parts[end].1.end;
        &self.id[start..end]
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        match self.distance_to(other) {
            Ok(v) => v < 0,
            _ => false,
        }
    }

    pub fn is_superset_of(&self, other: &Self) -> bool {
        match self.distance_to(other) {
            Ok(v) => v > 0,
            _ => false,
        }
    }

    pub fn distance_to(&self, other: &Self) -> Result<isize, isize> {
        for index in 0..self.depth().max(other.depth()) {
            match (self.part(index), other.part(index)) {
                (None, None) => return Ok(0),
                (None, Some(_)) | (Some(_), None) => {
                    return Ok(self.depth() as isize - other.depth() as isize);
                }
                (Some(a), Some(b)) => {
                    if a != b {
                        return Err(index as isize - other.depth() as isize);
                    }
                }
            }
        }
        Ok(0)
    }

    #[inline]
    pub fn parts(&self) -> impl Iterator<Item = &str> + '_ {
        self.parts
            .iter()
            .cloned()
            .map(move |(key, meta)| &self.id[key.start..meta.end])
    }

    #[inline]
    pub fn parts_key_meta(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.parts
            .iter()
            .cloned()
            .map(move |(key, meta)| (&self.id[key], &self.id[meta]))
    }

    #[inline]
    pub fn rparts(&self) -> impl Iterator<Item = &str> + '_ {
        self.parts
            .iter()
            .rev()
            .cloned()
            .map(move |(key, meta)| &self.id[key.start..meta.end])
    }

    #[inline]
    pub fn rparts_key_meta(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.parts
            .iter()
            .rev()
            .cloned()
            .map(move |(key, meta)| (&self.id[key], &self.id[meta]))
    }

    pub fn hashed_value(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl Hash for WidgetId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for WidgetId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for WidgetId {}

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
            let rest = &s[(index + b":".len())..];
            let path = rest.split('/').map(Cow::Borrowed).collect::<Vec<_>>();
            Ok(Self::new(&type_name, &path))
        } else {
            Err(())
        }
    }
}

impl std::fmt::Debug for WidgetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WidgetId")
            .field("id", &self.id)
            .field("type_name", &&self.id[self.type_name.clone()])
            .field(
                "parts",
                &self
                    .parts
                    .iter()
                    .map(|(key, meta)| (&self.id[key.to_owned()], &self.id[meta.to_owned()]))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl std::fmt::Display for WidgetId {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetIdCommon {
    id: Option<WidgetId>,
    count: usize,
}

impl Default for WidgetIdCommon {
    fn default() -> Self {
        Self {
            id: None,
            count: usize::MAX,
        }
    }
}

impl WidgetIdCommon {
    pub fn new(id: WidgetId) -> Self {
        Self {
            count: id.depth(),
            id: Some(id),
        }
    }

    pub fn include(&mut self, id: &WidgetId) -> &mut Self {
        if self.id.is_none() {
            self.id = Some(id.to_owned());
            self.count = id.depth();
            return self;
        }
        if let Some(source) = self.id.as_ref() {
            for index in 0..self.count.min(id.depth()) {
                if source.part(index) != id.part(index) {
                    self.count = index;
                    return self;
                }
            }
        }
        self
    }

    pub fn include_other(&mut self, other: &Self) -> &mut Self {
        if let Some(id) = other.id.as_ref() {
            if self.id.is_none() {
                self.id = Some(id.to_owned());
                self.count = other.count;
                return self;
            }
            if let Some(source) = self.id.as_ref() {
                for index in 0..self.count.min(other.count) {
                    if source.part(index) != id.part(index) {
                        self.count = index;
                        return self;
                    }
                }
            }
        }
        self
    }

    pub fn is_valid(&self) -> bool {
        self.id.is_some()
    }

    pub fn parts(&self) -> Option<impl Iterator<Item = &str>> {
        self.id
            .as_ref()
            .map(|id| (0..self.count).map_while(move |index| id.part(index)))
    }

    pub fn path(&self) -> Option<&str> {
        self.id
            .as_ref()
            .map(|id| id.range(0, self.count))
            .filter(|id| !id.is_empty())
    }
}

impl<'a> FromIterator<&'a WidgetId> for WidgetIdCommon {
    fn from_iter<T: IntoIterator<Item = &'a WidgetId>>(iter: T) -> Self {
        let mut result = Self::default();
        for id in iter {
            result.include(id);
        }
        result
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

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[serde(from = "WidgetRefDef")]
#[serde(into = "WidgetRefDef")]
pub struct WidgetRef(#[serde(skip)] Arc<RwLock<Option<WidgetId>>>);

impl WidgetRef {
    pub fn new(id: WidgetId) -> Self {
        Self(Arc::new(RwLock::new(Some(id))))
    }

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

    pub fn exists(&self) -> bool {
        self.0
            .read()
            .ok()
            .map(|data| data.is_some())
            .unwrap_or_default()
    }
}

impl PartialEq for WidgetRef {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl From<WidgetRefDef> for WidgetRef {
    fn from(data: WidgetRefDef) -> Self {
        WidgetRef(Arc::new(RwLock::new(data.0)))
    }
}

impl std::fmt::Display for WidgetRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(id) = self.0.read() {
            if let Some(id) = id.as_ref() {
                write!(f, "{id}")
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

#[derive(PropsData, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum WidgetIdOrRef {
    #[default]
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

impl std::fmt::Display for WidgetIdOrRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::Id(id) => write!(f, "{id}"),
            Self::Ref(id) => write!(f, "{id}"),
        }
    }
}

#[derive(Clone)]
pub enum FnWidget {
    Pointer(fn(WidgetContext) -> WidgetNode),
    Closure(Arc<dyn Fn(WidgetContext) -> WidgetNode + Send + Sync>),
}

impl FnWidget {
    pub fn pointer(value: fn(WidgetContext) -> WidgetNode) -> Self {
        Self::Pointer(value)
    }

    pub fn closure(value: impl Fn(WidgetContext) -> WidgetNode + Send + Sync + 'static) -> Self {
        Self::Closure(Arc::new(value))
    }

    pub fn call(&self, context: WidgetContext) -> WidgetNode {
        match self {
            Self::Pointer(value) => value(context),
            Self::Closure(value) => value(context),
        }
    }
}

#[derive(Default)]
pub struct WidgetLifeCycle {
    #[allow(clippy::type_complexity)]
    mount: Vec<Box<dyn FnMut(WidgetMountOrChangeContext) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    change: Vec<Box<dyn FnMut(WidgetMountOrChangeContext) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
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

pub fn none_widget(_: WidgetContext) -> WidgetNode {
    Default::default()
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
    app.register_props::<component::containers::anchor_box::AnchorProps>("AnchorProps");
    app.register_props::<component::containers::anchor_box::PivotBoxProps>("PivotBoxProps");
    app.register_props::<component::containers::content_box::ContentBoxProps>("ContentBoxProps");
    app.register_props::<component::containers::flex_box::FlexBoxProps>("FlexBoxProps");
    app.register_props::<component::containers::float_box::FloatBoxProps>("FloatBoxProps");
    app.register_props::<component::containers::float_box::FloatBoxNotifyProps>(
        "FloatBoxNotifyProps",
    );
    app.register_props::<component::containers::float_box::FloatBoxState>("FloatBoxState");
    app.register_props::<component::containers::grid_box::GridBoxProps>("GridBoxProps");
    app.register_props::<component::containers::horizontal_box::HorizontalBoxProps>(
        "HorizontalBoxProps",
    );
    app.register_props::<component::containers::hidden_box::HiddenBoxProps>("HiddenBoxProps");
    app.register_props::<component::containers::scroll_box::ScrollBoxOwner>("ScrollBoxOwner");
    app.register_props::<component::containers::scroll_box::SideScrollbarsProps>(
        "SideScrollbarsProps",
    );
    app.register_props::<component::containers::scroll_box::SideScrollbarsState>(
        "SideScrollbarsState",
    );
    app.register_props::<component::containers::portal_box::PortalsContainer>("PortalsContainer");
    app.register_props::<component::containers::responsive_box::MediaQueryExpression>(
        "MediaQueryExpression",
    );
    app.register_props::<component::containers::responsive_box::ResponsiveBoxState>(
        "ResponsiveBoxState",
    );
    app.register_props::<component::containers::size_box::SizeBoxProps>("SizeBoxProps");
    app.register_props::<component::containers::switch_box::SwitchBoxProps>("SwitchBoxProps");
    app.register_props::<component::containers::tabs_box::TabsBoxProps>("TabsBoxProps");
    app.register_props::<component::containers::tabs_box::TabPlateProps>("TabPlateProps");
    app.register_props::<component::containers::tooltip_box::TooltipState>("TooltipState");
    app.register_props::<component::containers::variant_box::VariantBoxProps>("VariantBoxProps");
    app.register_props::<component::containers::vertical_box::VerticalBoxProps>("VerticalBoxProps");
    app.register_props::<component::containers::wrap_box::WrapBoxProps>("WrapBoxProps");
    app.register_props::<component::image_box::ImageBoxProps>("ImageBoxProps");
    app.register_props::<component::interactive::button::ButtonProps>("ButtonProps");
    app.register_props::<component::interactive::button::ButtonNotifyProps>("ButtonNotifyProps");
    app.register_props::<component::interactive::input_field::TextInputMode>("TextInputMode");
    app.register_props::<component::interactive::input_field::TextInputProps>("TextInputProps");
    app.register_props::<component::interactive::input_field::TextInputState>("TextInputState");
    app.register_props::<component::interactive::input_field::TextInputNotifyProps>(
        "TextInputNotifyProps",
    );
    app.register_props::<component::interactive::input_field::TextInputControlNotifyProps>(
        "TextInputControlNotifyProps",
    );
    app.register_props::<component::interactive::options_view::OptionsViewMode>("OptionsViewMode");
    app.register_props::<component::interactive::options_view::OptionsViewProps>(
        "OptionsViewProps",
    );
    app.register_props::<component::interactive::slider_view::SliderViewProps>("SliderViewProps");
    app.register_props::<component::interactive::navigation::NavItemActive>("NavItemActive");
    app.register_props::<component::interactive::navigation::NavTrackingActive>(
        "NavTrackingActive",
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

    app.register_component("none_widget", FnWidget::pointer(none_widget));
    app.register_component(
        "area_box",
        FnWidget::pointer(component::containers::area_box::area_box),
    );
    app.register_component(
        "anchor_box",
        FnWidget::pointer(component::containers::anchor_box::anchor_box),
    );
    app.register_component(
        "pivot_box",
        FnWidget::pointer(component::containers::anchor_box::pivot_box),
    );
    app.register_component(
        "nav_content_box",
        FnWidget::pointer(component::containers::content_box::nav_content_box),
    );
    app.register_component(
        "content_box",
        FnWidget::pointer(component::containers::content_box::content_box),
    );
    app.register_component(
        "nav_flex_box",
        FnWidget::pointer(component::containers::flex_box::nav_flex_box),
    );
    app.register_component(
        "flex_box",
        FnWidget::pointer(component::containers::flex_box::flex_box),
    );
    app.register_component(
        "float_box",
        FnWidget::pointer(component::containers::float_box::float_box),
    );
    app.register_component(
        "nav_grid_box",
        FnWidget::pointer(component::containers::grid_box::nav_grid_box),
    );
    app.register_component(
        "grid_box",
        FnWidget::pointer(component::containers::grid_box::grid_box),
    );
    app.register_component(
        "nav_horizontal_box",
        FnWidget::pointer(component::containers::horizontal_box::nav_horizontal_box),
    );
    app.register_component(
        "horizontal_box",
        FnWidget::pointer(component::containers::horizontal_box::horizontal_box),
    );
    app.register_component(
        "nav_scroll_box",
        FnWidget::pointer(component::containers::scroll_box::nav_scroll_box),
    );
    app.register_component(
        "nav_scroll_box_side_scrollbars",
        FnWidget::pointer(component::containers::scroll_box::nav_scroll_box_side_scrollbars),
    );
    app.register_component(
        "portal_box",
        FnWidget::pointer(component::containers::portal_box::portal_box),
    );
    app.register_component(
        "responsive_box",
        FnWidget::pointer(component::containers::responsive_box::responsive_box),
    );
    app.register_component(
        "responsive_props_box",
        FnWidget::pointer(component::containers::responsive_box::responsive_props_box),
    );
    app.register_component(
        "size_box",
        FnWidget::pointer(component::containers::size_box::size_box),
    );
    app.register_component(
        "nav_switch_box",
        FnWidget::pointer(component::containers::switch_box::nav_switch_box),
    );
    app.register_component(
        "switch_box",
        FnWidget::pointer(component::containers::switch_box::switch_box),
    );
    app.register_component(
        "nav_tabs_box",
        FnWidget::pointer(component::containers::tabs_box::nav_tabs_box),
    );
    app.register_component(
        "tooltip_box",
        FnWidget::pointer(component::containers::tooltip_box::tooltip_box),
    );
    app.register_component(
        "portals_tooltip_box",
        FnWidget::pointer(component::containers::tooltip_box::portals_tooltip_box),
    );
    app.register_component(
        "variant_box",
        FnWidget::pointer(component::containers::variant_box::variant_box),
    );
    app.register_component(
        "nav_vertical_box",
        FnWidget::pointer(component::containers::vertical_box::nav_vertical_box),
    );
    app.register_component(
        "vertical_box",
        FnWidget::pointer(component::containers::vertical_box::vertical_box),
    );
    app.register_component(
        "wrap_box",
        FnWidget::pointer(component::containers::wrap_box::wrap_box),
    );
    app.register_component(
        "button",
        FnWidget::pointer(component::interactive::button::button),
    );
    app.register_component(
        "tracked_button",
        FnWidget::pointer(component::interactive::button::tracked_button),
    );
    app.register_component(
        "self_tracked_button",
        FnWidget::pointer(component::interactive::button::self_tracked_button),
    );
    app.register_component(
        "text_input",
        FnWidget::pointer(component::interactive::input_field::text_input),
    );
    app.register_component(
        "input_field",
        FnWidget::pointer(component::interactive::input_field::input_field),
    );
    app.register_component(
        "options_view",
        FnWidget::pointer(component::interactive::options_view::options_view),
    );
    app.register_component(
        "slider_view",
        FnWidget::pointer(component::interactive::slider_view::slider_view),
    );
    app.register_component(
        "navigation_barrier",
        FnWidget::pointer(component::interactive::navigation::navigation_barrier),
    );
    app.register_component(
        "tracking",
        FnWidget::pointer(component::interactive::navigation::tracking),
    );
    app.register_component(
        "self_tracking",
        FnWidget::pointer(component::interactive::navigation::self_tracking),
    );
    app.register_component(
        "space_box",
        FnWidget::pointer(component::space_box::space_box),
    );
    app.register_component(
        "image_box",
        FnWidget::pointer(component::image_box::image_box),
    );
    app.register_component("text_box", FnWidget::pointer(component::text_box::text_box));
}

/// Helper to manually create a [`WidgetComponent`][crate::widget::component::WidgetComponent]
/// struct from a function.
///
/// Users will not usually need this macro, but it can be useful in some advanced cases or where you
/// don't want to use the [`widget`] macro.
/// ```
#[macro_export]
macro_rules! make_widget {
    ($type_id:path) => {{
        let processor = $type_id;
        let type_name = stringify!($type_id);
        $crate::widget::component::WidgetComponent::new(
            $crate::widget::FnWidget::pointer(processor),
            type_name,
        )
    }};
}

/// A helper for getting the named children out of a widget context.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_id() {
        let id = WidgetId::empty();
        assert_eq!(id.type_name(), "");
        assert_eq!(id.path(), "");
        assert_eq!(id.depth(), 0);

        let id = WidgetId::new("type", &["parent".into(), "me".into()]);
        assert_eq!(id.to_string(), "type:parent/me".to_owned());
        assert_eq!(id.type_name(), "type");
        assert_eq!(id.parts().next().unwrap(), "parent");
        assert_eq!(id.key(), "me");
        assert_eq!(id.clone(), id);

        let a = WidgetId::from_str("a:root/a").unwrap();
        let b = WidgetId::from_str("b:root/b").unwrap();
        let mut common = WidgetIdCommon::default();
        assert_eq!(common.path(), None);
        common.include(&a);
        assert_eq!(common.path(), Some("root/a"));
        let mut common = WidgetIdCommon::default();
        common.include(&b);
        assert_eq!(common.path(), Some("root/b"));
        common.include(&a);
        assert_eq!(common.path(), Some("root"));

        let id = WidgetId::from_str("type:parent/me").unwrap();
        assert_eq!(&*id, "type:parent/me");
        assert_eq!(id.path(), "parent/me");
        let id = id.pop();
        assert_eq!(&*id, "type:parent");
        assert_eq!(id.path(), "parent");
        let id = id.pop();
        assert_eq!(&*id, "type:");
        assert_eq!(id.path(), "");
        let id = id.push("parent");
        assert_eq!(&*id, "type:parent");
        assert_eq!(id.path(), "parent");
        let id = id.push("me");
        assert_eq!(&*id, "type:parent/me");
        assert_eq!(id.path(), "parent/me");
        assert_eq!(id.key(), "me");
        assert_eq!(id.meta(), "");
        let id = id.push("with?meta");
        assert_eq!(&*id, "type:parent/me/with?meta");
        assert_eq!(id.path(), "parent/me/with?meta");
        assert_eq!(id.key(), "with");
        assert_eq!(id.meta(), "meta");

        let a = WidgetId::from_str("a:root/a/b/c").unwrap();
        let b = WidgetId::from_str("b:root/a/b/c").unwrap();
        assert_eq!(a.distance_to(&b), Ok(0));
        assert!(!a.is_subset_of(&b));
        assert!(!a.is_superset_of(&b));
        let a = WidgetId::from_str("a:root/a/b").unwrap();
        let b = WidgetId::from_str("b:root/a/b/c").unwrap();
        assert_eq!(a.distance_to(&b), Ok(-1));
        assert!(a.is_subset_of(&b));
        assert!(!a.is_superset_of(&b));
        let a = WidgetId::from_str("a:root/a").unwrap();
        let b = WidgetId::from_str("b:root/a/b/c").unwrap();
        assert_eq!(a.distance_to(&b), Ok(-2));
        assert!(a.is_subset_of(&b));
        assert!(!a.is_superset_of(&b));
        let a = WidgetId::from_str("a:root").unwrap();
        let b = WidgetId::from_str("b:root/a/b/c").unwrap();
        assert_eq!(a.distance_to(&b), Ok(-3));
        assert!(a.is_subset_of(&b));
        assert!(!a.is_superset_of(&b));
        let a = WidgetId::from_str("a:root/a/b/c").unwrap();
        let b = WidgetId::from_str("b:root/a/b").unwrap();
        assert_eq!(a.distance_to(&b), Ok(1));
        assert!(!a.is_subset_of(&b));
        assert!(a.is_superset_of(&b));
        let a = WidgetId::from_str("a:root/a/b/c").unwrap();
        let b = WidgetId::from_str("b:root/a").unwrap();
        assert_eq!(a.distance_to(&b), Ok(2));
        assert!(!a.is_subset_of(&b));
        assert!(a.is_superset_of(&b));
        let a = WidgetId::from_str("a:root/a/b/c").unwrap();
        let b = WidgetId::from_str("b:root").unwrap();
        assert_eq!(a.distance_to(&b), Ok(3));
        assert!(!a.is_subset_of(&b));
        assert!(a.is_superset_of(&b));
        let a = WidgetId::from_str("a:root/a/b/x").unwrap();
        let b = WidgetId::from_str("b:root/a/b/c").unwrap();
        assert_eq!(a.distance_to(&b), Err(-1));
        assert!(!a.is_subset_of(&b));
        assert!(!a.is_superset_of(&b));
        let a = WidgetId::from_str("a:root/a/x/y").unwrap();
        let b = WidgetId::from_str("b:root/a/b/c").unwrap();
        assert_eq!(a.distance_to(&b), Err(-2));
        assert!(!a.is_subset_of(&b));
        assert!(!a.is_superset_of(&b));
        let a = WidgetId::from_str("a:root/x/y/z").unwrap();
        let b = WidgetId::from_str("b:root/a/b/c").unwrap();
        assert_eq!(a.distance_to(&b), Err(-3));
        assert!(!a.is_subset_of(&b));
        assert!(!a.is_superset_of(&b));
    }
}
