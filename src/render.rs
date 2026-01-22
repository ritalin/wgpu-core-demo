use std::sync::Arc;
use wgpu::{wgc::id::SurfaceId, wgt::SurfaceConfiguration};
use wgpu::wgt::TextureFormat;
use crate::{runtime, wgpu_resource::AutoDropId};

pub struct WgpuRenderer {
    context: Arc<runtime::RenderContext>,
    surface: AutoDropId<SurfaceId>,
    config: SurfaceConfiguration<Vec<TextureFormat>>,
}
impl WgpuRenderer {
    pub fn new(context: Arc<runtime::RenderContext>, surface_id: SurfaceId, (width, height): (u32, u32)) -> Self {
        let mut config = context.config.clone();
        config.width = width;
        config.height = height;

        Self {
            surface: context.instance.as_auto_drop(surface_id),
            config,
            context,
        }
    }

    pub fn request_resize(&mut self, (width, height): (u32, u32)) {
        if (width > 0) && (height > 0) {
            self.config.width = width;
            self.config.height = height;
            let _ = self.context.instance.0.surface_configure(self.surface.id, self.context.device.id, &self.config);
        }
    }
}
