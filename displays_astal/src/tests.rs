use std::{env, process::Command, ptr};

use gio::prelude::ListModelExt;
use glib::{ffi::GError, prelude::ObjectExt, MainLoop, Object};

use crate::{
    display::Display,
    display_identifier::DisplayIdentifier,
    display_update::DisplayUpdate,
    manager::{
        ffi::{
            astal_displays_manager_get_default, astal_displays_manager_query_async,
            astal_displays_manager_query_finish,
        },
        Manager,
    },
    physical_display::{PhysicalDisplay, PhysicalDisplayUpdateContent},
};

const FFI_QUERY_HELPER_ENV: &str = "ASTAL_DISPLAYS_TEST_HELPER_FFI_QUERY";
const FFI_QUERY_TEST_NAME: &str = "tests::ffi_query_result_array_uses_glib_owned_memory";
const REAL_BACKLIGHT_ENABLE_ENV: &str = "ASTAL_DISPLAYS_REAL_BACKLIGHT_TEST";
const REAL_BACKLIGHT_HELPER_ENV: &str = "ASTAL_DISPLAYS_TEST_HELPER_REAL_BACKLIGHT";
const REAL_BACKLIGHT_TEST_NAME: &str = "tests::real_backlight_update_round_trips_without_abort";

fn run_in_subprocess(helper_env: &str, test_name: &str, extra_env: &[(&str, &str)]) {
    let mut command = Command::new(env::current_exe().expect("test binary path"));
    command
        .env(helper_env, "1")
        .arg("--exact")
        .arg(test_name)
        .arg("--nocapture");

    for (key, value) in extra_env {
        command.env(key, value);
    }

    let status = command.status().expect("spawn child test process");
    assert!(status.success(), "child process exited with {status}");
}

fn wait_for_async_result<T, F>(register: F) -> Result<T, glib::Error>
where
    F: FnOnce(Box<dyn FnOnce(Result<T, glib::Error>)>),
    T: 'static,
{
    let main_loop = MainLoop::new(None, false);
    let main_loop_clone = main_loop.clone();
    let result = std::rc::Rc::new(std::cell::RefCell::new(None));
    let result_clone = result.clone();

    register(Box::new(move |value| {
        *result_clone.borrow_mut() = Some(value);
        main_loop_clone.quit();
    }));

    main_loop.run();
    let outcome = result
        .borrow_mut()
        .take()
        .expect("async callback should store a result");
    outcome
}

fn run_query_and_free_like_gi_consumer() {
    struct QueryOutcome {
        results: *mut *mut crate::display::ffi::AstalDisplaysDisplay,
        result_len: usize,
        error: *mut GError,
    }

    let main_loop = MainLoop::new(None, false);
    let loop_clone = main_loop.clone();

    unsafe extern "C" fn query_ready_and_quit(
        source_object: *mut glib::gobject_ffi::GObject,
        result: *mut gio::ffi::GAsyncResult,
        user_data: glib::ffi::gpointer,
    ) {
        let pair = user_data as *mut (MainLoop, Option<QueryOutcome>);
        unsafe {
            let source_object = source_object as *mut crate::manager::ffi::AstalDisplaysManager;
            let mut result_len = 0usize;
            let mut error = ptr::null_mut();
            let results = astal_displays_manager_query_finish(
                source_object,
                result,
                &mut result_len,
                &mut error,
            );
            (*pair).1 = Some(QueryOutcome {
                results,
                result_len,
                error,
            });
            (*pair).0.quit();
        }
    }

    unsafe {
        let manager = astal_displays_manager_get_default();
        assert!(
            !manager.is_null(),
            "default manager pointer must be non-null"
        );

        let mut pair = (loop_clone, None);
        astal_displays_manager_query_async(
            manager,
            ptr::null_mut(),
            Some(query_ready_and_quit),
            &mut pair as *mut _ as _,
        );
        main_loop.run();

        let QueryOutcome {
            results,
            result_len,
            error,
        } = pair.1.take().expect("async query callback should run");

        assert!(error.is_null(), "query returned a GLib error pointer");

        if !results.is_null() {
            for index in 0..result_len {
                let item: *mut crate::display::ffi::AstalDisplaysDisplay = *results.add(index);
                assert!(!item.is_null(), "query returned a null display pointer");
                glib::gobject_ffi::g_object_unref(item as *mut _);
            }

            glib::ffi::g_free(results as *mut _);
        }

        glib::gobject_ffi::g_object_unref(manager as *mut _);
    }
}

fn get_property<T: for<'a> glib::value::FromValue<'a> + 'static>(
    object: &impl glib::object::IsA<glib::Object>,
    key: &str,
) -> T {
    object.property::<T>(key)
}

fn built_in_backlight_display(displays: Vec<Display>) -> Option<(DisplayIdentifier, u32)> {
    displays.into_iter().find_map(|display| {
        let identifier = get_property::<DisplayIdentifier>(&display, "id");
        let serial =
            get_property::<Option<String>>(&identifier, "serial-number").unwrap_or_default();
        if !serial.is_empty() {
            return None;
        }

        let physical = get_property::<Option<PhysicalDisplay>>(&display, "physical")?;
        if !get_property::<bool>(&physical, "has-brightness") {
            return None;
        }
        let brightness = get_property::<u32>(&physical, "brightness");

        Some((identifier, brightness))
    })
}

fn run_real_backlight_round_trip() {
    let manager = Manager::get_default();
    let displays = wait_for_async_result(|done| {
        manager.query_async(None::<&gio::Cancellable>, move |manager, result| {
            done(manager.query_finish(result));
        });
    })
    .expect("query displays before update");
    let Some((identifier, current_brightness)) = built_in_backlight_display(displays) else {
        eprintln!("skipping real backlight test: no built-in backlight display found");
        return;
    };

    let update_content = Object::builder::<PhysicalDisplayUpdateContent>()
        .property("has-brightness", true)
        .property("brightness", current_brightness)
        .build();
    let update = Object::builder::<DisplayUpdate>()
        .property("id", identifier)
        .property("physical", update_content)
        .build();

    let results = wait_for_async_result(|done| {
        manager.update_async(
            vec![update],
            None::<&gio::Cancellable>,
            move |manager, result| {
                done(manager.update_finish(result));
            },
        );
    })
    .expect("update current backlight brightness");
    assert!(
        results.len() == 1,
        "expected one update result, got {}",
        results.len()
    );

    let applied = get_property::<gio::ListStore>(&results[0], "applied");
    let failed = get_property::<gio::ListStore>(&results[0], "failed");
    assert_eq!(applied.n_items(), 1, "expected one applied display");
    assert_eq!(failed.n_items(), 0, "expected no failed displays");

    let refreshed = wait_for_async_result(|done| {
        manager.query_async(None::<&gio::Cancellable>, move |manager, result| {
            done(manager.query_finish(result));
        });
    })
    .expect("query displays after update");
    let Some((_, refreshed_brightness)) = built_in_backlight_display(refreshed) else {
        panic!("built-in backlight display disappeared after update");
    };

    assert_eq!(
        refreshed_brightness, current_brightness,
        "no-op backlight update should preserve brightness"
    );
}

#[cfg(feature = "faked")]
#[test]
fn fake_update_result_contains_applied_matches() {
    let update = Object::builder::<DisplayUpdate>()
        .property(
            "id",
            DisplayIdentifier::new(Some("Dell U2720Q"), Some("DELLA1")),
        )
        .property(
            "physical",
            Object::builder::<PhysicalDisplayUpdateContent>()
                .property("has-brightness", true)
                .property("brightness", 50u32)
                .build(),
        )
        .build();

    let manager = Manager::get_default();
    let results = wait_for_async_result(|done| {
        manager.update_async(
            vec![update],
            None::<&gio::Cancellable>,
            move |manager, result| {
                done(manager.update_finish(result));
            },
        );
    })
    .expect("update fake displays");

    assert_eq!(results.len(), 1);

    let applied = get_property::<gio::ListStore>(&results[0], "applied");
    let failed = get_property::<gio::ListStore>(&results[0], "failed");

    assert_eq!(applied.n_items(), 1);
    assert_eq!(failed.n_items(), 0);
}

#[test]
fn ffi_query_result_array_uses_glib_owned_memory() {
    if env::var_os(FFI_QUERY_HELPER_ENV).is_some() {
        run_query_and_free_like_gi_consumer();
        return;
    }

    run_in_subprocess(FFI_QUERY_HELPER_ENV, FFI_QUERY_TEST_NAME, &[]);
}

#[cfg(target_os = "linux")]
#[test]
fn real_backlight_update_round_trips_without_abort() {
    if env::var_os(REAL_BACKLIGHT_ENABLE_ENV).is_none() {
        eprintln!("skipping real backlight test; set {REAL_BACKLIGHT_ENABLE_ENV}=1 to enable it");
        return;
    }

    if env::var_os(REAL_BACKLIGHT_HELPER_ENV).is_some() {
        run_real_backlight_round_trip();
        return;
    }

    run_in_subprocess(
        REAL_BACKLIGHT_HELPER_ENV,
        REAL_BACKLIGHT_TEST_NAME,
        &[(REAL_BACKLIGHT_ENABLE_ENV, "1")],
    );
}
