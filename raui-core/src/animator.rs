use crate::{messenger::MessageSender, widget::WidgetId, Scalar};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum AnimationUpdateAction {
    None,
    Stop,
    Start(Animation),
}

impl Default for AnimationUpdateAction {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Default, Clone)]
pub struct AnimationUpdate(AnimationUpdateAction);

impl AnimationUpdate {
    pub fn change(&mut self, animation: Option<Animation>) {
        self.0 = match animation {
            Some(anim) => AnimationUpdateAction::Start(anim),
            None => AnimationUpdateAction::Stop,
        };
    }
}

pub struct Animator<'a> {
    state: Option<&'a AnimatorState>,
    update: AnimationUpdate,
}

impl<'a> Animator<'a> {
    #[inline]
    pub fn new(state: Option<&'a AnimatorState>, update: AnimationUpdate) -> Self {
        Self { state, update }
    }

    #[inline]
    pub fn change(&mut self, animation: Option<Animation>) {
        self.update.change(animation)
    }

    #[inline]
    pub fn value(&self, name: &str) -> Option<(Scalar, Scalar, Scalar)> {
        match &self.state {
            Some(state) => state.value(name),
            None => None,
        }
    }
}

impl<'a> Into<AnimationUpdateAction> for Animator<'a> {
    fn into(self) -> AnimationUpdateAction {
        self.update.0
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
