//! GObject-Introspection bridge for the `displays` crate.
//!
//! By default this crate delegates to the real `displays::manager::DisplayManager`.
//! When built with the `faked` Cargo feature it swaps in a deterministic fake
//! backend instead, which is useful for GI smoke tests and TypeScript iteration
//! without touching real hardware.

pub mod backend;
pub mod display;
pub mod display_identifier;
pub mod display_match;
pub mod display_update;
pub mod enums;
pub mod error;
pub mod logical_display;
pub mod manager;
pub mod physical_display;
pub mod point;

use glib::{ffi::GError, translate::IntoGlibPtr};

pub(crate) unsafe fn write_error(error: *mut *mut GError, err: glib::Error) {
    if !error.is_null() {
        *error = err.into_glib_ptr();
    }
}

pub(crate) fn object_vec_to_ptr_array<T, P: 'static>(items: Vec<T>) -> *mut *mut P
where
    T: IntoGlibPtr<*mut P>,
{
    let mut ptrs = items
        .into_iter()
        .map(|item| unsafe { item.into_glib_ptr() })
        .collect::<Vec<_>>();
    let ptr = ptrs.as_mut_ptr();
    std::mem::forget(ptrs);
    ptr
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
