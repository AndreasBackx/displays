#![doc = include_str!("../docs/crate.md")]
#![doc = ""]
#![doc = include_str!("../docs/fragments/astal-backend-selection.md")]
#![doc = ""]
#![doc = include_str!("../docs/fragments/astal-build.md")]
#![doc = ""]
#![doc = include_str!("../docs/fragments/astal-smoke-test.md")]
#![doc = ""]
#![doc = include_str!("../docs/fragments/displays-astal.md")]
#![doc = ""]
#![doc = include_str!("../docs/fragments/astal-api-notes.md")]

pub mod backend;
pub mod display;
pub mod display_identifier;
pub mod display_match;
pub mod display_update;
pub mod display_update_result;
pub mod enums;
pub mod error;
pub mod failed_display_update;
pub mod logical_display;
pub mod manager;
pub mod physical_display;
pub mod point;
pub mod size;

#[cfg(test)]
mod tests;

use std::{mem::size_of, ptr};

use glib::{
    ffi::{g_malloc, GError},
    translate::IntoGlibPtr,
};

pub(crate) unsafe fn write_error(error: *mut *mut GError, err: glib::Error) {
    if !error.is_null() {
        *error = err.into_glib_ptr();
    }
}

pub(crate) fn object_vec_to_ptr_array<T, P: 'static>(items: Vec<T>) -> *mut *mut P
where
    T: IntoGlibPtr<*mut P>,
{
    if items.is_empty() {
        return ptr::null_mut();
    }

    let ptrs = items
        .into_iter()
        .map(|item| unsafe { item.into_glib_ptr() })
        .collect::<Vec<_>>();

    let bytes = ptrs.len() * size_of::<*mut P>();
    let array = unsafe { g_malloc(bytes) as *mut *mut P };

    unsafe {
        ptr::copy_nonoverlapping(ptrs.as_ptr(), array, ptrs.len());
    }

    array
}

pub(crate) unsafe fn read_object_array<T, P: 'static>(items: *mut *mut P, len: usize) -> Vec<T>
where
    T: glib::translate::FromGlibPtrNone<*mut P>,
{
    if items.is_null() || len == 0 {
        return Vec::new();
    }

    std::slice::from_raw_parts(items as *const *mut P, len)
        .iter()
        .map(|item| T::from_glib_none(*item))
        .collect()
}
