use std::{ffi::c_void, ptr};

use gio::{ffi as gio_ffi, prelude::*, AsyncResult, Cancellable, Task};
use glib::{
    subclass::types::ObjectSubclass,
    translate::{FromGlibPtrNone, IntoGlib, IntoGlibPtr, ToGlibPtr},
};

use crate::{
    backend::types::{
        DisplayData, DisplayIdentifierData, DisplayMatchData, DisplayUpdateData,
        DisplayUpdateResultData,
    },
    display::ffi::DisplaysAstalDisplay,
    display_identifier::{ffi::DisplaysAstalDisplayIdentifier, DisplayIdentifier},
    display_match::ffi::DisplaysAstalDisplayMatch,
    display_update::{ffi::DisplaysAstalDisplayUpdate, DisplayUpdate},
    display_update_result::ffi::DisplaysAstalDisplayUpdateResult,
    object_vec_to_ptr_array, write_error,
};

pub type DisplaysAstalManager = <super::imp::Manager as ObjectSubclass>::Instance;

type QueryPayload = Vec<DisplayData>;
type GetPayload = Vec<DisplayMatchData>;
type UpdatePayload = Vec<DisplayUpdateResultData>;

enum TaskPayload {
    Query(Result<QueryPayload, glib::Error>),
    Get(Result<GetPayload, glib::Error>),
    Update(Result<UpdatePayload, glib::Error>),
}

type TaskCallback = dyn FnOnce(&super::Manager, &gio::AsyncResult) + 'static;

#[unsafe(no_mangle)]
pub extern "C" fn displays_astal_manager_get_type() -> glib::ffi::GType {
    super::Manager::static_type().into_glib()
}

#[unsafe(no_mangle)]
pub extern "C" fn displays_astal_get_default() -> *mut DisplaysAstalManager {
    unsafe { super::Manager::get_default().into_glib_ptr() }
}

#[unsafe(no_mangle)]
pub extern "C" fn displays_astal_manager_get_default() -> *mut DisplaysAstalManager {
    unsafe { super::Manager::get_default().into_glib_ptr() }
}

pub(crate) fn spawn_query_task<P, F>(manager: &super::Manager, cancellable: Option<&P>, callback: F)
where
    P: IsA<Cancellable>,
    F: FnOnce(&super::Manager, &gio::AsyncResult) + 'static,
{
    let task = create_task(manager, cancellable, callback);
    task.run_in_thread(
        move |task, _manager: Option<&super::Manager>, _cancellable| {
            return_task_payload(task, TaskPayload::Query(super::Manager::query_in_thread()));
        },
    );
}

pub(crate) fn spawn_get_task<P, F>(
    manager: &super::Manager,
    ids: Vec<DisplayIdentifierData>,
    cancellable: Option<&P>,
    callback: F,
) where
    P: IsA<Cancellable>,
    F: FnOnce(&super::Manager, &gio::AsyncResult) + 'static,
{
    let task = create_task(manager, cancellable, callback);
    task.run_in_thread(
        move |task, _manager: Option<&super::Manager>, _cancellable| {
            return_task_payload(task, TaskPayload::Get(super::Manager::get_in_thread(ids)));
        },
    );
}

pub(crate) fn spawn_apply_task<P, F>(
    manager: &super::Manager,
    updates: Vec<DisplayUpdateData>,
    validate: bool,
    cancellable: Option<&P>,
    callback: F,
) where
    P: IsA<Cancellable>,
    F: FnOnce(&super::Manager, &gio::AsyncResult) + 'static,
{
    let task = create_task(manager, cancellable, callback);
    task.run_in_thread(
        move |task, _manager: Option<&super::Manager>, _cancellable| {
            return_task_payload(
                task,
                TaskPayload::Update(super::Manager::apply_in_thread(updates, validate)),
            );
        },
    );
}

pub(crate) fn take_query_result(
    manager: &super::Manager,
    result: &impl IsA<AsyncResult>,
) -> Result<QueryPayload, glib::Error> {
    match take_task_payload(manager, result)? {
        TaskPayload::Query(payload) => payload,
        _ => Err(glib::Error::new(
            gio::IOErrorEnum::InvalidArgument,
            "async result did not contain a query payload",
        )),
    }
}

pub(crate) fn take_get_result(
    manager: &super::Manager,
    result: &impl IsA<AsyncResult>,
) -> Result<GetPayload, glib::Error> {
    match take_task_payload(manager, result)? {
        TaskPayload::Get(payload) => payload,
        _ => Err(glib::Error::new(
            gio::IOErrorEnum::InvalidArgument,
            "async result did not contain a get payload",
        )),
    }
}

pub(crate) fn take_update_result(
    manager: &super::Manager,
    result: &impl IsA<AsyncResult>,
) -> Result<UpdatePayload, glib::Error> {
    match take_task_payload(manager, result)? {
        TaskPayload::Update(payload) => payload,
        _ => Err(glib::Error::new(
            gio::IOErrorEnum::InvalidArgument,
            "async result did not contain an update payload",
        )),
    }
}

fn create_task<P, F>(manager: &super::Manager, cancellable: Option<&P>, callback: F) -> Task<bool>
where
    P: IsA<Cancellable>,
    F: FnOnce(&super::Manager, &gio::AsyncResult) + 'static,
{
    let cancellable = cancellable
        .map(|cancellable| cancellable.as_ref().to_glib_none().0)
        .unwrap_or(ptr::null_mut());
    create_trampolined_task(manager, cancellable, Box::new(callback))
}

fn return_task_payload(task: Task<bool>, payload: TaskPayload) {
    unsafe {
        gio_ffi::g_task_return_pointer(
            task.to_glib_none().0,
            Box::into_raw(Box::new(payload)) as *mut c_void,
            Some(free_task_payload),
        );
    }
}

fn take_task_payload(
    manager: &super::Manager,
    result: &impl IsA<AsyncResult>,
) -> Result<TaskPayload, glib::Error> {
    if !Task::<bool>::is_valid(result, Some(manager)) {
        return Err(glib::Error::new(
            gio::IOErrorEnum::InvalidArgument,
            "result does not belong to DisplaysAstal.Manager",
        ));
    }

    let task = result
        .as_ref()
        .clone()
        .downcast::<Task<bool>>()
        .map_err(|_| {
            glib::Error::new(
                gio::IOErrorEnum::InvalidArgument,
                "result is not a GTask produced by DisplaysAstal.Manager",
            )
        })?;

    let mut error = ptr::null_mut();
    let payload = unsafe { gio_ffi::g_task_propagate_pointer(task.to_glib_none().0, &mut error) };
    if !error.is_null() {
        return Err(unsafe { glib::translate::from_glib_full(error) });
    }

    if payload.is_null() {
        return Err(glib::Error::new(
            gio::IOErrorEnum::Failed,
            "async task completed without a payload",
        ));
    }

    Ok(*unsafe { Box::from_raw(payload as *mut TaskPayload) })
}

unsafe extern "C" fn free_task_payload(data: *mut c_void) {
    if !data.is_null() {
        let _ = Box::from_raw(data as *mut TaskPayload);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn displays_astal_manager_query_async(
    manager: *mut DisplaysAstalManager,
    cancellable: *mut gio_ffi::GCancellable,
    callback: gio_ffi::GAsyncReadyCallback,
    user_data: glib::ffi::gpointer,
) {
    let manager = super::Manager::from_glib_none(manager);
    spawn_ffi_query_task(&manager, cancellable, callback, user_data);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn displays_astal_manager_query_finish(
    manager: *mut DisplaysAstalManager,
    result: *mut gio_ffi::GAsyncResult,
    n_results: *mut usize,
    error: *mut *mut glib::ffi::GError,
) -> *mut *mut DisplaysAstalDisplay {
    let manager = super::Manager::from_glib_none(manager);
    let result = gio::AsyncResult::from_glib_none(result);
    match manager.query_finish(&result) {
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
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn displays_astal_manager_get_async(
    manager: *mut DisplaysAstalManager,
    ids: *mut *mut DisplaysAstalDisplayIdentifier,
    n_ids: usize,
    cancellable: *mut gio_ffi::GCancellable,
    callback: gio_ffi::GAsyncReadyCallback,
    user_data: glib::ffi::gpointer,
) {
    let manager = super::Manager::from_glib_none(manager);
    let ids =
        crate::read_object_array::<DisplayIdentifier, DisplaysAstalDisplayIdentifier>(ids, n_ids)
            .into_iter()
            .map(|id| id.to_data())
            .collect();
    spawn_ffi_get_task(&manager, ids, cancellable, callback, user_data);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn displays_astal_manager_get_finish(
    manager: *mut DisplaysAstalManager,
    result: *mut gio_ffi::GAsyncResult,
    n_results: *mut usize,
    error: *mut *mut glib::ffi::GError,
) -> *mut *mut DisplaysAstalDisplayMatch {
    let manager = super::Manager::from_glib_none(manager);
    let result = gio::AsyncResult::from_glib_none(result);
    match manager.get_finish(&result) {
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
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn displays_astal_manager_apply_async(
    manager: *mut DisplaysAstalManager,
    updates: *mut *mut DisplaysAstalDisplayUpdate,
    n_updates: usize,
    validate: bool,
    cancellable: *mut gio_ffi::GCancellable,
    callback: gio_ffi::GAsyncReadyCallback,
    user_data: glib::ffi::gpointer,
) {
    let manager = super::Manager::from_glib_none(manager);
    let updates =
        crate::read_object_array::<DisplayUpdate, DisplaysAstalDisplayUpdate>(updates, n_updates)
            .into_iter()
            .map(|update| update.to_data())
            .collect();
    spawn_ffi_apply_task(
        &manager,
        updates,
        validate,
        cancellable,
        callback,
        user_data,
    );
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn displays_astal_manager_apply_finish(
    manager: *mut DisplaysAstalManager,
    result: *mut gio_ffi::GAsyncResult,
    n_results: *mut usize,
    error: *mut *mut glib::ffi::GError,
) -> *mut *mut DisplaysAstalDisplayUpdateResult {
    let manager = super::Manager::from_glib_none(manager);
    let result = gio::AsyncResult::from_glib_none(result);
    match manager.apply_finish(&result) {
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
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn displays_astal_manager_update_async(
    manager: *mut DisplaysAstalManager,
    updates: *mut *mut DisplaysAstalDisplayUpdate,
    n_updates: usize,
    cancellable: *mut gio_ffi::GCancellable,
    callback: gio_ffi::GAsyncReadyCallback,
    user_data: glib::ffi::gpointer,
) {
    displays_astal_manager_apply_async(
        manager,
        updates,
        n_updates,
        false,
        cancellable,
        callback,
        user_data,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn displays_astal_manager_update_finish(
    manager: *mut DisplaysAstalManager,
    result: *mut gio_ffi::GAsyncResult,
    n_results: *mut usize,
    error: *mut *mut glib::ffi::GError,
) -> *mut *mut DisplaysAstalDisplayUpdateResult {
    displays_astal_manager_apply_finish(manager, result, n_results, error)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn displays_astal_manager_validate_async(
    manager: *mut DisplaysAstalManager,
    updates: *mut *mut DisplaysAstalDisplayUpdate,
    n_updates: usize,
    cancellable: *mut gio_ffi::GCancellable,
    callback: gio_ffi::GAsyncReadyCallback,
    user_data: glib::ffi::gpointer,
) {
    displays_astal_manager_apply_async(
        manager,
        updates,
        n_updates,
        true,
        cancellable,
        callback,
        user_data,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn displays_astal_manager_validate_finish(
    manager: *mut DisplaysAstalManager,
    result: *mut gio_ffi::GAsyncResult,
    n_results: *mut usize,
    error: *mut *mut glib::ffi::GError,
) -> *mut *mut DisplaysAstalDisplayUpdateResult {
    displays_astal_manager_apply_finish(manager, result, n_results, error)
}

fn spawn_ffi_query_task(
    manager: &super::Manager,
    cancellable: *mut gio_ffi::GCancellable,
    callback: gio_ffi::GAsyncReadyCallback,
    user_data: glib::ffi::gpointer,
) {
    let task = create_ffi_task(manager, cancellable, callback, user_data);
    task.run_in_thread(
        move |task, _manager: Option<&super::Manager>, _cancellable| {
            return_task_payload(task, TaskPayload::Query(super::Manager::query_in_thread()));
        },
    );
}

fn spawn_ffi_get_task(
    manager: &super::Manager,
    ids: Vec<DisplayIdentifierData>,
    cancellable: *mut gio_ffi::GCancellable,
    callback: gio_ffi::GAsyncReadyCallback,
    user_data: glib::ffi::gpointer,
) {
    let task = create_ffi_task(manager, cancellable, callback, user_data);
    task.run_in_thread(
        move |task, _manager: Option<&super::Manager>, _cancellable| {
            return_task_payload(task, TaskPayload::Get(super::Manager::get_in_thread(ids)));
        },
    );
}

fn spawn_ffi_apply_task(
    manager: &super::Manager,
    updates: Vec<DisplayUpdateData>,
    validate: bool,
    cancellable: *mut gio_ffi::GCancellable,
    callback: gio_ffi::GAsyncReadyCallback,
    user_data: glib::ffi::gpointer,
) {
    let task = create_ffi_task(manager, cancellable, callback, user_data);
    task.run_in_thread(
        move |task, _manager: Option<&super::Manager>, _cancellable| {
            return_task_payload(
                task,
                TaskPayload::Update(super::Manager::apply_in_thread(updates, validate)),
            );
        },
    );
}

fn create_ffi_task(
    manager: &super::Manager,
    cancellable: *mut gio_ffi::GCancellable,
    callback: gio_ffi::GAsyncReadyCallback,
    user_data: glib::ffi::gpointer,
) -> Task<bool> {
    unsafe {
        glib::translate::from_glib_full(gio_ffi::g_task_new(
            manager.as_ptr() as *mut _,
            cancellable,
            callback,
            user_data,
        ))
    }
}

fn create_trampolined_task(
    manager: &super::Manager,
    cancellable: *mut gio_ffi::GCancellable,
    callback: Box<TaskCallback>,
) -> Task<bool> {
    unsafe extern "C" fn rust_callback_trampoline(
        source_object: *mut glib::gobject_ffi::GObject,
        result: *mut gio_ffi::GAsyncResult,
        user_data: glib::ffi::gpointer,
    ) {
        let callback: Box<Box<TaskCallback>> = unsafe { Box::from_raw(user_data as *mut _) };
        let manager =
            unsafe { super::Manager::from_glib_none(source_object as *mut DisplaysAstalManager) };
        let result = unsafe { gio::AsyncResult::from_glib_none(result) };
        callback(&manager, &result);
    }

    unsafe {
        glib::translate::from_glib_full(gio_ffi::g_task_new(
            manager.as_ptr() as *mut _,
            cancellable,
            Some(rust_callback_trampoline),
            Box::into_raw(Box::new(callback)) as *mut _,
        ))
    }
}
