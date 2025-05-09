use raui_app::app::immediate::ImmediateApp;
use raui_core::widget::{
    component::{image_box::ImageBoxProps, interactive::navigation::NavItemActive},
    utils::Color,
};
use raui_immediate::{register_access, use_access};
use raui_immediate_widgets::core::{
    containers::nav_content_box,
    image_box,
    interactive::{ImmediateButton, button},
};

pub fn app() {
    nav_content_box((), || {
        clickable_button();
    });
}

pub fn clickable_button() {
    if colored_button().trigger_start() {
        // we use access point to some host data
        let clicked = use_access::<bool>("clicked");

        *clicked.write().unwrap() = true;
    }
}

fn colored_button() -> ImmediateButton {
    button(NavItemActive, |state| {
        let props = ImageBoxProps::colored(if state.state.trigger {
            Color {
                r: 0.5,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }
        } else {
            Color {
                r: 0.0,
                g: 0.5,
                b: 0.0,
                a: 1.0,
            }
        });

        image_box(props);
    })
}

fn main() {
    let mut clicked = false;

    ImmediateApp::simple("Immediate mode UI - Access and tests", move |_| {
        // here we register access point to some game state
        let _lifetime = register_access("clicked", &mut clicked);

        app();
    });
}

#[test]
fn test_tracked_button() {
    let mut tester = AppCycleTester::new(
        CoordsMapping::new(Rect {
            left: 0.0,
            right: 1024.0,
            top: 0.0,
            bottom: 576.0,
        }),
        ImmediateContext::default(),
    );
    let mut mock = false;

    tester
        .interactions_engine
        .interact(Interaction::PointerDown(
            PointerButton::Trigger,
            [100.0, 100.0].into(),
        ));

    // since RAUI has deferred UI resolution, signal will take
    // few frames to go through declarative layer to immediate
    // layer and then back to user site.
    for _ in 0..4 {
        tester.run_frame(ImmediateApp::test_frame(|| {
            // and here we register access point to mock data
            let _lifetime = register_access("clicked", &mut mock);

            app();
        }));
    }

    assert_eq!(mock, true);
}
