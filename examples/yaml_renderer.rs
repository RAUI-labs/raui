use raui::prelude::*;
use raui::renderer::yaml::YamlRenderer;

fn main() {
    // Create the application
    let mut application = Application::new();

    // We need to run the "setup" functions for the application to register components and
    // properties if we want to support serialization of the UI. We pass it a function that
    // will do the actual registration
    application.setup(setup /* the core setup function, in the raui prelude */);
    // application.setup(raui_material::setup /* and the raui_material setup if we need it */);

    // Create the renderer. In this case we render the UI to YAML for simplicity, but usually
    // you would have a custom renderer for your game engine or renderer.
    let mut renderer = YamlRenderer;

    // Create the interactions engine. The default interactions engine covers typical
    // pointer + keyboard + gamepad navigation/interactions.
    let mut interactions = DefaultInteractionsEngine::new();

    // We create our widget tree
    let tree = widget! {
        (#{"app"} nav_content_box [
            (#{"button"} button: {NavItemActive} {
                content = (#{"icon"} image_box)
            })
        ])
    };

    // We apply the tree to the application. This must be done again if we wish to change the
    // tree.
    application.apply(tree);

    // This and the following function calls would need to be called every frame
    loop {
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

        // We apply the application layout
        application
            // We use the default layout engine, but you could make your own layout engine
            .layout(&mapping, &mut DefaultLayoutEngine)
            .unwrap();

        // Since interactions engines require constructed layout to process interactions we
        // have to process interactions after we layout the UI.
        application.interact(&mut interactions).unwrap();

        // Now we render the app
        println!("{}", application.render(&mapping, &mut renderer).unwrap());

        // Let's not actually loop infinitely for this example
        break;
    }
}
