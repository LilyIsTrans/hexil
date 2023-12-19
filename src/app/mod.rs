use tokio::sync::RwLock;
use winit::dpi::LogicalPosition;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoopProxy;
use winit::window::Window;

use crate::window::WindowCommand;

pub struct Application {
    /// None if mouse outside window
    mouse_pos: RwLock<Option<LogicalPosition<f32>>>,
    eprox: EventLoopProxy<WindowCommand>,
    window: Window,
}

impl Application {
    pub fn new(eprox: EventLoopProxy<WindowCommand>, window: Window) -> Self {
        Self {
            mouse_pos: RwLock::const_new(None),
            eprox,
            window,
        }
    }

    /// Should be called if the mouse is no longer in the window.
    pub async fn clear_mouse_pos(&mut self) {
        let mut guard = self.mouse_pos.write().await;
        *guard = None;
    }

    pub async fn update_mouse_pos(&mut self, new_pos: LogicalPosition<f32>) {
        let mut guard = self.mouse_pos.write().await;
        *guard = Some(new_pos);
    }

    pub async fn get_mouse_pos(&self) -> Option<LogicalPosition<f32>> {
        let a = self.mouse_pos.read().await;
        *a
    }

    pub fn get_event_loop_proxy(&self) -> EventLoopProxy<WindowCommand> {
        self.eprox.clone()
    }
}
