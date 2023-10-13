// Example of retained mode UI on top of RAUI.
// It's goals are very similar to Unreal's UMG on top of Slate.
// Evolution of this approach allows to use retained mode views
// within declarative mode widgets and vice versa - they
// interleave quite seamingly.

use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;
use raui_retained::*;
use std::any::Any;

const FONT: &str = "./demos/hello-world/resources/verdana.ttf";

// root view of an application.
struct AppView {
    pub counter: View<CounterView>,
    pub increment_button: View<Button<LabelView>>,
    pub decrement_button: View<Button<LabelView>>,
}

impl ViewState for AppView {
    // `on_render` method constructs declarative nodes out of
    // retained node. this is similar to how Unreal's UMG builds
    // Slate widgets tree. you can do here whatever you would do
    // normally in RAUI widget component functions.
    fn on_render(&self, mut context: WidgetContext) -> WidgetNode {
        // as usual, at least root view should produce navigable
        // container to enable navigation on the UI, here navigation
        // being button clicks.
        context.use_hook(use_nav_container_active);

        make_widget!(vertical_box)
            .listed_slot(
                self.counter
                    .component()
                    .key("counter")
                    .with_props(FlexBoxItemLayout {
                        basis: Some(48.0),
                        grow: 0.0,
                        shrink: 0.0,
                        ..Default::default()
                    }),
            )
            .listed_slot(
                make_widget!(horizontal_box)
                    .with_props(FlexBoxItemLayout {
                        basis: Some(48.0),
                        grow: 0.0,
                        shrink: 0.0,
                        ..Default::default()
                    })
                    .with_props(HorizontalBoxProps {
                        separation: 50.0,
                        ..Default::default()
                    })
                    .listed_slot(self.increment_button.component().key("increment"))
                    .listed_slot(self.decrement_button.component().key("decrement")),
            )
            .into()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// counter view stores value that can notify RAUI about this
// widget being changed, so it will get re-renderred.
// if we don't wrap data that has to be observed in `ViewValue`,
// then we would need to find other way to notify RAUI app
// about the change in data whenever it happen, usually manually.
// alternatively we could use View-Model feature as we would
// normally do with RAUI, if we don't want to store host data in
// views (which is always good approach to take).
struct CounterView {
    pub counter: ViewValue<usize>,
}

impl ViewState for CounterView {
    fn on_render(&self, _: WidgetContext) -> WidgetNode {
        make_widget!(text_box)
            // to allow `ViewValue` notify RAUI app about changes,
            // we need to pass its widget ref to RAUI component.
            // `ViewValue` pass that widget id to change notifications.
            .idref(self.counter.widget_ref())
            .with_props(TextBoxProps {
                text: self.counter.to_string(),
                font: TextBoxFont {
                    name: FONT.to_owned(),
                    size: 32.0,
                },
                color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                ..Default::default()
            })
            .into()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct LabelView {
    pub text: String,
}

impl LabelView {
    fn new(text: impl ToString) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl ViewState for LabelView {
    fn on_render(&self, _: WidgetContext) -> WidgetNode {
        make_widget!(text_box)
            .with_props(TextBoxProps {
                text: self.text.to_owned(),
                font: TextBoxFont {
                    name: FONT.to_owned(),
                    size: 32.0,
                },
                color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                ..Default::default()
            })
            .into()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// button that can store `on click` callback that
// gets called whenever RAUI button detects click.
struct Button<T: ViewState> {
    pub content: View<T>,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
}

impl<T: ViewState> Button<T> {
    fn new(content: View<T>) -> Self {
        Self {
            content,
            on_click: None,
        }
    }

    fn on_click(mut self, on_click: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Box::new(on_click));
        self
    }
}

impl<T: ViewState> ViewState for Button<T> {
    fn on_change(&mut self, context: WidgetMountOrChangeContext) {
        // as usual, we listen for button messages sent to this
        // widget and call stored callback.
        if let Some(on_click) = self.on_click.take() {
            for message in context.messenger.messages {
                if let Some(message) = message.as_any().downcast_ref::<ButtonNotifyMessage>() {
                    if message.trigger_start() {
                        on_click();
                    }
                }
            }
            self.on_click = Some(on_click);
        }
    }

    fn on_render(&self, context: WidgetContext) -> WidgetNode {
        make_widget!(button)
            .with_props(NavItemActive)
            // this enables RAUI interaction system to send button
            // events to same widget
            .with_props(ButtonNotifyProps(context.id.to_owned().into()))
            .named_slot(
                "content",
                make_widget!(content_box)
                    .listed_slot(make_widget!(image_box).with_props(ImageBoxProps {
                        material: ImageBoxMaterial::Color(ImageBoxColor {
                            color: Color {
                                r: 0.75,
                                g: 0.75,
                                b: 0.75,
                                a: 1.0,
                            },
                            ..Default::default()
                        }),
                        ..Default::default()
                    }))
                    .listed_slot(self.content.component()),
            )
            .into()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// here we construct application view tree out of view objects.
fn create_app(ui: &mut Application) -> View<AppView> {
    // create counter view and get lazy handles to it for buttons to use.
    // nice thing about lazy views is that they can be shared across
    // entire application - think of them just as handles/references to
    // any view you create, that can be accessed from whatever place.
    let counter = View::new(CounterView {
        counter: ViewValue::new(0).with_notifier(ui.notifier()),
    });
    let lazy_counter_increment = counter.lazy();
    let lazy_counter_decrement = counter.lazy();

    let increment_button = View::new(Button::new(View::new(LabelView::new("Add"))).on_click(
        move || {
            // we can access other views using lazy views.
            *lazy_counter_increment.write().unwrap().counter += 1;
        },
    ));
    let decrement_button = View::new(Button::new(View::new(LabelView::new("Subtract"))).on_click(
        move || {
            let mut access = lazy_counter_decrement.write().unwrap();
            *access.counter = access.counter.saturating_sub(1);
        },
    ));

    View::new(AppView {
        counter,
        increment_button,
        decrement_button,
    })
}

fn main() {
    // we keep screen shared view in application scope
    // to keep views tree alive for entire app lifetime.
    // this somewhat simulates having UI manager storing views tree.
    let mut screen = View::new(SharedView::<AppView>::default());

    RauiQuickStartBuilder::default()
        .window_title("Retained mode UI".to_owned())
        .build()
        .unwrap()
        .run_with(|app| {
            // create app views tree and put it into screen root
            // to extend its lifetime to application.
            screen.write().unwrap().replace(create_app(app));

            // finally send screen widget component to RAUI app.
            app.apply(screen.component().key("screen").into());
        })
        .unwrap();
}
