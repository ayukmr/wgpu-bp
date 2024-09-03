use crate::control_flow::ControlFlow;
use winit::event_loop::EventLoop;

use anyhow::Result;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    result_runner().unwrap()
}

pub fn result_runner() -> Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));

            console_log::init_with_level(log::Level::Info)
                .expect("error occurred when initializing logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new()?;
    let _ = event_loop.run_app(&mut ControlFlow::new());

    Ok(())
}
