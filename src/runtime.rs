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

    Ok(RenderContext{
        device: instance.as_auto_drop(device_id),
        queue: instance.as_auto_drop(queue_id),
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
