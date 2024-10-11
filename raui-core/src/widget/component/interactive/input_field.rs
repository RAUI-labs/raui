use crate::{
    pre_hooks, unpack_named_slots,
    view_model::ViewModelValue,
    widget::{
        component::interactive::{
            button::{use_button, ButtonProps},
            navigation::{use_nav_item, use_nav_text_input, NavSignal, NavTextChange},
        },
        context::{WidgetContext, WidgetMountOrChangeContext},
        node::WidgetNode,
        unit::area::AreaBoxNode,
        WidgetId, WidgetIdOrRef,
    },
    Integer, MessageData, PropsData, Scalar, UnsignedInteger,
};
use intuicio_data::managed::ManagedLazy;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

fn is_false(v: &bool) -> bool {
    !*v
}

fn is_zero(v: &usize) -> bool {
    *v == 0
}

pub trait TextInputProxy: Send + Sync {
    fn get(&self) -> String;
    fn set(&mut self, value: String);
}

impl<T> TextInputProxy for T
where
    T: ToString + FromStr + Send + Sync,
{
    fn get(&self) -> String {
        self.to_string()
    }

    fn set(&mut self, value: String) {
        if let Ok(value) = value.parse() {
            *self = value;
        }
    }
}

impl<T> TextInputProxy for ViewModelValue<T>
where
    T: ToString + FromStr + Send + Sync,
{
    fn get(&self) -> String {
        self.to_string()
    }

    fn set(&mut self, value: String) {
        if let Ok(value) = value.parse() {
            **self = value;
        }
    }
}

#[derive(Clone)]
pub struct TextInput(ManagedLazy<dyn TextInputProxy>);

impl TextInput {
    pub fn new(data: ManagedLazy<impl TextInputProxy + 'static>) -> Self {
        let (lifetime, data) = data.into_inner();
        let data = data as *mut dyn TextInputProxy;
        unsafe { Self(ManagedLazy::<dyn TextInputProxy>::new_raw(data, lifetime).unwrap()) }
    }

    pub fn into_inner(self) -> ManagedLazy<dyn TextInputProxy> {
        self.0
    }

    pub fn get(&self) -> String {
        self.0.read().map(|data| data.get()).unwrap_or_default()
    }

    pub fn set(&mut self, value: impl ToString) {
        if let Some(mut data) = self.0.write() {
            data.set(value.to_string());
        }
    }
}

impl std::fmt::Debug for TextInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TextInput")
            .field(&self.0.read().map(|data| data.get()).unwrap_or_default())
            .finish()
    }
}

impl<T: TextInputProxy + 'static> From<ManagedLazy<T>> for TextInput {
    fn from(value: ManagedLazy<T>) -> Self {
        Self::new(value)
    }
}

#[derive(PropsData, Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub enum TextInputMode {
    #[default]
    Text,
    Number,
    Integer,
    UnsignedInteger,
    #[serde(skip)]
    Filter(fn(usize, char) -> bool),
}

impl TextInputMode {
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text)
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number)
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer)
    }

    pub fn is_unsigned_integer(&self) -> bool {
        matches!(self, Self::UnsignedInteger)
    }

    pub fn is_filter(&self) -> bool {
        matches!(self, Self::Filter(_))
    }

    pub fn process(&self, text: &str) -> Option<String> {
        match self {
            Self::Text => Some(text.to_owned()),
            Self::Number => text.parse::<Scalar>().ok().map(|v| v.to_string()),
            Self::Integer => text.parse::<Integer>().ok().map(|v| v.to_string()),
            Self::UnsignedInteger => text.parse::<UnsignedInteger>().ok().map(|v| v.to_string()),
            Self::Filter(f) => {
                if text.char_indices().any(|(i, c)| !f(i, c)) {
                    None
                } else {
                    Some(text.to_owned())
                }
            }
        }
    }

    pub fn is_valid(&self, text: &str) -> bool {
        match self {
            Self::Text => true,
            Self::Number => text.parse::<Scalar>().is_ok() || text == "-",
            Self::Integer => text.parse::<Integer>().is_ok() || text == "-",
            Self::UnsignedInteger => text.parse::<UnsignedInteger>().is_ok(),
            Self::Filter(f) => text.char_indices().all(|(i, c)| f(i, c)),
        }
    }
}

#[derive(PropsData, Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct TextInputState {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub focused: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub cursor_position: usize,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct TextInputProps {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub allow_new_line: bool,
    #[serde(default)]
    #[serde(skip)]
    pub text: Option<TextInput>,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct TextInputNotifyProps(
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub WidgetIdOrRef,
);

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct TextInputControlNotifyProps(
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub WidgetIdOrRef,
);

#[derive(MessageData, Debug, Clone)]
#[message_data(crate::messenger::MessageData)]
pub struct TextInputNotifyMessage {
    pub sender: WidgetId,
    pub state: TextInputState,
    pub submitted: bool,
}

#[derive(MessageData, Debug, Clone)]
#[message_data(crate::messenger::MessageData)]
pub struct TextInputControlNotifyMessage {
    pub sender: WidgetId,
    pub character: char,
}

pub fn use_text_input_notified_state(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<TextInputNotifyMessage>() {
                let _ = context.state.write_with(msg.state.to_owned());
            }
        }
    });
}

#[pre_hooks(use_nav_text_input)]
pub fn use_text_input(context: &mut WidgetContext) {
    fn notify(context: &WidgetMountOrChangeContext, data: TextInputNotifyMessage) {
        if let Ok(notify) = context.props.read::<TextInputNotifyProps>() {
            if let Some(to) = notify.0.read() {
                context.messenger.write(to, data);
            }
        }
    }

    context.life_cycle.mount(|context| {
        notify(
            &context,
            TextInputNotifyMessage {
                sender: context.id.to_owned(),
                state: Default::default(),
                submitted: false,
            },
        );
        let _ = context.state.write_with(TextInputState::default());
    });

    context.life_cycle.change(|context| {
        let mode = context.props.read_cloned_or_default::<TextInputMode>();
        let mut props = context.props.read_cloned_or_default::<TextInputProps>();
        let mut state = context.state.read_cloned_or_default::<TextInputState>();
        let mut text = props
            .text
            .as_ref()
            .map(|text| text.get())
            .unwrap_or_default();
        let mut dirty_text = false;
        let mut dirty_state = false;
        let mut submitted = false;
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref() {
                match msg {
                    NavSignal::FocusTextInput(idref) => {
                        state.focused = idref.is_some();
                        dirty_state = true;
                    }
                    NavSignal::TextChange(change) => {
                        if state.focused {
                            match change {
                                NavTextChange::InsertCharacter(c) => {
                                    if c.is_control() {
                                        if let Ok(notify) =
                                            context.props.read::<TextInputControlNotifyProps>()
                                        {
                                            if let Some(to) = notify.0.read() {
                                                context.messenger.write(
                                                    to,
                                                    TextInputControlNotifyMessage {
                                                        sender: context.id.to_owned(),
                                                        character: *c,
                                                    },
                                                );
                                            }
                                        }
                                    } else {
                                        state.cursor_position =
                                            state.cursor_position.min(text.chars().count());
                                        let mut iter = text.chars();
                                        let mut new_text = iter
                                            .by_ref()
                                            .take(state.cursor_position)
                                            .collect::<String>();
                                        new_text.push(*c);
                                        new_text.extend(iter);
                                        if mode.is_valid(&new_text) {
                                            state.cursor_position += 1;
                                            text = new_text;
                                            dirty_text = true;
                                            dirty_state = true;
                                        }
                                    }
                                }
                                NavTextChange::MoveCursorLeft => {
                                    if state.cursor_position > 0 {
                                        state.cursor_position -= 1;
                                        dirty_state = true;
                                    }
                                }
                                NavTextChange::MoveCursorRight => {
                                    if state.cursor_position < text.chars().count() {
                                        state.cursor_position += 1;
                                        dirty_state = true;
                                    }
                                }
                                NavTextChange::MoveCursorStart => {
                                    state.cursor_position = 0;
                                    dirty_state = true;
                                }
                                NavTextChange::MoveCursorEnd => {
                                    state.cursor_position = text.chars().count();
                                    dirty_state = true;
                                }
                                NavTextChange::DeleteLeft => {
                                    if state.cursor_position > 0 {
                                        let mut iter = text.chars();
                                        let mut new_text = iter
                                            .by_ref()
                                            .take(state.cursor_position - 1)
                                            .collect::<String>();
                                        iter.by_ref().next();
                                        new_text.extend(iter);
                                        if mode.is_valid(&new_text) {
                                            state.cursor_position -= 1;
                                            text = new_text;
                                            dirty_text = true;
                                            dirty_state = true;
                                        }
                                    }
                                }
                                NavTextChange::DeleteRight => {
                                    let mut iter = text.chars();
                                    let mut new_text = iter
                                        .by_ref()
                                        .take(state.cursor_position)
                                        .collect::<String>();
                                    iter.by_ref().next();
                                    new_text.extend(iter);
                                    if mode.is_valid(&new_text) {
                                        text = new_text;
                                        dirty_text = true;
                                        dirty_state = true;
                                    }
                                }
                                NavTextChange::NewLine => {
                                    if props.allow_new_line {
                                        let mut iter = text.chars();
                                        let mut new_text = iter
                                            .by_ref()
                                            .take(state.cursor_position)
                                            .collect::<String>();
                                        new_text.push('\n');
                                        new_text.extend(iter);
                                        if mode.is_valid(&new_text) {
                                            state.cursor_position += 1;
                                            text = new_text;
                                            dirty_text = true;
                                            dirty_state = true;
                                        }
                                    } else {
                                        submitted = true;
                                        dirty_state = true;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        if dirty_state {
            state.cursor_position = state.cursor_position.min(text.chars().count());
            notify(
                &context,
                TextInputNotifyMessage {
                    sender: context.id.to_owned(),
                    state,
                    submitted,
                },
            );
            let _ = context.state.write_with(state);
        }
        if dirty_text {
            if let Some(data) = props.text.as_mut() {
                data.set(text);
                context.messenger.write(context.id.to_owned(), ());
            }
        }
        if submitted {
            context.signals.write(NavSignal::FocusTextInput(().into()));
        }
    });
}

#[pre_hooks(use_button, use_text_input)]
pub fn use_input_field(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        let focused = context
            .state
            .map_or_default::<TextInputState, _, _>(|s| s.focused);
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref() {
                match msg {
                    NavSignal::Accept(true) => {
                        if !focused {
                            context
                                .signals
                                .write(NavSignal::FocusTextInput(context.id.to_owned().into()));
                        }
                    }
                    NavSignal::Cancel(true) => {
                        if focused {
                            context.signals.write(NavSignal::FocusTextInput(().into()));
                        }
                    }
                    _ => {}
                }
            }
        }
    });
}

#[pre_hooks(use_nav_item, use_text_input)]
pub fn text_input(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        state,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    if let Some(p) = content.props_mut() {
        p.write(state.read_cloned_or_default::<TextInputState>());
        p.write(props.read_cloned_or_default::<TextInputProps>());
    }

    AreaBoxNode {
        id: id.to_owned(),
        slot: Box::new(content),
    }
    .into()
}

#[pre_hooks(use_nav_item, use_input_field)]
pub fn input_field(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        state,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    if let Some(p) = content.props_mut() {
        p.write(state.read_cloned_or_default::<ButtonProps>());
        p.write(state.read_cloned_or_default::<TextInputState>());
        p.write(props.read_cloned_or_default::<TextInputProps>());
    }

    AreaBoxNode {
        id: id.to_owned(),
        slot: Box::new(content),
    }
    .into()
}

pub fn input_text_with_cursor(text: &str, position: usize, cursor: char) -> String {
    text.chars()
        .take(position)
        .chain(std::iter::once(cursor))
        .chain(text.chars().skip(position))
        .collect()
}
