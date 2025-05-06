use raui_core::{
    application::Application,
    interactive::default_interactions_engine::{
        DefaultInteractionsEngine, Interaction, PointerButton,
    },
    layout::{CoordsMapping, default_layout_engine::DefaultLayoutEngine},
    make_widget,
    widget::{
        component::{
            containers::content_box::nav_content_box,
            image_box::image_box,
            interactive::{button::button, navigation::NavItemActive},
        },
        setup,
        utils::{Rect, Vec2},
    },
};
use raui_json_renderer::JsonRenderer;

fn main() {
    // Create the application
    let mut application = Application::default();

    // We need to run the "setup" functions for the application to register components and
    // properties if we want to support serialization of the UI. We pass it a function that
    // will do the actual registration
    application.setup(setup);
    // application.setup(raui_material::setup /* and the raui_material setup if we need it */);

    // Create the renderer. In this case we render the UI to JSON for simplicity, but usually
    // you would have a custom renderer for your game engine or renderer.
    let mut renderer = JsonRenderer { pretty: true };

    // Create the interactions engine. The default interactions engine covers typical
    // pointer + keyboard + gamepad navigation/interactions.
    let mut interactions = DefaultInteractionsEngine::default();

    // We create our widget tree
    let tree = make_widget!(nav_content_box).key("app").listed_slot(
        make_widget!(button)
            .key("button")
            .with_props(NavItemActive)
            .named_slot("content", make_widget!(image_box).key("icon")),
    );

    // We apply the tree to the application. This must be done again if we wish to change the
    // tree.
    application.apply(tree);

    // This scope content would need to be called every frame
    {
        // Telling the app to `process` will make it perform any necessary updates.
        application.process();

        // To properly handle layout we need to create a mapping of the screen coordinates to
        // the RAUI coordinates. We would update this with the size of the window every frame.
        let mapping = CoordsMapping::new(Rect {
            left: 0.0,
            right: 1024.0,
            top: 0.0,
            bottom: 576.0,
        });

        // we interact with UI by sending interaction messages to the engine. You would hook this up
        // to whatever game engine or window event loop to perform the proper interactions when
        // different events are emitted.
        interactions.interact(Interaction::PointerMove(Vec2 { x: 200.0, y: 100.0 }));
        interactions.interact(Interaction::PointerDown(
            PointerButton::Trigger,
            Vec2 { x: 200.0, y: 100.0 },
        ));

        // We apply the application layout.
        // We use the default layout engine, but you could make your own layout engine.
        let mut layout_engine = DefaultLayoutEngine::<()>::default();
        application.layout(&mapping, &mut layout_engine).unwrap();

        // Since interactions engines require constructed layout to process interactions we
        // have to process interactions after we layout the UI.
        application.interact(&mut interactions).unwrap();

        // Now we render the app
        println!(
            "{}",
            application
                .render::<_, String, _>(&mapping, &mut renderer)
                .unwrap()
        );
    }
}
