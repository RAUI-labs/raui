use crate::render_worker::{
    RenderWorkerDescriptor, RenderWorkerTaskContext, RenderWorkersViewModel,
};
use raui_core::{
    MessageData, Prefab, PropsData, make_widget,
    messenger::MessageData,
    pre_hooks,
    props::PropsData,
    widget::{
        component::{
            ResizeListenerSignal,
            image_box::{ImageBoxProps, image_box},
            use_resize_listener,
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::area::AreaBoxNode,
        utils::Color,
    },
};
use serde::{Deserialize, Serialize};
use spitfire_glow::renderer::GlowTextureFormat;
use std::sync::Arc;

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct CanvasProps {
    #[serde(default)]
    pub color: Color,
    #[serde(default)]
    pub clear: bool,
}

#[derive(MessageData, Debug, Clone)]
pub struct RequestCanvasRedrawMessage;

#[derive(MessageData, Clone)]
pub enum DrawOnCanvasMessage {
    Function(fn(RenderWorkerTaskContext)),
    #[allow(clippy::type_complexity)]
    Generator(Arc<dyn Fn() -> Box<dyn FnOnce(RenderWorkerTaskContext)> + Send + Sync>),
}

impl DrawOnCanvasMessage {
    pub fn function(callback: fn(RenderWorkerTaskContext)) -> Self {
        Self::Function(callback)
    }

    pub fn generator(
        callback: impl Fn() -> Box<dyn FnOnce(RenderWorkerTaskContext)> + Send + Sync + 'static,
    ) -> Self {
        Self::Generator(Arc::new(callback))
    }
}

impl std::fmt::Debug for DrawOnCanvasMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DrawOnCanvasMessage")
            .finish_non_exhaustive()
    }
}

pub fn use_canvas(ctx: &mut WidgetContext) {
    ctx.life_cycle.mount(|mut ctx| {
        let props = ctx.props.read_cloned_or_default::<CanvasProps>();
        let mut workers = ctx
            .view_models
            .view_model_mut(RenderWorkersViewModel::VIEW_MODEL)
            .unwrap()
            .write::<RenderWorkersViewModel>()
            .unwrap();
        workers.add_worker(RenderWorkerDescriptor {
            id: ctx.id.to_string(),
            width: 1,
            height: 1,
            format: GlowTextureFormat::Rgba,
            color: [props.color.r, props.color.g, props.color.b, props.color.a],
        });
        ctx.messenger
            .write(ctx.id.to_owned(), RequestCanvasRedrawMessage);
    });

    ctx.life_cycle.unmount(|mut ctx| {
        let mut workers = ctx
            .view_models
            .view_model_mut(RenderWorkersViewModel::VIEW_MODEL)
            .unwrap()
            .write::<RenderWorkersViewModel>()
            .unwrap();
        workers.remove_worker(ctx.id.as_ref());
    });

    ctx.life_cycle.change(|mut ctx| {
        let props = ctx.props.read_cloned_or_default::<CanvasProps>();
        let mut workers = ctx
            .view_models
            .view_model_mut(RenderWorkersViewModel::VIEW_MODEL)
            .unwrap()
            .write::<RenderWorkersViewModel>()
            .unwrap();
        for msg in ctx.messenger.messages {
            if let Some(ResizeListenerSignal::Change(size)) = msg.as_any().downcast_ref() {
                workers.add_worker(RenderWorkerDescriptor {
                    id: ctx.id.to_string(),
                    width: size.x as u32,
                    height: size.y as u32,
                    format: GlowTextureFormat::Rgba,
                    color: [props.color.r, props.color.g, props.color.b, props.color.a],
                });
                ctx.messenger
                    .write(ctx.id.to_owned(), RequestCanvasRedrawMessage);
            } else if let Some(msg) = msg.as_any().downcast_ref::<DrawOnCanvasMessage>() {
                match msg {
                    DrawOnCanvasMessage::Function(task) => {
                        workers.schedule_task(ctx.id.as_ref(), props.clear, *task);
                    }
                    DrawOnCanvasMessage::Generator(task) => {
                        workers.schedule_task(ctx.id.as_ref(), props.clear, (*task)());
                    }
                }
            }
        }
    });
}

#[pre_hooks(use_resize_listener, use_canvas)]
pub fn canvas(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext { id, idref, key, .. } = context;

    let content = make_widget!(image_box)
        .key(key)
        .maybe_idref(idref.cloned())
        .with_props(ImageBoxProps::image(id))
        .into();

    AreaBoxNode {
        id: id.to_owned(),
        slot: Box::new(content),
    }
    .into()
}
