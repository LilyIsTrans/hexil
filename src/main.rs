#![feature(error_generic_member_access)]
#![feature(negative_impls)]

pub(crate) mod global_state;
pub(crate) mod hexil_prelude;

use hexil_prelude::all::*;
use winit::event_loop::EventLoop;
impl winit::application::ApplicationHandler<HexilEvent> for Option<GlobalState> {
    fn resumed(&mut self, eloop: &winit::event_loop::ActiveEventLoop) {
        if self.is_none() {
            if let Ok(new_state) = GlobalState::new(eloop) {
                *self = Some(new_state);
            }
        }
        if let Some(state) = self {
            todo!("Allocate some command buffers and a window and a swapchain and shit!")
        } else {
            eloop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        todo!()
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::<HexilEvent>::with_user_event().build()?;

    let mut global_state = None;

    event_loop.run_app(&mut global_state)?;
    Ok(())
}
