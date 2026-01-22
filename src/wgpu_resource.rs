use std::sync::Arc;

#[derive(Clone)]
pub struct WgpuInstance(pub Arc<wgpu::wgc::global::Global>);

impl WgpuInstance {
    pub(crate) fn as_auto_drop<T: AutoDrop>(&self, id: T) -> AutoDropId<T> {
        AutoDropId { instance: self.0.clone(), id }
    }
}

pub struct AutoDropId<T: AutoDrop> {
    instance: Arc<wgpu::wgc::global::Global>,
    pub id: T,
}

impl<T: AutoDrop> Drop for AutoDropId<T> {
    fn drop(&mut self) {
        self.id.drop_id(&self.instance);
    }
}

pub(crate) trait AutoDrop {
    fn drop_id(&self, instance: &wgpu::wgc::global::Global);
}

impl AutoDrop for wgpu::wgc::id::AdapterId {
    fn drop_id(&self, instance: &wgpu::wgc::global::Global) {
        instance.adapter_drop(*self);
    }
}

impl AutoDrop for wgpu::wgc::id::DeviceId {
    fn drop_id(&self, instance: &wgpu::wgc::global::Global) {
        instance.device_drop(*self);
    }
}

impl AutoDrop for wgpu::wgc::id::QueueId {
    fn drop_id(&self, instance: &wgpu::wgc::global::Global) {
        instance.queue_drop(*self);
    }
}
