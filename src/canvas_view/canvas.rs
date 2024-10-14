use bevy::window::WindowWrapper;
use std::{ops::Deref, ptr::NonNull};
use wasm_bindgen::{JsCast, JsValue};

// ViewObj enum to support both Canvas and OffscreenCanvas
#[derive(Debug)]
pub enum ViewObj {
    Canvas(WindowWrapper<CanvasWrapper>),
    Offscreen(WindowWrapper<OffscreenCanvasWrapper>),
}

impl ViewObj {
    // Create a ViewObj from a Canvas
    pub fn from_canvas(canvas: Canvas) -> Self {
        ViewObj::Canvas(WindowWrapper::new(CanvasWrapper::new(canvas)))
    }

    // Create a ViewObj from an OffscreenCanvas
    pub fn from_offscreen_canvas(canvas: OffscreenCanvas) -> Self {
        ViewObj::Offscreen(WindowWrapper::new(OffscreenCanvasWrapper::new(canvas)))
    }
}

// Wrapper to make types Send and Sync
#[derive(Clone, Debug)]
pub(crate) struct SendSyncWrapper<T>(pub(crate) T);

// Implement Send and Sync for SendSyncWrapper
unsafe impl<T> Send for SendSyncWrapper<T> {}
unsafe impl<T> Sync for SendSyncWrapper<T> {}

// Wrapper for Canvas to implement necessary traits
#[derive(Debug)]
pub struct CanvasWrapper(SendSyncWrapper<Canvas>);
impl CanvasWrapper {
    pub fn new(canvas: Canvas) -> Self {
        CanvasWrapper(SendSyncWrapper(canvas))
    }
}

// Implement Deref for CanvasWrapper
impl Deref for CanvasWrapper {
    type Target = Canvas;

    fn deref(&self) -> &Self::Target {
        &self.0 .0
    }
}

// Wrapper for OffscreenCanvas to implement necessary traits
#[derive(Debug)]
pub struct OffscreenCanvasWrapper(SendSyncWrapper<OffscreenCanvas>);
impl OffscreenCanvasWrapper {
    pub fn new(canvas: OffscreenCanvas) -> Self {
        OffscreenCanvasWrapper(SendSyncWrapper(canvas))
    }
}

// Implement Deref for OffscreenCanvasWrapper
impl Deref for OffscreenCanvasWrapper {
    type Target = OffscreenCanvas;

    fn deref(&self) -> &Self::Target {
        &self.0 .0
    }
}

// Canvas struct representing an HTML canvas element
#[derive(Debug, Clone)]
pub struct Canvas {
    element: web_sys::HtmlCanvasElement,
    pub scale_factor: f32,
    handle: u32,
}

#[allow(dead_code)]
impl Canvas {
    // Create a new Canvas instance
    pub fn new(selectors: &str, handle: u32) -> Self {
        // Ensure handle is greater than 0 (0 is reserved for the window itself)
        assert!(handle > 0);

        // Get the canvas element and scale factor
        let (element, scale_factor) = Self::get_canvas_element(selectors);
        // Set the data-raw-handle attribute required by raw-window-handle
        element
            .set_attribute("data-raw-handle", handle.to_string().as_str())
            .unwrap();

        Self {
            element,
            scale_factor,
            handle,
        }
    }

    // Get the canvas element and scale factor
    pub fn get_canvas_element(element_id: &str) -> (web_sys::HtmlCanvasElement, f32) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let element = document
            .get_element_by_id(element_id)
            .expect("Cannot find canvas element in the page");

        let canvas = element.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        let scale_factor = window.device_pixel_ratio() as f32;

        (canvas, scale_factor)
    }

    // Get the handle
    #[inline]
    pub fn handle(&self) -> u32 {
        self.handle
    }

    // Get the logical resolution of the canvas
    pub fn logical_resolution(&self) -> (f32, f32) {
        let width = self.element.width();
        let height = self.element.height();
        (width as f32, height as f32)
    }
}

// Implement Deref for Canvas
impl Deref for Canvas {
    type Target = web_sys::HtmlCanvasElement;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

// Implement HasWindowHandle for CanvasWrapper
impl raw_window_handle::HasWindowHandle for CanvasWrapper {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        use raw_window_handle::{RawWindowHandle, WebCanvasWindowHandle, WindowHandle};

        let value: &JsValue = &self.0 .0.element;
        let obj: NonNull<std::ffi::c_void> = NonNull::from(value).cast();
        let handle = WebCanvasWindowHandle::new(obj);
        let raw = RawWindowHandle::WebCanvas(handle);
        unsafe { Ok(WindowHandle::borrow_raw(raw)) }
    }
}

// Implement HasDisplayHandle for CanvasWrapper
impl raw_window_handle::HasDisplayHandle for CanvasWrapper {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        use raw_window_handle::{DisplayHandle, RawDisplayHandle, WebDisplayHandle};
        let handle = WebDisplayHandle::new();
        let raw = RawDisplayHandle::Web(handle);
        unsafe { Ok(DisplayHandle::borrow_raw(raw)) }
    }
}

// OffscreenCanvas struct representing an offscreen canvas
#[derive(Debug, Clone)]
pub struct OffscreenCanvas {
    inner: web_sys::OffscreenCanvas,
    pub scale_factor: f32,
    handle: u32,
}

#[allow(dead_code)]
impl OffscreenCanvas {
    // Create a new OffscreenCanvas instance
    pub const fn new(canvas: web_sys::OffscreenCanvas, scale_factor: f32, handle: u32) -> Self {
        Self {
            inner: canvas,
            scale_factor,
            handle,
        }
    }

    // Get the inner OffscreenCanvas and handle
    pub fn each(self) -> (web_sys::OffscreenCanvas, u32) {
        (self.inner, self.handle)
    }

    // Get the logical resolution of the offscreen canvas
    pub fn logical_resolution(&self) -> (f32, f32) {
        let width = self.inner.width();
        let height = self.inner.height();
        (width as f32, height as f32)
    }
}

// Implement From<&Canvas> for OffscreenCanvas
impl From<&Canvas> for OffscreenCanvas {
    fn from(value: &Canvas) -> Self {
        let offscreen = value.element.transfer_control_to_offscreen().unwrap();
        let handle = value.handle;
        Self::new(offscreen, value.scale_factor, handle)
    }
}

// Implement HasWindowHandle for OffscreenCanvasWrapper
impl raw_window_handle::HasWindowHandle for OffscreenCanvasWrapper {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        use raw_window_handle::{RawWindowHandle, WebOffscreenCanvasWindowHandle, WindowHandle};

        let value: &JsValue = &self.0 .0.inner;
        let obj: NonNull<std::ffi::c_void> = NonNull::from(value).cast();
        let handle = WebOffscreenCanvasWindowHandle::new(obj);
        let raw = RawWindowHandle::WebOffscreenCanvas(handle);
        unsafe { Ok(WindowHandle::borrow_raw(raw)) }
    }
}

// Implement HasDisplayHandle for OffscreenCanvasWrapper
impl raw_window_handle::HasDisplayHandle for OffscreenCanvasWrapper {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        use raw_window_handle::{DisplayHandle, RawDisplayHandle, WebDisplayHandle};
        let handle = WebDisplayHandle::new();
        let raw = RawDisplayHandle::Web(handle);
        unsafe { Ok(DisplayHandle::borrow_raw(raw)) }
    }
}
