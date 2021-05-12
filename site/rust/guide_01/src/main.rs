use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;

fn main() {
    // Create our quick start builder
    RauiQuickStartBuilder::default()
        // Set our window title
        .window_title("RAUI Guide".into())
        // Set the RAUI widget tree for our app
        .widget_tree(widget! {
            // For now we have no widgets!
            ()
        })
        // Build the app
        .build()
        .expect("Error building quick start")
        // And run it! 🚀
        .run()
        .expect("Error running RAUI app");
}
