use crate::{messenger::MessageSender, widget::WidgetId, Scalar};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::mpsc::Sender};

pub enum AnimationError {
    CouldNotReadData,
    CouldNotWriteData,
}

#[derive(Clone)]
pub struct AnimationUpdate(Sender<(String, Option<Animation>)>);

impl AnimationUpdate {
    pub fn new(sender: Sender<(String, Option<Animation>)>) -> Self {
        Self(sender)
    }

    pub fn change(&self, name: &str, data: Option<Animation>) -> Result<(), AnimationError> {
        if self.0.send((name.to_owned(), data)).is_err() {
            Err(AnimationError::CouldNotWriteData)
        } else {
            Ok(())
        }
    }
}

pub struct Animator<'a> {
    states: &'a AnimatorStates,
    update: AnimationUpdate,
}

impl<'a> Animator<'a> {
    #[inline]
    pub fn new(states: &'a AnimatorStates, update: AnimationUpdate) -> Self {
        Self { states, update }
    }

    #[inline]
    pub fn has(&self, id: &str) -> bool {
        self.states.has(id)
    }

    #[inline]
    pub fn change(&self, name: &str, data: Option<Animation>) -> Result<(), AnimationError> {
        self.update.change(name, data)
    }

    /// (progress factor, time, duration)
    #[inline]
    pub fn value(&self, id: &str, name: &str) -> Option<(Scalar, Scalar, Scalar)> {
        self.states.value(id, name)
    }

    #[inline]
    pub fn value_progress(&self, id: &str, name: &str) -> Option<Scalar> {
        self.value(id, name).map(|v| v.0)
    }

    #[inline]
    pub fn value_progress_or(&self, id: &str, name: &str, v: Scalar) -> Scalar {
        self.value_progress(id, name).unwrap_or(v)
    }

    #[inline]
    pub fn value_progress_or_zero(&self, id: &str, name: &str) -> Scalar {
        self.value_progress(id, name).unwrap_or(0.0)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AnimatorStates(
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub HashMap<String, AnimatorState>,
);

impl AnimatorStates {
    pub fn new(name: String, animation: Animation) -> Self {
        let mut result = HashMap::with_capacity(1);
        result.insert(name, AnimatorState::new(animation));
        Self(result)
    }

    pub fn in_progress(&self) -> bool {
        self.0.values().any(|s| s.in_progress())
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        !self.in_progress()
    }

    #[inline]
    pub fn has(&self, id: &str) -> bool {
        self.0.contains_key(id)
    }

    /// (progress factor, time, duration)
    #[inline]
    pub fn value(&self, id: &str, name: &str) -> Option<(Scalar, Scalar, Scalar)> {
        if let Some(state) = self.0.get(id) {
            state.value(name)
        } else {
            None
        }
    }

    #[inline]
    pub fn value_progress(&self, id: &str, name: &str) -> Option<Scalar> {
        if let Some(state) = self.0.get(id) {
            state.value_progress(name)
        } else {
            None
        }
    }

    #[inline]
    pub fn value_progress_or(&self, id: &str, name: &str, v: Scalar) -> Scalar {
        if let Some(state) = self.0.get(id) {
            state.value_progress_or(name, v)
        } else {
            v
        }
    }

    #[inline]
    pub fn value_progress_or_zero(&self, id: &str, name: &str) -> Scalar {
        if let Some(state) = self.0.get(id) {
            state.value_progress_or_zero(name)
        } else {
            0.0
        }
    }

    pub fn change(&mut self, name: String, animation: Option<Animation>) {
        if let Some(animation) = animation {
            self.0.insert(name, AnimatorState::new(animation));
        } else {
            self.0.remove(&name);
        }
    }

    pub fn process(
        &mut self,
        delta_time: Scalar,
        owner: &WidgetId,
        message_sender: &MessageSender,
    ) {
        for state in self.0.values_mut() {
            state.process(delta_time, owner, message_sender);
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AnimatorState {
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    sheet: HashMap<String, AnimationPhase>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    messages: Vec<(Scalar, String)>,
    #[serde(default)]
    time: Scalar,
    #[serde(default)]
    duration: Scalar,
    #[serde(default)]
    looped: bool,
}

impl AnimatorState {
    pub fn new(animation: Animation) -> Self {
        let mut sheet = HashMap::new();
        let mut messages = vec![];
        let (time, looped) = Self::include_animation(animation, &mut sheet, &mut messages, 0.0);
        Self {
            sheet,
            messages,
            time: 0.0,
            duration: time,
            looped,
        }
    }

    #[inline]
    pub fn in_progress(&self) -> bool {
        self.looped || (self.time <= self.duration && !self.sheet.is_empty())
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        !self.in_progress()
    }

    /// (progress factor, time, duration)
    #[inline]
    pub fn value(&self, name: &str) -> Option<(Scalar, Scalar, Scalar)> {
        self.sheet
            .get(name)
            .map(|p| (p.cached_progress, p.cached_time, p.duration))
    }

    #[inline]
    pub fn value_progress(&self, name: &str) -> Option<Scalar> {
        self.sheet.get(name).map(|p| p.cached_progress)
    }

    #[inline]
    pub fn value_progress_or(&self, name: &str, v: Scalar) -> Scalar {
        self.value_progress(name).unwrap_or(v)
    }

    #[inline]
    pub fn value_progress_or_zero(&self, name: &str) -> Scalar {
        self.value_progress(name).unwrap_or(0.0)
    }

    pub fn process(
        &mut self,
        delta_time: Scalar,
        owner: &WidgetId,
        message_sender: &MessageSender,
    ) {
        if delta_time > 0.0 {
            if self.looped && self.time > self.duration {
                self.time = 0.0;
            }
            let old_time = self.time;
            self.time += delta_time;
            for phase in self.sheet.values_mut() {
                phase.cached_time = (self.time - phase.start).min(phase.duration).max(0.0);
                phase.cached_progress = if phase.duration > 0.0 {
                    phase.cached_time / phase.duration
                } else {
                    0.0
                };
            }
            for (time, message) in &self.messages {
                if *time >= old_time && *time < self.time {
                    message_sender.write(owner.to_owned(), AnimationMessage(message.to_owned()));
                }
            }
        }
    }

    fn include_animation(
        animation: Animation,
        sheet: &mut HashMap<String, AnimationPhase>,
        messages: &mut Vec<(Scalar, String)>,
        mut time: Scalar,
    ) -> (Scalar, bool) {
        match animation {
            Animation::Value(value) => {
                let duration = value.duration.max(0.0);
                let phase = AnimationPhase {
                    start: time,
                    duration,
                    cached_time: 0.0,
                    cached_progress: 0.0,
                };
                sheet.insert(value.name, phase);
                (time + duration, false)
            }
            Animation::Sequence(anims) => {
                for anim in anims {
                    time = Self::include_animation(anim, sheet, messages, time).0;
                }
                (time, false)
            }
            Animation::Parallel(anims) => {
                let mut result = time;
                for anim in anims {
                    result = Self::include_animation(anim, sheet, messages, time)
                        .0
                        .max(result);
                }
                (result, false)
            }
            Animation::Looped(anim) => {
                let looped = sheet.is_empty();
                time = Self::include_animation(*anim, sheet, messages, time).0;
                (time, looped)
            }
            Animation::TimeShift(v) => ((time - v).max(0.0), false),
            Animation::Message(message) => {
                messages.push((time, message));
                (time, false)
            }
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct AnimationPhase {
    #[serde(default)]
    pub start: Scalar,
    #[serde(default)]
    pub duration: Scalar,
    #[serde(default)]
    pub cached_time: Scalar,
    #[serde(default)]
    pub cached_progress: Scalar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Animation {
    Value(AnimatedValue),
    Sequence(Vec<Animation>),
    Parallel(Vec<Animation>),
    Looped(Box<Animation>),
    TimeShift(Scalar),
    Message(String),
}

impl Default for Animation {
    fn default() -> Self {
        Self::TimeShift(0.0)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AnimatedValue {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub duration: Scalar,
}

#[derive(Debug, Default, Clone)]
pub struct AnimationMessage(pub String);
implement_message_data!(AnimationMessage);

#[cfg(test)]
mod tests {
    use super::*;
    use std::{str::FromStr, sync::mpsc::channel};

    #[test]
    fn test_animator() {
        let animation = Animation::Sequence(vec![
            Animation::Value(AnimatedValue {
                name: "fade-in".to_owned(),
                duration: 0.2,
            }),
            Animation::Value(AnimatedValue {
                name: "delay".to_owned(),
                duration: 0.6,
            }),
            Animation::Value(AnimatedValue {
                name: "fade-out".to_owned(),
                duration: 0.2,
            }),
            Animation::Message("next".to_owned()),
        ]);
        println!("Animation: {:#?}", animation);
        let mut states = AnimatorStates::new("".to_owned(), animation);
        println!("States 0: {:#?}", states);
        let id = WidgetId::from_str("type:/widget").unwrap();
        let (sender, receiver) = channel();
        let sender = MessageSender::new(sender);
        states.process(0.5, &id, &sender);
        println!("States 1: {:#?}", states);
        states.process(0.6, &id, &sender);
        println!("States 2: {:#?}", states);
        println!(
            "Message: {:#?}",
            receiver
                .try_recv()
                .unwrap()
                .1
                .as_any()
                .downcast_ref::<AnimationMessage>()
                .unwrap()
        );
    }
}
