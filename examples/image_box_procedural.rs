use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

fn main() {
    let tree = make_widget!(image_box).with_props(ImageBoxProps {
        // procedural image material allows to draw custom mesh with dedicated
        // shader either from statics or from file.
        // available static shaders:
        // - `@pass`: simple pass through shader that ignores camera matrix.
        // - `@colored`: shader that applies camera transform and color vertices.
        // - `@textured`: shader that applies camera transform and texture with color vertices.
        // if we want to use shader from files, assuming we have two files:
        // - `path/to/shader.vs`
        // - `path/to/shader.fs`
        // then our id would be: `path/to/shader`.
        material: ImageBoxMaterial::Procedural(
            ImageBoxProcedural::new("@colored")
                // if we tell material to fit vertices into layout rect,
                // then we can define vertices in unit space (0-1 coords).
                .fit_to_rect(true)
                .triangle([
                    ImageBoxProceduralVertex {
                        position: Vec2 { x: 0.0, y: 0.0 },
                        color: Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        },
                        ..Default::default()
                    },
                    ImageBoxProceduralVertex {
                        position: Vec2 { x: 1.0, y: 0.0 },
                        color: Color {
                            r: 0.0,
                            g: 1.0,
                            b: 0.0,
                            a: 1.0,
                        },
                        ..Default::default()
                    },
                    ImageBoxProceduralVertex {
                        position: Vec2 { x: 0.5, y: 1.0 },
                        color: Color {
                            r: 0.0,
                            g: 0.0,
                            b: 1.0,
                            a: 1.0,
                        },
                        ..Default::default()
                    },
                ]),
        ),
        ..Default::default()
    });

    DeclarativeApp::simple("Image Box - Procedural", tree);
}
