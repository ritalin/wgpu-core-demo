use std::{collections::HashMap, sync::Arc};
use winit::{application::ApplicationHandler, dpi::PhysicalSize, event::{KeyEvent, StartCause, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy}, keyboard::{KeyCode, PhysicalKey}, platform::macos::WindowAttributesExtMacOS, window::{Window, WindowAttributes, WindowId}};

use crate::{render, runtime};

pub struct App {
    proxy_loop: EventLoopProxy<runtime::UserEvent>,
    suspended: bool,
    state: AppState
}
impl App {
    pub fn new(event_loop: &EventLoop<runtime::UserEvent>, terminate_on_empty: bool) -> Self {
        Self {
            proxy_loop: event_loop.create_proxy(),
            suspended: true,
            state: AppState::new(terminate_on_empty),
        }
    }
}

impl ApplicationHandler<runtime::UserEvent> for App {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.render_context.is_some() {
            self.state.resume_app();
            self.suspended = false;
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
       self.suspended = true;
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        self.state.handle(window_id, event_loop, |entry, status| {
            match event {
                WindowEvent::CloseRequested => {
                    *status = HandleStatus::Closed;
                }
                WindowEvent::KeyboardInput { event: KeyEvent{ physical_key: PhysicalKey::Code(KeyCode::Escape), .. }, .. } => {
                    *status = HandleStatus::Closed;
                }
                WindowEvent::Resized(size) => {
                    entry.handle_resize(size);
                }
                WindowEvent::RedrawRequested if ! self.suspended => {
                    entry.handle_draw();
                }
                _ => {
                    log::warn!("Event handler is not implemented: (id: {window_id:?}, event: {event:?}");
                }
            }
        });
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            if let Err(err) = self.state.init_render_context(event_loop, self.proxy_loop.clone()) {
                log::error!("Failed to create gpu rendering context (cause: {err})");
                event_loop.exit();
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: runtime::UserEvent) {
        match event {
            runtime::UserEvent::RequestNew => {
                self.state.add_new_window(event_loop).unwrap();
            }
        }
    }
}

struct AppState {
    app_entries: HashMap<WindowId, Entry>,
    render_context: Option<Arc<runtime::RenderContext>>,
    terminate_on_empty: bool,
}
impl AppState {
    const DEFAULT_SIZE: PhysicalSize<u32> = PhysicalSize::new(1024, 768);

    fn new(terminate_on_empty: bool) -> Self {
        Self {
            app_entries: HashMap::new(),
            render_context: None,
            terminate_on_empty,
        }
    }

    fn init_render_context(&mut self, event_loop: &ActiveEventLoop, event_loop_proxy: EventLoopProxy<runtime::UserEvent>) -> Result<(), anyhow::Error> {
        let attr = WindowAttributes::default()
            .with_inner_size(PhysicalSize::new(1, 1))
            .with_transparent(true)
            .with_has_shadow(false)
        ;
        let window = Arc::new(event_loop.create_window(attr).unwrap());

        let context = runtime::init_render_context(Box::new(WindowWrapper(window)))?;
        self.render_context = Some(Arc::new(context));

        event_loop_proxy.send_event(runtime::UserEvent::RequestNew).map_err(|err| anyhow::anyhow!("Failed to create new window (reson: {err}"))?;
        Ok(())
    }

    fn add_new_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), anyhow::Error> {
        let Some(context) = self.render_context.as_ref() else { anyhow::bail!("GPU rendering context is not initialized") };

        let attr = WindowAttributes::default()
            .with_inner_size(Self::DEFAULT_SIZE)
        ;
        let window = Arc::new(event_loop.create_window(attr)?);
        let surface_id = runtime::create_surface(context, WindowWrapper(window.clone()))?;
        let renderer = render::WgpuRenderer::new(context.clone(), surface_id, (Self::DEFAULT_SIZE.width, Self::DEFAULT_SIZE.height))?;

        self.app_entries.insert(window.id(), Entry::new(window, renderer));
        Ok(())
    }

    fn handle(&mut self, id: WindowId, event_loop: &ActiveEventLoop, mut callback: impl FnMut(&mut Entry, &mut HandleStatus)) {
        let mut status = HandleStatus::None;

        if let Some(entry) = self.app_entries.get_mut(&id) {
            callback(entry, &mut status);
        }
        if status == HandleStatus::Closed {
            self.app_entries.remove(&id);
            if self.terminate_on_empty && self.app_entries.is_empty() {
                event_loop.exit();
            }
        }
    }

    fn resume_app(&mut self) {
        for entry in self.app_entries.values() {
            entry.window.request_redraw();
        }
    }
}

#[derive(PartialEq)]
enum HandleStatus {
    None,
    Closed,
}

struct Entry {
    dirty_resized: Option<(u32, u32)>,
    window: Arc<Window>,
    renderer: render::WgpuRenderer,
}
impl Entry {
    fn new(window: Arc<Window>, renderer: render::WgpuRenderer) -> Self {
        let size = window.inner_size();

        Self {
            dirty_resized: Some((size.width, size.height)),
            window,
            renderer,
        }
    }

    fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        self.dirty_resized = Some((size.width, size.height));
    }

    fn handle_draw(&mut self) {
        if let Some(size) = self.dirty_resized.take() {
            self.renderer.request_resize(size);
        }

        self.renderer.render().unwrap();
        self.window.request_redraw();
    }
}

struct WindowWrapper(Arc<Window>);

impl runtime::AsRawWindow for WindowWrapper {
    fn get_handle(&self) -> Result<runtime::RawWindowHandle, wgpu::rwh::HandleError> {
        use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
        Ok(runtime::RawWindowHandle::new(
            self.0.display_handle()?.as_raw(),
            self.0.window_handle()?.as_raw(),
        ))
    }
}
