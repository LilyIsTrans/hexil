use thiserror::Error;
use tracing::instrument;
use winit::error::EventLoopError;
use winit::error::OsError;
use winit::event;
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::window::Window;

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
        .build(eloop)
}

/// If this function returns, the event loop is dead. Ok(()) means it closed gracefully.
#[instrument]
pub fn run_event_loop(eloop: EventLoop<WindowCommand>) -> Result<(), EventLoopError> {
    eloop.run(|event, window_target| match event {
        Event::WindowEvent { window_id, event } => match event {
            event::WindowEvent::Resized(_) => todo!(),
            event::WindowEvent::Moved(_) => todo!(),
            event::WindowEvent::CloseRequested => todo!(),
            event::WindowEvent::Destroyed => todo!(),
            event::WindowEvent::DroppedFile(_) => todo!(),
            event::WindowEvent::Focused(_) => todo!(),
            event::WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => todo!(),
            event::WindowEvent::ModifiersChanged(_) => todo!(),
            event::WindowEvent::CursorMoved {
                device_id,
                position,
            } => todo!(),
            event::WindowEvent::CursorEntered { device_id } => todo!(),
            event::WindowEvent::CursorLeft { device_id } => todo!(),
            event::WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => todo!(),
            event::WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => todo!(),
            event::WindowEvent::TouchpadMagnify {
                device_id,
                delta,
                phase,
            } => todo!(),
            event::WindowEvent::SmartMagnify { device_id } => todo!(),
            event::WindowEvent::Touch(_) => todo!(),
            event::WindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer,
            } => todo!(),
            event::WindowEvent::ThemeChanged(_) => todo!(),
            event::WindowEvent::Occluded(_) => todo!(),
            event::WindowEvent::RedrawRequested => todo!(),
            _ => (),
        },
        Event::UserEvent(_) => (),
        _ => (),
    })?;

    Ok(())
}
