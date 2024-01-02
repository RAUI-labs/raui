use crate::model::quests::{Quest, Quests};
use raui::prelude::*;

#[pre_hooks(use_quests)]
pub fn quests(mut context: WidgetContext) -> WidgetNode {
    make_widget!(window_paper)
        .key("quests")
        .with_props(WindowPaperProps {
            bar_margin: 20.0.into(),
            bar_height: Some(80.0),
            content_margin: 40.0.into(),
            ..Default::default()
        })
        .named_slot(
            "bar",
            make_widget!(text_paper)
                .key("title")
                .with_props(TextPaperProps {
                    text: "QUESTS".to_owned(),
                    variant: "title".to_owned(),
                    use_main_color: true,
                    ..Default::default()
                }),
        )
        .named_slot(
            "content",
            make_widget!(nav_tabs_box)
                .key("tabs")
                .with_props(NavItemActive)
                .with_props(TabsBoxProps {
                    tabs_basis: Some(64.0),
                    tabs_and_content_separation: 20.0,
                    ..Default::default()
                })
                .listed_slot([
                    make_widget!(tab_plate)
                        .with_props("AVAILABLE".to_owned())
                        .into(),
                    make_widget!(available_tasks)
                        .with_props(ButtonNotifyProps(context.id.to_owned().into()))
                        .into(),
                ])
                .listed_slot([
                    make_widget!(tab_plate)
                        .with_props("COMPLETED".to_owned())
                        .into(),
                    make_widget!(completed_tasks)
                        .with_props(ButtonNotifyProps(context.id.to_owned().into()))
                        .into(),
                ]),
        )
        .into()
}

fn tab_plate(context: WidgetContext) -> WidgetNode {
    let WidgetContext { props, .. } = context;
    let active = props.read_cloned_or_default::<TabPlateProps>().active;
    let text = props.read_cloned_or_default::<String>();

    make_widget!(content_box)
        .key("plate")
        .maybe_listed_slot(active.then(|| {
            make_widget!(image_box)
                .key("background")
                .with_props(ImageBoxProps::colored(Default::default()))
        }))
        .listed_slot(
            make_widget!(text_paper)
                .key("text")
                .with_props(TextPaperProps {
                    text,
                    variant: "tab-label".to_owned(),
                    use_main_color: !active,
                    ..Default::default()
                }),
        )
        .into()
}

fn available_tasks(context: WidgetContext) -> WidgetNode {
    let quests = context
        .view_models
        .view_model(Quests::VIEW_MODEL)
        .unwrap()
        .read::<Quests>()
        .unwrap();
    let notify = context.props.read_cloned_or_default::<ButtonNotifyProps>();

    make_tasks_list(notify, "available", quests.available())
}

fn completed_tasks(context: WidgetContext) -> WidgetNode {
    let quests = context
        .view_models
        .view_model(Quests::VIEW_MODEL)
        .unwrap()
        .read::<Quests>()
        .unwrap();
    let notify = context.props.read_cloned_or_default::<ButtonNotifyProps>();

    make_tasks_list(notify, "completed", quests.completed())
}

fn make_tasks_list<'a>(
    notify: ButtonNotifyProps,
    key: &str,
    tasks: impl Iterator<Item = (&'a str, &'a Quest)>,
) -> WidgetNode {
    make_widget!(vertical_box)
        .key(key)
        .with_props(VerticalBoxProps {
            override_slots_layout: Some(FlexBoxItemLayout {
                basis: Some(48.0),
                grow: 0.0,
                shrink: 0.0,
                ..Default::default()
            }),
            separation: 10.0,
            ..Default::default()
        })
        .listed_slots(tasks.enumerate().map(|(index, (id, task))| {
            make_widget!(quest_task)
                .key(format!("{}?item={}", index, id))
                .with_props(task.name.to_owned())
                .with_props(notify.to_owned())
        }))
        .into()
}

fn quest_task(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;

    let notify = props.read_cloned::<ButtonNotifyProps>().ok();
    let name = props.read_cloned_or_default::<String>();

    make_widget!(text_button_paper)
        .key(key)
        .maybe_with_props(notify)
        .with_props(NavItemActive)
        .with_props(TextPaperProps {
            text: name,
            variant: "task-name".to_owned(),
            ..Default::default()
        })
        .into()
}

fn use_quests(context: &mut WidgetContext) {
    context.life_cycle.mount(|mut context| {
        context
            .view_models
            .bindings(Quests::VIEW_MODEL, Quests::COMPLETED)
            .unwrap()
            .bind(context.id.to_owned());
    });

    context.life_cycle.change(|mut context| {
        let mut quests = context
            .view_models
            .view_model_mut(Quests::VIEW_MODEL)
            .unwrap()
            .write::<Quests>()
            .unwrap();

        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if let Some(id) = WidgetIdMetaParams::new(msg.sender.meta()).find_value("item") {
                    if msg.trigger_start() {
                        quests.toggle(id);
                    }
                }
            }
        }
    });
}
