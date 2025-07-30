// Make sure you have seen `render_workers` code example first, because this is an evolution of that.

use raui_app::{
    Vertex,
    app::declarative::DeclarativeApp,
    components::canvas::{CanvasProps, DrawOnCanvasMessage, RequestCanvasRedrawMessage, canvas},
    render_worker::RenderWorkerTaskContext,
    third_party::spitfire_glow::{
        graphics::GraphicsBatch,
        renderer::{GlowBlending, GlowUniformValue},
    },
};
use raui_core::{
    make_widget, pre_hooks,
    widget::{context::WidgetContext, node::WidgetNode, utils::Color},
};

fn use_my_canvas(ctx: &mut WidgetContext) {
    ctx.life_cycle.change(|ctx| {
        // canvas will send redraw request on mount and resize.
        // we can react with sending drawing task message to canvas.
        for msg in ctx.messenger.messages {
            if msg
                .as_any()
                .downcast_ref::<RequestCanvasRedrawMessage>()
                .is_some()
            {
                ctx.messenger.write(
                    ctx.id.to_owned(),
                    DrawOnCanvasMessage::function(render_task),
                );
            }
        }
    });
}

#[pre_hooks(use_my_canvas)]
fn my_canvas(mut ctx: WidgetContext) -> WidgetNode {
    // we are specializing canvas widget by simply executing canvas
    // widget function in place, so we have easier times sending
    // drawing task to canvas by its id.
    canvas(ctx)
}

fn main() {
    let tree = make_widget!(my_canvas).with_props(CanvasProps {
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.5,
        },
        clear: true,
    });

    DeclarativeApp::simple("Canvas", tree);
}

fn render_task(ctx: RenderWorkerTaskContext) {
    ctx.graphics.stream.batch_optimized(GraphicsBatch {
        shader: Some(ctx.colored_shader.clone()),
        uniforms: [(
            "u_projection_view".into(),
            GlowUniformValue::M4(ctx.graphics.main_camera.world_matrix().into_col_array()),
        )]
        .into_iter()
        .collect(),
        textures: Default::default(),
        blending: GlowBlending::Alpha,
        scissor: None,
        wireframe: false,
    });
    ctx.graphics.stream.quad([
        Vertex {
            position: [50.0, 50.0],
            uv: [0.0, 0.0, 0.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex {
            position: [ctx.graphics.main_camera.screen_size.x - 50.0, 50.0],
            uv: [0.0, 0.0, 0.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            position: [
                ctx.graphics.main_camera.screen_size.x - 50.0,
                ctx.graphics.main_camera.screen_size.y - 50.0,
            ],
            uv: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
        Vertex {
            position: [50.0, ctx.graphics.main_camera.screen_size.y - 50.0],
            uv: [0.0, 0.0, 0.0],
            color: [1.0, 1.0, 0.0, 1.0],
        },
    ]);
}
