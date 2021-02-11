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
pub struct AnimatorStates(pub HashMap<String, AnimatorState>);

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
    sheet: HashMap<String, AnimationPhase>,
    messages: Vec<(Scalar, String)>,
    time: Scalar,
    duration: Scalar,
}

impl AnimatorState {
    pub fn new(animation: Animation) -> Self {
        let mut sheet = HashMap::new();
        let mut messages = vec![];
        let time = Self::include_animation(animation, &mut sheet, &mut messages, 0.0);
        Self {
            sheet,
            messages,
            time: 0.0,
            duration: time,
        }
    }

    #[inline]
    pub fn in_progress(&self) -> bool {
        self.time < self.duration && (!self.sheet.is_empty() || !self.messages.is_empty())
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
            self.time += delta_time;
            for phase in self.sheet.values_mut() {
                phase.cached_time = (self.time - phase.start).min(phase.duration).max(0.0);
                phase.cached_progress = if phase.duration > 0.0 {
                    phase.cached_time / phase.duration
                } else {
                    0.0
                };
            }
            let messages = std::mem::take(&mut self.messages);
            self.messages = messages
                .into_iter()
                .filter(|(time, message)| {
                    if *time <= self.time {
                        message_sender
                            .write(owner.to_owned(), AnimationMessage(message.to_owned()));
                        false
                    } else {
                        true
                    }
                })
                .collect::<Vec<_>>();
        }
    }

    fn include_animation(
        animation: Animation,
        sheet: &mut HashMap<String, AnimationPhase>,
        messages: &mut Vec<(Scalar, String)>,
        mut time: Scalar,
    ) -> Scalar {
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
                time + duration
            }
            Animation::Sequence(anims) => {
                for anim in anims {
                    time = Self::include_animation(anim, sheet, messages, time);
                }
                time
            }
            Animation::Parallel(anims) => {
                let mut result = time;
                for anim in anims {
                    result = Self::include_animation(anim, sheet, messages, time).max(result);
                }
                result
            }
            Animation::TimeShift(v) => (time - v).max(0.0),
            Animation::Message(message) => {
                messages.push((time, message));
                time
            }
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct AnimationPhase {
    pub start: Scalar,
    pub duration: Scalar,
    pub cached_time: Scalar,
    pub cached_progress: Scalar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Animation {
    Value(AnimatedValue),
    Sequence(Vec<Animation>),
    Parallel(Vec<Animation>),
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
    pub name: String,
    pub duration: Scalar,
}

#[derive(Debug, Default, Clone)]
pub struct AnimationMessage(String);
