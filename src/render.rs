use std::borrow::Cow;
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

    pub fn render(&self) -> Result<(), anyhow::Error> {
        let desc = wgpu::wgt::CommandEncoderDescriptor { label: Some("Begin encode").map(Cow::Borrowed) };
        let (encoder_id, err) = self.context.instance.0.device_create_command_encoder(self.context.device.id, &desc, None);
        let encoder = self.context.instance.as_auto_drop(encoder_id);
        if let Some(err) = err { anyhow::bail!("{err}") }

        let surface_texture = self.context.instance.0.surface_get_current_texture(self.surface.id, None)?;
        let Some(texture_id) = surface_texture.texture else { anyhow::bail!("Surface is not configured (cause: {:?}", surface_texture.status) };
        let desc = wgpu::wgc::resource::TextureViewDescriptor::default();
        let (view_id, err) = self.context.instance.0.texture_create_view(texture_id, &desc, None);
        let view = self.context.instance.as_auto_drop(view_id);
        if let Some(err) = err { anyhow::bail!("{err}") }

        let desc = wgpu::wgc::command::RenderPassDescriptor {
            label: Some("Render pass").map(Cow::Borrowed),
            color_attachments: Cow::Borrowed(&[
                Some(wgpu::wgc::command::RenderPassColorAttachment {
                    view: view.id,
                    depth_slice: None,
                    resolve_target: None,
                    load_op: wgpu::wgc::command::LoadOp::Clear(wgpu::wgt::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                    store_op: wgpu::wgc::command::StoreOp::Store,
                })
            ]),
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        };

        let (mut pass, err) = self.context.instance.0.command_encoder_begin_render_pass(encoder.id, &desc);
        if let Some(err) = err { anyhow::bail!("{err}") }
        self.context.instance.0.render_pass_end(&mut pass)?;

        let desc = wgpu::wgt::CommandBufferDescriptor { label: Some("Finish encode").map(Cow::Borrowed) };
        let (buffer_id, err) = self.context.instance.0.command_encoder_finish(encoder.id, &desc, None);
        let buffer = self.context.instance.as_auto_drop(buffer_id);
        if let Some((msg, err)) = err { anyhow::bail!("{msg} (cause: {err})") }

        match self.context.instance.0.queue_submit(self.context.queue.id, &[buffer.id]) {
            Ok(_) => self.context.instance.0.surface_present(self.surface.id)?,
            Err((index, err)) => anyhow::bail!("{err} @ {index}"),
        };

        Ok(())
    }
}
