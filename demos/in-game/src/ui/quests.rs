use crate::model::quests::{Quest, Quests};
use raui::prelude::*;

pub fn quests(_: WidgetContext) -> WidgetNode {
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
                    make_widget!(available_tasks).into(),
                ])
                .listed_slot([
                    make_widget!(tab_plate)
                        .with_props("COMPLETED".to_owned())
                        .into(),
                    make_widget!(completed_tasks).into(),
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

    make_tasks_list("available", quests.available().map(|(_, task)| task))
}

fn completed_tasks(context: WidgetContext) -> WidgetNode {
    let quests = context
        .view_models
        .view_model(Quests::VIEW_MODEL)
        .unwrap()
        .read::<Quests>()
        .unwrap();

    make_tasks_list("completed", quests.completed().map(|(_, task)| task))
}

fn make_tasks_list<'a>(key: &str, tasks: impl Iterator<Item = &'a Quest>) -> WidgetNode {
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
        .listed_slots(
            tasks
                .enumerate()
                .map(|(index, task)| make_task(index, task)),
        )
        .into()
}

fn make_task(index: usize, task: &Quest) -> WidgetNode {
    make_widget!(text_paper)
        .key(index)
        .with_props(TextPaperProps {
            text: task.name.to_owned(),
            variant: "task-name".to_owned(),
            use_main_color: true,
            ..Default::default()
        })
        .into()
}
