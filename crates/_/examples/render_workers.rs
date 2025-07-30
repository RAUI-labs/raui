// This example shows how to render arbitrary geometry "raw" way into a texture
// that can be used as image in the UI - useful for more demanding rendering.

use raui_app::{
    Vertex,
    app::declarative::DeclarativeApp,
    render_worker::{RenderWorkerDescriptor, RenderWorkerTaskContext, RenderWorkersViewModel},
    third_party::spitfire_glow::{
        graphics::GraphicsBatch,
        renderer::{GlowBlending, GlowTextureFormat, GlowUniformValue},
    },
};
use raui_core::{
    make_widget, pre_hooks,
    widget::{
        component::image_box::{ImageBoxProps, image_box},
        context::WidgetContext,
        node::WidgetNode,
    },
};

fn use_app(ctx: &mut WidgetContext) {
    ctx.life_cycle.mount(|mut ctx| {
        // RenderWorkersViewModel is a special view model that stores render worker
        // surfaces that we can schedule render tasks to.
        let mut workers = ctx
            .view_models
            .view_model_mut(RenderWorkersViewModel::VIEW_MODEL)
            .unwrap()
            .write::<RenderWorkersViewModel>()
            .unwrap();
        // First we add worker with the same id as the widget id, to ensure its
        // uniqueness - we can use whatever id we want, but if workers are
        // intended to be personalized to widgets, it's good to use widget id.
        workers.add_worker(RenderWorkerDescriptor {
            id: ctx.id.to_string(),
            width: 256,
            height: 256,
            format: GlowTextureFormat::Rgba,
            color: [1.0, 1.0, 1.0, 0.0],
        });
        // Once we added worker, we schedule to render its content first time.
        workers.schedule_task(ctx.id.as_ref(), true, render_task);
    });

    ctx.life_cycle.unmount(|mut ctx| {
        let mut workers = ctx
            .view_models
            .view_model_mut(RenderWorkersViewModel::VIEW_MODEL)
            .unwrap()
            .write::<RenderWorkersViewModel>()
            .unwrap();
        // When widget is unmounted, we need to remove the worker,
        // otherwise it will be left in the view model for ever.
        workers.remove_worker(ctx.id.as_ref());
    });

    ctx.life_cycle.change(|mut ctx| {
        let mut workers = ctx
            .view_models
            .view_model_mut(RenderWorkersViewModel::VIEW_MODEL)
            .unwrap()
            .write::<RenderWorkersViewModel>()
            .unwrap();
        // When widget is changed, we need to update the worker surface content.
        workers.schedule_task(ctx.id.as_ref(), true, render_task);
    });
}

#[pre_hooks(use_app)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    // Show rendered worker surface as image with aspect ratio to not stretch it.
    make_widget!(image_box)
        .with_props(ImageBoxProps::image_aspect_ratio(ctx.id.as_ref(), false))
        .into()
}

fn main() {
    DeclarativeApp::simple("Render Workers", make_widget!(app));
}

// Function representing render task that will paint some surface content.
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
            position: [
                ctx.graphics.main_camera.screen_size.x * 0.25,
                ctx.graphics.main_camera.screen_size.y * 0.25,
            ],
            uv: [0.0, 0.0, 0.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex {
            position: [
                ctx.graphics.main_camera.screen_size.x * 0.75,
                ctx.graphics.main_camera.screen_size.y * 0.25,
            ],
            uv: [0.0, 0.0, 0.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            position: [
                ctx.graphics.main_camera.screen_size.x * 0.75,
                ctx.graphics.main_camera.screen_size.y * 0.75,
            ],
            uv: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
        Vertex {
            position: [
                ctx.graphics.main_camera.screen_size.x * 0.25,
                ctx.graphics.main_camera.screen_size.y * 0.75,
            ],
            uv: [0.0, 0.0, 0.0],
            color: [1.0, 1.0, 0.0, 1.0],
        },
    ]);
}
