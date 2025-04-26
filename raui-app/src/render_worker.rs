use crate::{AssetsManager, Vertex};
use spitfire_glow::{
    graphics::{Graphics, Shader, Surface},
    renderer::GlowTextureFormat,
};
use std::collections::HashMap;

pub struct RenderWorkerDescriptor {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub format: GlowTextureFormat,
    pub color: [f32; 4],
}

#[derive(Default)]
pub struct RenderWorkersViewModel {
    surfaces: HashMap<String, Surface>,
    commands: Vec<Command>,
}

impl RenderWorkersViewModel {
    pub const VIEW_MODEL: &str = "RenderWorkersViewModel";

    pub fn workers(&self) -> impl Iterator<Item = &str> {
        self.surfaces.keys().map(|s| s.as_str())
    }

    pub fn add_worker(&mut self, worker: RenderWorkerDescriptor) {
        self.commands.push(Command::Create { worker });
    }

    pub fn remove_worker(&mut self, worker: &str) {
        self.commands.push(Command::Remove {
            worker: worker.to_owned(),
        });
    }

    pub fn schedule_task(
        &mut self,
        worker: &str,
        clear: bool,
        task: impl FnOnce(RenderWorkerTaskContext) + 'static,
    ) {
        self.commands.push(Command::Schedule {
            worker: worker.to_owned(),
            clear,
            task: Box::new(task),
        });
    }

    pub(crate) fn maintain(
        &mut self,
        graphics: &mut Graphics<Vertex>,
        assets: &mut AssetsManager,
        colored_shader: &Shader,
        textured_shader: &Shader,
        text_shader: &Shader,
    ) {
        for command in self.commands.drain(..) {
            match command {
                Command::Create { worker } => {
                    let Ok(texture) =
                        graphics.texture(worker.width, worker.height, 1, worker.format, None)
                    else {
                        continue;
                    };
                    let Ok(mut surface) = graphics.surface(vec![texture.clone().into()]) else {
                        continue;
                    };
                    surface.set_color(worker.color);
                    self.surfaces.insert(worker.id.clone(), surface);
                    assets.add_texture(worker.id, texture);
                }
                Command::Remove { worker } => {
                    self.surfaces.remove(&worker);
                    assets.remove_texture(&worker);
                }
                Command::Schedule {
                    worker,
                    clear,
                    task,
                } => {
                    if let Some(surface) = self.surfaces.get(&worker) {
                        let _ = graphics.draw();
                        let _ = graphics.push_surface(surface.clone());
                        let _ = graphics.prepare_frame(clear);
                        (task)(RenderWorkerTaskContext {
                            width: surface.width(),
                            height: surface.height(),
                            format: surface.attachments()[0].texture.format(),
                            graphics,
                            colored_shader,
                            textured_shader,
                            text_shader,
                        });
                        let _ = graphics.draw();
                        let _ = graphics.pop_surface();
                        let _ = graphics.prepare_frame(false);
                    }
                }
            }
        }
    }
}

pub struct RenderWorkerTaskContext<'a> {
    pub width: u32,
    pub height: u32,
    pub format: GlowTextureFormat,
    pub graphics: &'a mut Graphics<Vertex>,
    pub colored_shader: &'a Shader,
    pub textured_shader: &'a Shader,
    pub text_shader: &'a Shader,
}

enum Command {
    Create {
        worker: RenderWorkerDescriptor,
    },
    Remove {
        worker: String,
    },
    Schedule {
        worker: String,
        clear: bool,
        #[allow(clippy::type_complexity)]
        task: Box<dyn FnOnce(RenderWorkerTaskContext)>,
    },
}
