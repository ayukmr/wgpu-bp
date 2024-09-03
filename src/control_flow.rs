use instant::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::window::{Window, WindowAttributes};

use crate::state::State;

pub struct ControlFlow<'a> {
    window: Option<Window>,
    state:  Option<State<'a>>,
    last_render: Instant,
}

impl<'a> ControlFlow<'a> {
    pub fn new() -> Self {
        Self {
            window: None,
            state:  None,
            last_render: Instant::now(),
        }
    }
}

impl<'a> ApplicationHandler for ControlFlow<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(event_loop.create_window(
            WindowAttributes::default().with_title("WGPU Boilerplate"),
        ).unwrap());

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let body = doc.body()?;

                    let canvas = self.window.as_ref().unwrap().canvas()?;
                    body.append_child(&canvas).ok()?;

                    Some(())
                });
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        } else if let Some(state) = self.state.as_ref() {
            state.window().request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event:      winit::event::WindowEvent,
    ) {
        if let Some(state) = self.state.as_mut() {
            if !state.event(&event) {
                match event {
                    WindowEvent::CloseRequested => {
                        event_loop.exit();
                    }

                    WindowEvent::Resized(physical_size) => {
                        state.resize(physical_size);
                    }

                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        state.scale(&scale_factor);
                    }

                    WindowEvent::RedrawRequested => {
                        let now = instant::Instant::now();

                        let dt = now - self.last_render;
                        self.last_render = now;

                        state.update(dt);
                        state.render().unwrap();
                    }

                    _ => {}
                }
            }
        } else if let WindowEvent::Resized(_) = &event {
            self.state = Some(
                pollster::block_on(
                    State::new(self.window.take().unwrap()),
                ).unwrap(),
            );
        }
    }
}
