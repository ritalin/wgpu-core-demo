use std::{collections::HashMap, sync::Arc};
use winit::{application::ApplicationHandler, dpi::PhysicalSize, event::{KeyEvent, StartCause, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowAttributes, WindowId}};

pub struct App {
    suspended: bool,
    state: AppState
}
impl App {
    pub fn new() -> Self {
        Self {
            suspended: true,
            state: AppState::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.state.resume_app();
        self.suspended = false;
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
        self.state.handle(window_id, |entry| {
            match event {
                WindowEvent::CloseRequested => {
                    entry.handle_close(event_loop)
                }
                WindowEvent::KeyboardInput { event: KeyEvent{ physical_key: PhysicalKey::Code(KeyCode::Escape), .. }, .. } => {
                    entry.handle_close(event_loop);
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
            if self.state.add_new_window(event_loop).is_err() {
                log::error!("Failed to create new window");
                event_loop.exit();
            }
        }
    }
}

struct AppState {
    app_entries: HashMap<WindowId, Entry>,
}
impl AppState {
    const DEFAULT_SIZE: PhysicalSize<u32> = PhysicalSize::new(1024, 768);

    fn new() -> Self {
        Self {
            app_entries: HashMap::new(),
        }
    }

    pub fn add_new_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), anyhow::Error> {
        let attr = WindowAttributes::default()
            .with_inner_size(Self::DEFAULT_SIZE)
        ;
        let window = Arc::new(event_loop.create_window(attr)?);

        //

        self.app_entries.insert(window.id(), Entry::new(window));
        Ok(())
    }

    fn handle(&mut self, id: WindowId, mut callback: impl FnMut(&mut Entry)) {
        if let Some(entry) = self.app_entries.get_mut(&id) {
            callback(entry);
        }
    }

    fn resume_app(&mut self) {
        for entry in self.app_entries.values() {
            entry.window.request_redraw();
        }
    }
}

struct Entry {
    dirty_resized: Option<(u32, u32)>,
    window: Arc<Window>,
}
impl Entry {
    fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        Self {
            dirty_resized: Some((size.width, size.height)),
            window,
        }
    }
    fn handle_close(&self, event_loop: &ActiveEventLoop) {
        event_loop.exit();
    }

    fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        self.dirty_resized = Some((size.width, size.height));
    }

    fn handle_draw(&self) {

    }
}
