use glib::{
    prelude::*,
    subclass::types::ObjectSubclass,
    translate::{FromGlibPtrNone, IntoGlib, IntoGlibPtr},
};

use crate::{
    display::ffi::AstalDisplaysDisplay,
    display_identifier::{ffi::AstalDisplaysDisplayIdentifier, DisplayIdentifier},
    display_match::ffi::AstalDisplaysDisplayMatch,
    display_update::{ffi::AstalDisplaysDisplayUpdate, DisplayUpdate},
    object_vec_to_ptr_array, write_error,
};

pub type AstalDisplaysManager = <super::imp::Manager as ObjectSubclass>::Instance;

#[unsafe(no_mangle)]
pub extern "C" fn astal_displays_manager_get_type() -> glib::ffi::GType {
    super::Manager::static_type().into_glib()
}

#[unsafe(no_mangle)]
pub extern "C" fn astal_displays_get_default() -> *mut AstalDisplaysManager {
    unsafe { super::Manager::get_default().into_glib_ptr() }
}

#[unsafe(no_mangle)]
pub extern "C" fn astal_displays_manager_get_default() -> *mut AstalDisplaysManager {
    unsafe { super::Manager::get_default().into_glib_ptr() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn astal_displays_manager_query(
    manager: *mut AstalDisplaysManager,
    n_results: *mut usize,
    error: *mut *mut glib::ffi::GError,
) -> *mut *mut AstalDisplaysDisplay {
    let manager = super::Manager::from_glib_none(manager);
    match manager.query() {
        Ok(items) => {
            if !n_results.is_null() {
                *n_results = items.len();
            }
            object_vec_to_ptr_array(items)
        }
        Err(err) => {
            if !n_results.is_null() {
                *n_results = 0;
            }
            write_error(error, err);
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn astal_displays_manager_get(
    manager: *mut AstalDisplaysManager,
    ids: *mut *mut AstalDisplaysDisplayIdentifier,
    n_ids: usize,
    n_results: *mut usize,
    error: *mut *mut glib::ffi::GError,
) -> *mut *mut AstalDisplaysDisplayMatch {
    let manager = super::Manager::from_glib_none(manager);
    let ids =
        crate::read_object_array::<DisplayIdentifier, AstalDisplaysDisplayIdentifier>(ids, n_ids);
    match manager.get(ids) {
        Ok(items) => {
            if !n_results.is_null() {
                *n_results = items.len();
            }
            object_vec_to_ptr_array(items)
        }
        Err(err) => {
            if !n_results.is_null() {
                *n_results = 0;
            }
            write_error(error, err);
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn astal_displays_manager_apply(
    manager: *mut AstalDisplaysManager,
    updates: *mut *mut AstalDisplaysDisplayUpdate,
    n_updates: usize,
    validate: bool,
    n_results: *mut usize,
    error: *mut *mut glib::ffi::GError,
) -> *mut *mut AstalDisplaysDisplayUpdate {
    let manager = super::Manager::from_glib_none(manager);
    let updates =
        crate::read_object_array::<DisplayUpdate, AstalDisplaysDisplayUpdate>(updates, n_updates);
    match manager.apply(updates, validate) {
        Ok(items) => {
            if !n_results.is_null() {
                *n_results = items.len();
            }
            object_vec_to_ptr_array(items)
        }
        Err(err) => {
            if !n_results.is_null() {
                *n_results = 0;
            }
            write_error(error, err);
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn astal_displays_manager_update(
    manager: *mut AstalDisplaysManager,
    updates: *mut *mut AstalDisplaysDisplayUpdate,
    n_updates: usize,
    n_results: *mut usize,
    error: *mut *mut glib::ffi::GError,
) -> *mut *mut AstalDisplaysDisplayUpdate {
    astal_displays_manager_apply(manager, updates, n_updates, false, n_results, error)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn astal_displays_manager_validate(
    manager: *mut AstalDisplaysManager,
    updates: *mut *mut AstalDisplaysDisplayUpdate,
    n_updates: usize,
    n_results: *mut usize,
    error: *mut *mut glib::ffi::GError,
) -> *mut *mut AstalDisplaysDisplayUpdate {
    astal_displays_manager_apply(manager, updates, n_updates, true, n_results, error)
}
