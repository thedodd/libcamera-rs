use std::{ffi::CStr, ptr::NonNull};

use drm_fourcc::{DrmFormat, DrmFourcc, DrmModifier};
use libcamera_sys::*;

#[derive(Clone, Copy)]
pub struct PixelFormat(pub(crate) libcamera_pixel_format_t);

impl PixelFormat {
    pub const fn new(fourcc: u32, modifier: u64) -> Self {
        Self(libcamera_pixel_format_t { fourcc, modifier })
    }

    pub fn fourcc(&self) -> u32 {
        self.0.fourcc
    }

    pub fn set_fourcc(&mut self, fourcc: u32) {
        self.0.fourcc = fourcc;
    }

    pub fn modifier(&self) -> u64 {
        self.0.modifier
    }

    pub fn set_modifier(&mut self, modifier: u64) {
        self.0.modifier = modifier;
    }

    pub fn to_string(&self) -> String {
        let ptr = unsafe { libcamera_pixel_format_str(&self.0) };
        let out = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_string();
        unsafe { libc::free(ptr.cast()) };
        out
    }
}

impl core::fmt::Debug for PixelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl TryFrom<PixelFormat> for DrmFormat {
    type Error = drm_fourcc::UnrecognizedFourcc;

    fn try_from(value: PixelFormat) -> Result<Self, Self::Error> {
        let code = DrmFourcc::try_from(value.0.fourcc)?;
        let modifier = DrmModifier::from(value.0.modifier);
        Ok(DrmFormat { code, modifier })
    }
}

impl From<DrmFormat> for PixelFormat {
    fn from(f: DrmFormat) -> Self {
        PixelFormat::new(f.code as u32, f.modifier.into())
    }
}

pub struct PixelFormats {
    ptr: NonNull<libcamera_pixel_formats_t>,
}

impl PixelFormats {
    pub(crate) unsafe fn from_ptr(ptr: NonNull<libcamera_pixel_formats_t>) -> Self {
        Self { ptr }
    }

    pub fn len(&self) -> usize {
        unsafe { libcamera_pixel_formats_size(self.ptr.as_ptr()) as _ }
    }

    pub fn get(&self, index: usize) -> Option<PixelFormat> {
        if index >= self.len() {
            None
        } else {
            Some(unsafe { self.get_unchecked(index) })
        }
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> PixelFormat {
        PixelFormat(unsafe { libcamera_pixel_formats_get(self.ptr.as_ptr(), index as _) })
    }
}

impl<'d> IntoIterator for &'d PixelFormats {
    type Item = PixelFormat;

    type IntoIter = PixelFormatsIterator<'d>;

    fn into_iter(self) -> Self::IntoIter {
        PixelFormatsIterator {
            formats: self,
            index: 0,
        }
    }
}

impl Drop for PixelFormats {
    fn drop(&mut self) {
        unsafe { libcamera_pixel_formats_destroy(self.ptr.as_ptr()) }
    }
}

pub struct PixelFormatsIterator<'d> {
    formats: &'d PixelFormats,
    index: usize,
}

impl<'d> Iterator for PixelFormatsIterator<'d> {
    type Item = PixelFormat;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.formats.get(self.index) {
            self.index += 1;
            Some(next)
        } else {
            None
        }
    }
}
