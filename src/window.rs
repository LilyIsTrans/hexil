use thiserror::Error;
use tracing::error;
use tracing::info;
use tracing::instrument;
use tracing::warn;
use winit::error::EventLoopError;
use winit::error::OsError;
use winit::event;
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::render::RenderCommand;

#[derive(Debug, Error)]
pub enum WindowingError {
    #[error("")]
    ELoopErr(EventLoopError),
    #[error("")]
    WinErr(OsError),
}

impl From<EventLoopError> for WindowingError {
    fn from(value: EventLoopError) -> Self {
        Self::ELoopErr(value)
    }
}

impl From<OsError> for WindowingError {
    fn from(value: OsError) -> Self {
        Self::WinErr(value)
    }
}

/// A message that can be sent to the window event loop
pub enum WindowCommand {}

#[instrument]
pub fn make_event_loop() -> Result<EventLoop<WindowCommand>, EventLoopError> {
    use winit::event_loop::EventLoopBuilder;
    EventLoopBuilder::<WindowCommand>::with_user_event().build()
}

#[instrument(skip(eloop))]
pub fn make_window(title: &str, eloop: &EventLoop<WindowCommand>) -> Result<Window, OsError> {
    use winit::window::WindowBuilder;
    WindowBuilder::new()
        .with_title(title)
        .with_visible(false)
        .with_theme(Some(winit::window::Theme::Dark))
        .with_resizable(true)
        .build(eloop)
}

/// If this function returns, the event loop is dead. Ok(()) means it closed gracefully.
#[instrument]
pub fn run_event_loop(
    eloop: EventLoop<WindowCommand>,
    render_handle: std::sync::mpsc::Sender<RenderCommand>,
) -> Result<(), EventLoopError> {
    eloop.run(|event, window_target| match event {
        Event::WindowEvent {
            window_id: _,
            event,
        } => match event {
            event::WindowEvent::Resized(new_size) => {
                render_handle.send(RenderCommand::WindowResized(new_size.into()));
            }
            event::WindowEvent::CloseRequested => {
                info!("Closing window!");
                let _ = render_handle.send(RenderCommand::Shutdown);
                window_target.exit();
            }
            event::WindowEvent::Destroyed => {
                warn!("Window destroyed!!");
                let _ = render_handle.send(RenderCommand::Shutdown);
                window_target.exit();
            }
            _ => (),
        },
        Event::UserEvent(_) => (),
        _ => (),
    })?;

    Ok(())
}
