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
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

use crate::render::RenderCommand;

/// The unified error type for Hexil's windowing system.
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

/// A message that can be sent to the window event loop. Currently, there's nothing here. As long as that remains the case,
/// this should act like `Infallible` (that is, the compiler should recognize that any code which would need a value of this
/// type must never run, so, for example, waiting on an `std::sync::mpsc<WindowCommand>` should be optimized out as a noop.)
pub enum WindowCommand {}

/// Makes an event loop suitable for Hexil.
#[instrument]
pub fn make_event_loop() -> Result<EventLoop<WindowCommand>, EventLoopError> {
    use winit::event_loop::EventLoopBuilder;
    EventLoopBuilder::<WindowCommand>::with_user_event().build()
}

/// Makes a window with the given title and event loop, suitable for Hexil. Hexil won't show it until the renderer is started.
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
/// This must be run on the main thread, and will not return until program termination. As such,
/// any code which runs independently must be initialized to a separate thread before this is called.
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
            event::WindowEvent::Resized(new_size) => send_or_exit(
                &render_handle,
                window_target,
                RenderCommand::WindowResized(new_size.into()),
            ),
            event::WindowEvent::CloseRequested => {
                info!("Closing window!");
                send_or_exit(&render_handle, window_target, RenderCommand::Shutdown);
            }
            event::WindowEvent::Destroyed => {
                warn!("Window destroyed!!");
                send_or_exit(&render_handle, window_target, RenderCommand::Shutdown);
            }
            _ => (),
        },
        Event::UserEvent(_) => (),
        _ => (),
    })?;

    Ok(())
}

/// Sends `command`, and calls `window_target.exit()` if the render thread is dead.
fn send_or_exit(
    render_handle: &std::sync::mpsc::Sender<RenderCommand>,
    window_target: &EventLoopWindowTarget<WindowCommand>,
    command: RenderCommand,
) {
    match render_handle.send(command) {
        Ok(_) => (),
        Err(e) => {
            // TODO: Add the ability to trigger a Command & Control thread to rebuild the Vulkan instance and display an error message here
            error!("Renderer has died! Last command to renderer: {:#?}", e.0);
            window_target.exit();
        }
    };
}
