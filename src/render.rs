use std::borrow::Cow;
use std::sync::Arc;
use wgpu::BufferSize;
use wgpu::wgc::id::BufferId;
use wgpu::{wgc::id::SurfaceId, wgt::SurfaceConfiguration};
use wgpu::wgt::TextureFormat;
use crate::{runtime, wgpu_resource::AutoDropId};

pub struct WgpuRenderer {
    context: Arc<runtime::RenderContext>,
    surface: AutoDropId<SurfaceId>,
    config: SurfaceConfiguration<Vec<TextureFormat>>,
    vertex_buffer: AutoDropId<BufferId>,
}
impl WgpuRenderer {
    pub fn new(context: Arc<runtime::RenderContext>, surface_id: SurfaceId, (width, height): (u32, u32)) -> Result<Self, anyhow::Error> {
        let mut config = context.config.clone();
        config.width = width;
        config.height = height;

        let vertex_size = (crate::VERTEXIES.len() * size_of::<crate::Vertex>()) as u64;

        let desc = wgpu::wgt::BufferDescriptor {
            label: Some("Vertex buffer").map(Cow::Borrowed),
            size: vertex_size,
            mapped_at_creation: false, // For staging copy
            usage: wgpu::wgt::BufferUsages::VERTEX | wgpu::wgt::BufferUsages::COPY_DST,
        };
        let (buffer_id, err) = context.instance.0.device_create_buffer(context.device.id, &desc, None);
        let vbuffer = context.instance.as_auto_drop(buffer_id);
        if let Some(err) = err { anyhow::bail!("{err}") }

        Ok(Self {
            surface: context.instance.as_auto_drop(surface_id),
            config,
            context,
            vertex_buffer: vbuffer,
        })
    }

    pub fn request_resize(&mut self, (width, height): (u32, u32)) {
        if (width > 0) && (height > 0) {
            self.config.width = width;
            self.config.height = height;
            let _ = self.context.instance.0.surface_configure(self.surface.id, self.context.device.id, &self.config);
        }
    }

    #[track_caller]
    pub fn render(&mut self) -> Result<(), anyhow::Error> {
        // copy vertex data
        let source = bytemuck::cast_slice(crate::VERTEXIES);
        let vertex_len = crate::VERTEXIES.len() as u32;
        let vertex_size = BufferSize::new((crate::VERTEXIES.len() * size_of::<crate::Vertex>()) as u64).unwrap();

        let (vertex_staging_id, vertex_staging_offset) = self.context.instance.0.queue_create_staging_buffer(
            self.context.queue.id,
            vertex_size,
            None
        )?;

        let slice = unsafe { std::slice::from_raw_parts_mut(vertex_staging_offset.as_ptr(), source.len()) };
        slice.copy_from_slice(source);
        self.context.instance.0.queue_write_staging_buffer(
            self.context.queue.id,
            self.vertex_buffer.id,
            0, // dst offset
            vertex_staging_id
        )?;

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
        self.context.instance.0.render_pass_set_pipeline(&mut pass, self.context.pipeline.id)?;
        self.context.instance.0.render_pass_set_vertex_buffer(&mut pass, 0, self.vertex_buffer.id, 0, None)?; // offset <- vertex buffer offset, size <- vertex buffer size
        self.context.instance.0.render_pass_draw(&mut pass, vertex_len, 1, 0, 0)?;
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
