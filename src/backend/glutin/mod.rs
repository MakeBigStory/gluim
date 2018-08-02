#![cfg(feature = "glutin")]
/*!

Backend implementation for the glutin library

# Features

Only available if the 'glutin' feature is enabled.

*/
pub extern crate glutin;

pub mod headless;

use {Frame, IncompatibleOpenGl, SwapBuffersError};
use debug;
use context;
use backend;
use backend::Context;
use backend::Backend;
use std;
use std::cell::{Cell, RefCell, Ref};
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use std::ops::Deref;
use std::os::raw::c_void;

/// A GL context combined with a facade for drawing upon.
///
/// The `Display` uses **glutin** for the **Window** and its associated GL **Context**.
///
/// These are stored alongside a glium-specific context.
#[derive(Clone)]
pub struct Display {
    // contains everything related to the current context and its state
    context: Rc<context::Context>,
    // Used to check whether the framebuffer dimensions have changed between frames. If they have,
    // the glutin context must be resized accordingly.
    last_framebuffer_dimensions: Cell<(u32, u32)>,
}

/// Error that can happen while creating a glium display.
#[derive(Debug)]
pub enum DisplayCreationError {
    /// An error has happened while creating the backend.
    GlutinCreationError(glutin::CreationError),
    /// The OpenGL implementation is too old.
    IncompatibleOpenGl(IncompatibleOpenGl),
}

struct NullBacked;

unsafe impl backend::Backend for NullBacked {
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        Ok(())
    }

    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        std::ptr::null()
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        (800, 600)
    }

    fn is_current(&self) -> bool {
        true
    }

    unsafe fn make_current(&self) {
    }
}

impl Display {
    /// Create a new glium `Display` from the given context and window builders.
    ///
    /// Performs a compatibility check to make sure that all core elements of glium are supported
    /// by the implementation.
    pub fn new() -> Result<Self, DisplayCreationError>
    {
        Self::from_gl_window().map_err(From::from)
    }

    /// Create a new glium `Display`.
    ///
    /// Performs a compatibility check to make sure that all core elements of glium are supported
    /// by the implementation.
    pub fn from_gl_window() -> Result<Self, IncompatibleOpenGl> {
        Self::with_debug(Default::default())
    }

    /// Create a new glium `Display`.
    ///
    /// This function does the same as `build_glium`, except that the resulting context
    /// will assume that the current OpenGL context will never change.
    pub unsafe fn unchecked() -> Result<Self, IncompatibleOpenGl> {
        Self::unchecked_with_debug(Default::default())
    }

    /// The same as the `new` constructor, but allows for specifying debug callback behaviour.
    pub fn with_debug(debug: debug::DebugCallbackBehavior)
        -> Result<Self, IncompatibleOpenGl>
    {
        Self::new_inner(debug, true)
    }

    /// The same as the `unchecked` constructor, but allows for specifying debug callback behaviour.
    pub unsafe fn unchecked_with_debug(
        debug: debug::DebugCallbackBehavior,
    ) -> Result<Self, IncompatibleOpenGl>
    {
        Self::new_inner(debug, false)
    }

    fn new_inner(
        debug: debug::DebugCallbackBehavior,
        checked: bool,
    ) -> Result<Self, IncompatibleOpenGl>
    {
        let glutin_backend = NullBacked {};
        let framebuffer_dimensions = glutin_backend.get_framebuffer_dimensions();
        let context = try!(unsafe { context::Context::new(glutin_backend, checked, debug) });
        Ok(Display {
            context: context,
            last_framebuffer_dimensions: Cell::new(framebuffer_dimensions),
        })
    }

    /// Rebuilds the Display's `GlWindow` with the given window and context builders.
    ///
    /// This method ensures that the new `GlWindow`'s `Context` will share the display lists of the
    /// original `GlWindow`'s `Context`.
    pub fn rebuild(
        &self,
        window_builder: glutin::WindowBuilder,
        context_builder: glutin::ContextBuilder,
        events_loop: &glutin::EventsLoop,
    ) -> Result<(), DisplayCreationError>
    {
        Ok(())
    }

    /// Start drawing on the backbuffer.
    ///
    /// This function returns a `Frame`, which can be used to draw on it. When the `Frame` is
    /// destroyed, the buffers are swapped.
    ///
    /// Note that destroying a `Frame` is immediate, even if vsync is enabled.
    ///
    /// If the framebuffer dimensions have changed since the last call to `draw`, the inner glutin
    /// context will be resized accordingly before returning the `Frame`.
    #[inline]
    pub fn draw(&self) -> Frame {
        let (w, h) = self.get_framebuffer_dimensions();
        Frame::new(self.context.clone(), (w, h))
    }
}

impl fmt::Display for DisplayCreationError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.description())
    }
}

impl Error for DisplayCreationError {
    #[inline]
    fn description(&self) -> &str {
        match *self {
            DisplayCreationError::GlutinCreationError(ref err) => err.description(),
            DisplayCreationError::IncompatibleOpenGl(ref err) => err.description(),
        }
    }

    #[inline]
    fn cause(&self) -> Option<&Error> {
        match *self {
            DisplayCreationError::GlutinCreationError(ref err) => Some(err),
            DisplayCreationError::IncompatibleOpenGl(ref err) => Some(err),
        }
    }
}

impl From<glutin::CreationError> for DisplayCreationError {
    #[inline]
    fn from(err: glutin::CreationError) -> DisplayCreationError {
        DisplayCreationError::GlutinCreationError(err)
    }
}

impl From<IncompatibleOpenGl> for DisplayCreationError {
    #[inline]
    fn from(err: IncompatibleOpenGl) -> DisplayCreationError {
        DisplayCreationError::IncompatibleOpenGl(err)
    }
}

impl Deref for Display {
    type Target = Context;
    #[inline]
    fn deref(&self) -> &Context {
        &self.context
    }
}

impl backend::Facade for Display {
    #[inline]
    fn get_context(&self) -> &Rc<Context> {
        &self.context
    }
}
