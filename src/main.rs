#![feature(error_generic_member_access)]
#![feature(negative_impls)]
#![feature(try_blocks)]
#![feature(cast_maybe_uninit)]

pub(crate) mod global_state;
pub(crate) mod hexil_prelude;

use hexil_prelude::all::*;
use tracing::{error, info, warn};
use winit::event_loop::EventLoop;

impl winit::application::ApplicationHandler<HexilEvent> for Option<GlobalState> {
    fn resumed(&mut self, eloop: &winit::event_loop::ActiveEventLoop) {
        if let Err(uh_oh) = try {
            // TODO: This feels UGLY. Find a better way.
            let state = match self {
                Some(s) => s,
                None => {
                    *self = Some(GlobalState::new(eloop)?);
                    self.as_mut().unwrap()
                }
            };

            state.vulkan_state.activate()?;
        } {
            eprintln!("{:?}", uh_oh);
            eloop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if !matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed)
            && !event_loop.exiting()
        {
            if self.is_none() {
                self.resumed(event_loop);
            }
        }
        let state = self.as_mut();
        use winit::event::WindowEvent;
        if let Err(uh_oh) = try {
            match &event {
                WindowEvent::Resized(physical_size) => state
                    .expect("Expected global state to be initialized!")
                    .handle_resized(*physical_size, window_id)?,
                WindowEvent::CloseRequested | WindowEvent::Destroyed => match state {
                    None => (),
                    Some(state) => {
                        state.handle_close_requested(event_loop);
                    }
                },
                WindowEvent::ScaleFactorChanged {
                    scale_factor,
                    inner_size_writer,
                } => warn!("We don't support scaling yet!"),
                WindowEvent::RedrawRequested => match state {
                    None => error!("Something invalidated my state! Wtf???"),
                    Some(state) => state.perform_redraw()?,
                },

                other => {
                    info!("Ignored event: {:?}", other)
                }
            }
        } {
            error!(window_id = Into::<u64>::into(window_id), "{:?}", event);
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::<HexilEvent>::with_user_event().build()?;

    let mut global_state = None;

    event_loop.run_app(&mut global_state)?;
    Ok(())
}
