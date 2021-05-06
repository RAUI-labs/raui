//! Animation engine
//!
//! RAUI widget components can be animated by updating and adding animations using the [`Animator`]
//! inside of widget lifecycle hooks and by reading the progress of those animations from the
//! [`AnimatorStates`] provided by the [`WidgetContext`].
//!
//! See [`Animator`] and [`AnimatorStates`] for code samples.
//!
//! [`WidgetContext`]: crate::widget::context::WidgetContext
use crate::{messenger::MessageSender, widget::WidgetId, MessageData, Scalar};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::mpsc::Sender};

/// An error that may occur when animating a value
pub enum AnimationError {
    CouldNotReadData,
    CouldNotWriteData,
}

/// Handle to an animation sending channel used internally to update widget animations values in
/// lifecycle hooks
#[derive(Clone)]
pub(crate) struct AnimationUpdate(Sender<(String, Option<Animation>)>);

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

/// Allows manipulating widget animations
///
/// An [`Animator`] can be used inside of the [`WidgetMountOrChangeContext`] that is provided when
/// setting widget lifecycle handlers.
///
/// # Example
///
/// ```
/// # use raui_core::prelude::*;
/// fn my_widget(context: WidgetContext) -> WidgetNode {
///     // When my_widget changes
///     context.life_cycle.change(|change_context| {
///         // Get the `Animator`
///         let animator = change_context.animator;
///
///         // Stop "my_animation"
///         animator.change("my_animation", None);
///     });
///
///     widget! { () }
/// }
/// ```
///
/// # Animations & Values
///
/// The animator can manage any number of different animations identified by a string `anim_id`.
/// Additionally each animation can have more than one _value_ that is animated and each of these
/// values has a `value_name` that can be used to get the animated value.
///
/// [`WidgetMountOrChangeContext`]: crate::widget::context::WidgetMountOrChangeContext
pub struct Animator<'a> {
    states: &'a AnimatorStates,
    update: AnimationUpdate,
}

impl<'a> Animator<'a> {
    /// Create a new [`Animator`]
    #[inline]
    pub(crate) fn new(states: &'a AnimatorStates, update: AnimationUpdate) -> Self {
        Self { states, update }
    }

    /// Check whether or not the widget has an animation with the given `anim_id`
    #[inline]
    pub fn has(&self, anim_id: &str) -> bool {
        self.states.has(anim_id)
    }

    /// Change the animation associated to a given `anim_id`
    #[inline]
    pub fn change(
        &self,
        anim_id: &str,
        animation: Option<Animation>,
    ) -> Result<(), AnimationError> {
        self.update.change(anim_id, animation)
    }

    /// Get the current progress of the animation of a given value
    ///
    /// This will return [`None`] if the value is not currently being animated.
    #[inline]
    pub fn value_progress(&self, anim_id: &str, value_name: &str) -> Option<AnimatedValueProgress> {
        self.states.value_progress(anim_id, value_name)
    }

    /// Get the current progress factor of the animation of a given value
    ///
    /// If the value is currently being animated this will return [`Some`] [`Scalar`] between `0`
    /// and `1` with `0` meaning just started and `1` meaning finished.
    ///
    /// If the value is **not** currently being animated [`None`] will be returned
    #[inline]
    pub fn value_progress_factor(&self, anim_id: &str, value_name: &str) -> Option<Scalar> {
        self.states
            .value_progress(anim_id, value_name)
            .map(|x| x.progress_factor)
    }

    /// Same as [`value_progress_factor`][Self::value_progress_factor] but returning `default` instead of [`None`]
    #[inline]
    pub fn value_progress_factor_or(
        &self,
        anim_id: &str,
        value_name: &str,
        default: Scalar,
    ) -> Scalar {
        self.value_progress_factor(anim_id, value_name)
            .unwrap_or(default)
    }

    /// Same as [`value_progress_factor`][Self::value_progress_factor] but returning `0` instead of [`None`]
    #[inline]
    pub fn value_progress_factor_or_zero(&self, anim_id: &str, value_name: &str) -> Scalar {
        self.value_progress_factor(anim_id, value_name)
            .unwrap_or(0.)
    }
}

/// The amount of progress made for a value in an animation
pub struct AnimatedValueProgress {
    /// How far along this animation is from 0 to 1
    pub progress_factor: Scalar,
    /// The amount of time this animation has been running
    pub time: Scalar,
    /// The amount of time that this animation will run for
    pub duration: Scalar,
}

/// The current state of animations in a component
///
/// The [`AnimatorStates`] can be accessed from the [`WidgetContext`] to get information about the
/// current state of all component animations.
///
/// # Example
///
/// ```
/// # use raui_core::prelude::*;
/// # fn my_button(_: WidgetContext) -> WidgetNode { widget!(()) }
/// fn my_widget(context: WidgetContext) -> WidgetNode {
///     // Get the animator from our context
///     let WidgetContext { animator, .. } = context;
///     
///     // Create the properties for a size box
///     let size_box_props = Props::new(SizeBoxProps {
///         transform: Transform {
///             // Get the `scale` value of the `my_anim` animation or and
///             // scale our button based on the animation progress
///             scale: Vec2::from(animator.value_progress_factor_or("my_anim", "scale", 1.)),
///             ..Default::default()
///         },
///         ..Default::default()
///     });
///
///     // Wrap our button in our animated size box
///     widget! { (size_box: {size_box_props} {
///         content = (my_button)
///     }) }
/// }
/// ```
///
/// # Animations & Values
///
/// A component may have any number of different animations identified by a string `anim_id`.
/// Additionally each animation can have more than one _value_ that is animated and each of these
/// values has a `value_name` that can be used to get the animated value.
///
/// [`WidgetContext`]: crate::widget::context::WidgetContext
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AnimatorStates(
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub HashMap<String, AnimatorState>,
);

impl AnimatorStates {
    /// Initialize a new [`AnimatorStates`] that contains a single animation
    pub(crate) fn new(anim_id: String, animation: Animation) -> Self {
        let mut result = HashMap::with_capacity(1);
        result.insert(anim_id, AnimatorState::new(animation));
        Self(result)
    }

    /// Returns whether or not _any_ of the animations for this component are in-progress
    pub fn in_progress(&self) -> bool {
        self.0.values().any(|s| s.in_progress())
    }

    /// Returns `true` if none of this component's animations are currently running
    #[inline]
    pub fn is_done(&self) -> bool {
        !self.in_progress()
    }

    /// Returns true if the widget has an animation with the given `anim_id`
    #[inline]
    pub fn has(&self, anim_id: &str) -> bool {
        self.0.contains_key(anim_id)
    }

    /// Get the current progress of the animation of a given value
    ///
    /// This will return [`None`] if the value is not currently being animated.
    #[inline]
    pub fn value_progress(&self, anim_id: &str, value_name: &str) -> Option<AnimatedValueProgress> {
        if let Some(state) = self.0.get(anim_id) {
            state.value_progress(value_name)
        } else {
            None
        }
    }

    /// Get the current progress factor of the animation of a given value
    ///
    /// If the value is currently being animated this will return [`Some`] [`Scalar`] between `0`
    /// and `1` with `0` meaning just started and `1` meaning finished.
    ///
    /// If the value is **not** currently being animated [`None`] will be returned
    #[inline]
    pub fn value_progress_factor(&self, anim_id: &str, value_name: &str) -> Option<Scalar> {
        if let Some(state) = self.0.get(anim_id) {
            state.value_progress_factor(value_name)
        } else {
            None
        }
    }

    /// Same as [`value_progress_factor`][Self::value_progress_factor] but returning `default` instead of [`None`]
    #[inline]
    pub fn value_progress_factor_or(
        &self,
        anim_id: &str,
        value_name: &str,
        default: Scalar,
    ) -> Scalar {
        self.value_progress_factor(anim_id, value_name)
            .unwrap_or(default)
    }

    /// Same as [`value_progress_factor`][Self::value_progress_factor] but returning `0` instead of [`None`]
    #[inline]
    pub fn value_progress_factor_or_zero(&self, anim_id: &str, value_name: &str) -> Scalar {
        self.value_progress_factor(anim_id, value_name)
            .unwrap_or(0.)
    }

    /// Update the animation with the given `anim_id`
    ///
    /// If `animation` is [`None`] the animation will be removed.
    pub fn change(&mut self, anim_id: String, animation: Option<Animation>) {
        if let Some(animation) = animation {
            self.0.insert(anim_id, AnimatorState::new(animation));
        } else {
            self.0.remove(&anim_id);
        }
    }

    /// Processes the animations, updating the values of each animation baed on the progressed time
    pub(crate) fn process(
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

/// The state of a single animation in a component
///
/// This is most often accessed though [`AnimatorStates`] in the [`WidgetContext`].
///
/// [`WidgetContext`]: crate::widget::context::WidgetContext
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
    /// Initialize a new [`AnimatorState`] given an animation
    pub(crate) fn new(animation: Animation) -> Self {
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

    /// Returns whether or not the animations is in-progress
    #[inline]
    pub fn in_progress(&self) -> bool {
        self.looped || (self.time <= self.duration && !self.sheet.is_empty())
    }

    /// Returns `true` if this animation is not in-progress
    #[inline]
    pub fn is_done(&self) -> bool {
        !self.in_progress()
    }

    /// Get the current progress of the animation of a given value
    ///
    /// This will return [`None`] if the value is not currently being animated.
    #[inline]
    pub fn value_progress(&self, name: &str) -> Option<AnimatedValueProgress> {
        self.sheet.get(name).map(|p| AnimatedValueProgress {
            progress_factor: p.cached_progress,
            time: p.cached_time,
            duration: p.duration,
        })
    }

    /// Get the current progress factor of the animation of a given value
    ///
    /// If the value is currently being animated this will return [`Some`] [`Scalar`] between `0`
    /// and `1` with `0` meaning just started and `1` meaning finished.
    ///
    /// If the value is **not** currently being animated [`None`] will be returned
    #[inline]
    pub fn value_progress_factor(&self, name: &str) -> Option<Scalar> {
        self.sheet.get(name).map(|p| p.cached_progress)
    }

    /// Same as [`value_progress_factor`][Self::value_progress_factor] but returning `default` instead of [`None`]
    #[inline]
    pub fn value_progress_factor_or(&self, name: &str, default: Scalar) -> Scalar {
        self.value_progress_factor(name).unwrap_or(default)
    }

    /// Same as [`value_progress_factor`][Self::value_progress_factor] but returning `0` instead of [`None`]
    #[inline]
    pub fn value_progress_factor_or_zero(&self, name: &str) -> Scalar {
        self.value_progress_factor(name).unwrap_or(0.)
    }

    /// Processes the animations, updating the values of each animation baed on the progressed time
    pub(crate) fn process(
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

    // Add an animation to this [`AnimatorState`] recursively
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

/// Defines a widget animation
///
/// [`Animation`]'s can be added to widget component's [`AnimatorStates`] to animate values.
///
/// Creating an [`Animation`] doesn't actually animate a specific value, but instead gives you a way
/// to track the _progress_ of an animated value using the
/// [`value_progress`][AnimatorStates::value_progress] function. This allows you to use the progress
/// to calculate how to interpolate the real values when you build your widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Animation {
    /// A single animated value with a name and a duration
    Value(AnimatedValue),
    /// A sequence of animations that will be run in a row
    Sequence(Vec<Animation>),
    /// A set of animations that will be run at the same time
    Parallel(Vec<Animation>),
    /// An animation that will play in a loop
    Looped(Box<Animation>),
    /// TODO: Document `TimeShift`
    TimeShift(Scalar),
    /// Send an [`AnimationMessage`]
    Message(String),
}

impl Default for Animation {
    fn default() -> Self {
        Self::TimeShift(0.0)
    }
}

/// A single, animated value with a name and a duration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AnimatedValue {
    /// The name of the animated value
    ///
    /// This is used to get the progress of the animation value with the
    /// [`value_progress`][AnimatorStates::value_progress] function.
    #[serde(default)]
    pub name: String,
    /// The duration of the animation
    #[serde(default)]
    pub duration: Scalar,
}

/// A [`MessageData`][crate::messenger::MessageData] implementation sent by running an
/// [`Animation::Message`] animation
#[derive(MessageData, Debug, Default, Clone)]
#[message_data(crate::messenger::MessageData)]
pub struct AnimationMessage(pub String);

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
