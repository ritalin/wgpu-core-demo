use std::borrow::Cow;
use std::sync::Arc;
use wgpu::BufferSize;
use wgpu::wgc::id::{BindGroupId, BufferId};
use wgpu::{wgc::id::SurfaceId, wgt::SurfaceConfiguration};
use wgpu::wgt::TextureFormat;
use crate::{runtime, wgpu_resource::AutoDropId};

const IMAGE: &'static [u8] = include_bytes!("../assets/img/happy-tree.png");

pub struct WgpuRenderer {
    context: Arc<runtime::RenderContext>,
    surface: AutoDropId<SurfaceId>,
    config: SurfaceConfiguration<Vec<TextureFormat>>,
    vertex_buffer: AutoDropId<BufferId>,
    index_buffer: AutoDropId<BufferId>,
    image_bind_group: AutoDropId<BindGroupId>,
}
impl WgpuRenderer {
    pub fn new(context: Arc<runtime::RenderContext>, surface_id: SurfaceId, (width, height): (u32, u32)) -> Result<Self, anyhow::Error> {
        let mut config = context.config.clone();
        config.width = width;
        config.height = height;

        let vertex_size = (crate::VERTICES.len() * size_of::<crate::Vertex>()) as u64;

        let desc = wgpu::wgt::BufferDescriptor {
            label: Some("Vertex buffer").map(Cow::Borrowed),
            size: vertex_size,
            mapped_at_creation: false, // For staging copy
            usage: wgpu::wgt::BufferUsages::VERTEX | wgpu::wgt::BufferUsages::COPY_DST,
        };
        let (buffer_id, err) = context.instance.0.device_create_buffer(context.device.id, &desc, None);
        let vbuffer = context.instance.as_auto_drop(buffer_id);
        if let Some(err) = err { anyhow::bail!("{err}") }

        let index_size = (crate::INDICES.len() * size_of::<u32>()) as u64;
        let desc = wgpu::wgt::BufferDescriptor {
            label: Some("Index buffer").map(Cow::Borrowed),
            size: index_size,
            mapped_at_creation: false, // For staging copy
            usage: wgpu::wgt::BufferUsages::INDEX | wgpu::wgt::BufferUsages::COPY_DST,
        };
        let (buffer_id, err) = context.instance.0.device_create_buffer(context.device.id, &desc, None);
        let ibuffer = context.instance.as_auto_drop(buffer_id);
        if let Some(err) = err { anyhow::bail!("{err}") }

        let image_bind_group = {
            let image = image::load_from_memory(IMAGE)?.to_rgba8();
            let dims = image.dimensions();
            let size = wgpu::wgt::Extent3d { width: dims.0, height: dims.1, depth_or_array_layers: 1 };
            let desc = wgpu::wgt::TextureDescriptor {
                label: Some("Diffuse texture").map(Cow::Borrowed),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::wgt::TextureDimension::D2,
                format: wgpu::wgt::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::wgt::TextureUsages::TEXTURE_BINDING | wgpu::wgt::TextureUsages::COPY_DST,
                view_formats: vec![],
            };
            let (texture_id, err) = context.instance.0.device_create_texture(context.device.id, &desc, None);
            let texture = context.instance.as_auto_drop(texture_id);
            if let Some(err) = err { anyhow::bail!("{err}") }

            let dest = wgpu::wgt::TexelCopyTextureInfo {
                texture: texture.id,
                mip_level: 0,
                origin: wgpu::wgt::Origin3d::ZERO,
                aspect: wgpu::wgt::TextureAspect::All,
            };
            let layout = wgpu::wgt::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dims.0),
                rows_per_image: Some(dims.1),
            };
            context.instance.0.queue_write_texture(context.queue.id, &dest, &image, &layout, &size)?;

            let desc = wgpu::wgc::resource::TextureViewDescriptor::default();
            let (view_id, err) = context.instance.0.texture_create_view(texture.id, &desc, None);
            let view = context.instance.as_auto_drop(view_id);
            if let Some(err) = err { anyhow::bail!("{err}") }

            let desc: wgpu::wgt::SamplerDescriptor<Cow<'_, &str>> = wgpu::wgt::SamplerDescriptor::default();
            let desc = wgpu::wgc::resource::SamplerDescriptor {
                label: Some("Diffuse texture sampler").map(Cow::Borrowed),
                address_modes: [
                    wgpu::wgt::AddressMode::ClampToEdge, // u
                    wgpu::wgt::AddressMode::ClampToEdge, // v
                    wgpu::wgt::AddressMode::ClampToEdge, // w
                ],
                mag_filter: wgpu::wgt::FilterMode::Linear,
                min_filter: wgpu::wgt::FilterMode::Nearest,
                mipmap_filter: wgpu::wgt::MipmapFilterMode::Nearest,
                lod_min_clamp: desc.lod_min_clamp,
                lod_max_clamp: desc.lod_max_clamp,
                compare: desc.compare,
                anisotropy_clamp: desc.anisotropy_clamp,
                border_color: desc.border_color,
            };
            let (sampler_id, err) = context.instance.0.device_create_sampler(context.device.id, &desc, None);
            let sampler = context.instance.as_auto_drop(sampler_id);
            if let Some(err) = err { anyhow::bail!("{err}") }

            let desc = wgpu::wgc::binding_model::BindGroupDescriptor {
                label: Some("Diffuse texture bind group").map(Cow::Borrowed),
                layout: context.bing_group_layout.id,
                entries: Cow::Borrowed(&[
                    wgpu::wgc::binding_model::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::wgc::binding_model::BindingResource::TextureView(view.id),
                    },
                    wgpu::wgc::binding_model::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::wgc::binding_model::BindingResource::Sampler(sampler.id),
                    },
                ]),
            };
            let (group_id, err) = context.instance.0.device_create_bind_group(context.device.id, &desc, None);
            let bind_group = context.instance.as_auto_drop(group_id);
            if let Some(err) = err { anyhow::bail!("{err}") }
            bind_group
        };

        Ok(Self {
            surface: context.instance.as_auto_drop(surface_id),
            config,
            context,
            vertex_buffer: vbuffer,
            index_buffer: ibuffer,
            image_bind_group,
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
        let source = bytemuck::cast_slice(crate::VERTICES);
        let vertex_size = BufferSize::new((crate::VERTICES.len() * size_of::<crate::Vertex>()) as u64).unwrap();

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

        // copy index data
        let source = bytemuck::cast_slice(crate::INDICES);
        let index_len = crate::INDICES.len() as u32;
        let index_size = BufferSize::new((crate::INDICES.len() * size_of::<u32>()) as u64).unwrap();

        let (index_staging_id, index_staging_offset) = self.context.instance.0.queue_create_staging_buffer(
            self.context.queue.id,
            index_size,
            None
        )?;

        let slice = unsafe { std::slice::from_raw_parts_mut(index_staging_offset.as_ptr(), source.len()) };
        slice.copy_from_slice(source);
        self.context.instance.0.queue_write_staging_buffer(
            self.context.queue.id,
            self.index_buffer.id,
            0, // dst offset
            index_staging_id
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
        self.context.instance.0.render_pass_set_bind_group(&mut pass, 0, Some(self.image_bind_group.id), &[])?;
        self.context.instance.0.render_pass_set_vertex_buffer(&mut pass, 0, self.vertex_buffer.id, 0, None)?; // offset <- vertex buffer offset, size <- vertex buffer size
        self.context.instance.0.render_pass_set_index_buffer(&mut pass, self.index_buffer.id, wgpu::wgt::IndexFormat::Uint32, 0, None)?;
        self.context.instance.0.render_pass_draw_indexed(&mut pass, index_len, 1, 0, 0, 0)?;
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
