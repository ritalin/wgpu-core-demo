use std::{borrow::Cow, sync::Arc};

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

type WgpuInstance = Arc<wgpu::wgc::global::Global>;

pub struct RenderContext {
    instance: WgpuInstance,
    device_id:  wgpu::wgc::id::DeviceId,
    queue_id: wgpu::wgc::id::QueueId,
}

pub fn init_render_context(target: Box<dyn AsRawWindow + 'static>) -> Result<RenderContext, anyhow::Error> {
    let desc = wgpu::wgt::InstanceDescriptor {
        backends: wgpu::wgt::Backends::PRIMARY,
        ..Default::default()
    };
    let instance = Arc::new(wgpu::wgc::global::Global::new("gpu", &desc, None));

    let handle = target.get_handle().unwrap();
    let surface_id = unsafe { instance.instance_create_surface(handle.display_handle, handle.window_handle, None) }.unwrap();

    let desc = wgpu::wgt::RequestAdapterOptions {
        power_preference: wgpu::wgt::PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(surface_id),
    };
    let adapter_id = instance.request_adapter(&desc, wgpu::wgt::Backends::all(), None)?;

    let desc = wgpu::wgt::DeviceDescriptor {
        label: Some("Fetch the driver and the queue"),
        required_features: wgpu::wgt::Features::empty(),
        required_limits: wgpu::wgt::Limits::defaults(),
        experimental_features: wgpu::wgt::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::wgt::MemoryHints::default(),
        trace: wgpu::wgt::Trace::Off,
    };
    let (device_id, queue_id) = instance.adapter_request_device(adapter_id, &desc.map_label(|s| s.map(Cow::Borrowed)), None, None)?;

    instance.adapter_drop(adapter_id);

    Ok(RenderContext{
        instance,
        device_id,
        queue_id,
    })
}
