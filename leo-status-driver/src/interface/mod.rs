#[cfg(feature = "hidapi")]
mod hidapi;

#[cfg(feature = "hidapi")]
pub use hidapi::GpsdoHidApiInterface;
