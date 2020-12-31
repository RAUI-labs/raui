pub mod app;
pub mod inventory;
pub mod item_cell;
pub mod minimap;
pub mod popup;

use raui_core::prelude::*;
use raui_material::prelude::*;

pub fn new_theme() -> ThemeProps {
    let mut theme = new_all_white_theme();
    theme.content_backgrounds.insert(
        String::new(),
        ThemedImageMaterial::Image(ImageBoxImage {
            id: "panel".to_owned(),
            scaling: ImageBoxImageScaling::Frame(24.0, false),
            ..Default::default()
        }),
    );
    theme.content_backgrounds.insert(
        "frame".to_owned(),
        ThemedImageMaterial::Image(ImageBoxImage {
            id: "frame".to_owned(),
            scaling: ImageBoxImageScaling::Frame(24.0, false),
            ..Default::default()
        }),
    );
    theme.content_backgrounds.insert(
        "cell".to_owned(),
        ThemedImageMaterial::Image(ImageBoxImage {
            id: "cell".to_owned(),
            scaling: ImageBoxImageScaling::Frame(3.0, false),
            ..Default::default()
        }),
    );
    theme.content_backgrounds.insert(
        "framed-cell".to_owned(),
        ThemedImageMaterial::Image(ImageBoxImage {
            id: "framed-cell".to_owned(),
            scaling: ImageBoxImageScaling::Frame(4.0, false),
            ..Default::default()
        }),
    );
    theme
}
