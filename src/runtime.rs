use std::{borrow::Cow, sync::Arc};

use crate::wgpu_resource::{AutoDropId, WgpuInstance};

pub enum UserEvent {
    RequestNew,
}

pub trait AsRawWindow {
    fn get_handle(&self) -> Result<RawWindowHandle, wgpu::rwh::HandleError>;
}

#[derive(Debug)]
pub struct RawWindowHandle {
    pub(crate) display_handle: wgpu::rwh::RawDisplayHandle,
    pub(crate) window_handle: wgpu::rwh::RawWindowHandle,
}
impl RawWindowHandle {
    pub fn new(display_handle: wgpu::rwh::RawDisplayHandle, window_handle: wgpu::rwh::RawWindowHandle) -> Self {
        Self { display_handle, window_handle }
    }
}

pub struct RenderContext {
    pub(crate) instance: WgpuInstance,
    pub(crate) device:  AutoDropId<wgpu::wgc::id::DeviceId>,
    pub(crate) queue: AutoDropId<wgpu::wgc::id::QueueId>,
    pub(crate) pipeline: AutoDropId<wgpu::wgc::id::RenderPipelineId>,
    pub(crate) config: wgpu::wgt::SurfaceConfiguration<Vec<wgpu::wgt::TextureFormat>>,
}

pub fn init_render_context(target: Box<dyn AsRawWindow + 'static>) -> Result<RenderContext, anyhow::Error> {
    let desc = wgpu::wgt::InstanceDescriptor {
        backends: wgpu::wgt::Backends::PRIMARY,
        ..Default::default()
    };
    let instance = WgpuInstance(Arc::new(wgpu::wgc::global::Global::new("gpu", &desc, None)));

    let handle = target.get_handle().unwrap();
    let surface_id = unsafe { instance.0.instance_create_surface(handle.display_handle, handle.window_handle, None) }.unwrap();

    let desc = wgpu::wgt::RequestAdapterOptions {
        power_preference: wgpu::wgt::PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(surface_id),
    };
    let adapter_id: wgpu::wgc::id::AdapterId = instance.0.request_adapter(&desc, wgpu::wgt::Backends::all(), None)?;
    let adapter = instance.as_auto_drop(adapter_id);

    let desc = wgpu::wgt::DeviceDescriptor {
        label: Some("Fetch the driver and the queue"),
        required_features: wgpu::wgt::Features::empty(),
        required_limits: wgpu::wgt::Limits::defaults(),
        experimental_features: wgpu::wgt::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::wgt::MemoryHints::default(),
        trace: wgpu::wgt::Trace::Off,
    };
    let (device_id, queue_id) = instance.0.adapter_request_device(adapter.id, &desc.map_label(|s| s.map(Cow::Borrowed)), None, None)?;

    let caps = instance.0.surface_get_capabilities(surface_id, adapter.id)?;
    let format = caps.formats.iter().find(|fmt| fmt.is_srgb()).cloned().unwrap_or(caps.formats[0]);
    let config = wgpu::wgt::SurfaceConfiguration {
        usage: wgpu::wgt::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: 0,
        height: 0,
        present_mode: caps.present_modes[0],
        desired_maximum_frame_latency: 2,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
    };

    let desc = wgpu::wgc::binding_model::PipelineLayoutDescriptor {
        label: Some("Render pipeline layout").map(Cow::Borrowed),
        bind_group_layouts: Cow::Borrowed(&[]),
        immediate_size: 0,
    };
    let (layout_id, err) = instance.0.device_create_pipeline_layout(device_id, &desc, None);
    let layout = instance.as_auto_drop(layout_id);
    if let Some(err) = err { anyhow::bail!("{err}") }

    let source = wgpu::wgc::pipeline::ShaderModuleSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl")));
    let desc = wgpu::wgc::pipeline::ShaderModuleDescriptor {
        label: Some("Shader").map(Cow::Borrowed),
        runtime_checks: wgpu::wgt::ShaderRuntimeChecks::checked(),
    };
    let (shader_id, err) = instance.0.device_create_shader_module(device_id, &desc, source, None);
    let shader = instance.as_auto_drop(shader_id);
    if let Some(err) = err { anyhow::bail!("{err}") }

    let desc = wgpu::wgc::pipeline::RenderPipelineDescriptor {
        label: Some("Render pipeline").map(Cow::Borrowed),
        layout: Some(layout.id),
        vertex: wgpu::wgc::pipeline::VertexState {
            stage: wgpu::wgc::pipeline::ProgrammableStageDescriptor {
                module: shader.id,
                entry_point: Some("vs_main").map(Cow::Borrowed),
                constants: wgpu::naga::back::PipelineConstants::default(),
                zero_initialize_workgroup_memory: false,
            },
            buffers: Cow::Borrowed(&[crate::Vertex::desc()]),
        },
        fragment: Some(wgpu::wgc::pipeline::FragmentState {
            stage: wgpu::wgc::pipeline::ProgrammableStageDescriptor {
                module: shader.id,
                entry_point: Some("fs_main").map(Cow::Borrowed),
                constants: wgpu::naga::back::PipelineConstants::default(),
                zero_initialize_workgroup_memory: false,
            },
            targets: Cow::Borrowed(&[
                Some(wgpu::wgt::ColorTargetState {
                    format,
                    blend: Some(wgpu::wgt::BlendState::REPLACE),
                    write_mask: wgpu::wgt::ColorWrites::ALL,
                })
            ]),
        }),
        primitive: wgpu::wgt::PrimitiveState {
            topology: wgpu::wgt::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::wgt::FrontFace::Ccw,
            cull_mode: Some(wgpu::wgt::Face::Back),
            polygon_mode: wgpu::wgt::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        multisample: wgpu::wgt::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        depth_stencil: None,
        multiview_mask: None,
        cache: None,
    };
    let (pipeline_id, err) = instance.0.device_create_render_pipeline(device_id, &desc, None);
    if let Some(err) = err { anyhow::bail!("{err}") }

    Ok(RenderContext{
        device: instance.as_auto_drop(device_id),
        queue: instance.as_auto_drop(queue_id),
        pipeline: instance.as_auto_drop(pipeline_id),
        instance,
        config,
    })
}

pub fn create_surface(context: &RenderContext, target: impl AsRawWindow) -> Result<wgpu::wgc::id::SurfaceId, anyhow::Error> {
    let handle = target.get_handle()?;
    let surface_id = unsafe {
        context.instance.0.instance_create_surface(handle.display_handle, handle.window_handle, None)?
    };
    Ok(surface_id)
}
