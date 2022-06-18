use std::sync::atomic::AtomicBool;

use lazy_static::lazy_static;
use libloading::{Library, Symbol};

/// The dynamic bindings for KDMAPI
pub struct KDMAPIBinds {
    is_kdmapi_available: Symbol<'static, unsafe extern "C" fn() -> bool>,
    initialize_kdmapi_stream: Symbol<'static, unsafe extern "C" fn() -> i32>,
    terminate_kdmapi_stream: Symbol<'static, unsafe extern "C" fn() -> i32>,
    reset_kdmapi_stream: Symbol<'static, unsafe extern "C" fn()>,
    send_direct_data: Symbol<'static, unsafe extern "C" fn(u32) -> u32>,
    send_direct_data_no_buf: Symbol<'static, unsafe extern "C" fn(u32) -> u32>,
    is_stream_open: AtomicBool,
}

impl KDMAPIBinds {
    /// Calls `IsKDMAPIAvailable`
    pub fn is_kdmapi_available(&self) -> bool {
        unsafe { (self.is_kdmapi_available)() }
    }

    /// Calls `InitializeKDMAPIStream` and returns a stream struct with access
    /// to the stream functions.
    ///
    /// Automatically calls `TerminateKDMAPIStream` when dropped.
    ///
    /// Errors if multiple streams are opened in parallel.
    pub fn open_stream(&'static self) -> KDMAPIStream {
        if self
            .is_stream_open
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            panic!("KDMAPI stream is already open");
        }
        unsafe {
            let result = (self.initialize_kdmapi_stream)();
            if result == 0 {
                panic!("Failed to initialize KDMAPI stream");
            }
            KDMAPIStream { binds: &self }
        }
    }
}

fn load_kdmapi_lib() -> Library {
    unsafe { Library::new("OmniMIDI\\OmniMIDI").unwrap() }
}

fn load_kdmapi_binds(lib: &'static Library) -> KDMAPIBinds {
    unsafe {
        KDMAPIBinds {
            is_kdmapi_available: lib.get(b"IsKDMAPIAvailable").unwrap(),
            initialize_kdmapi_stream: lib.get(b"InitializeKDMAPIStream").unwrap(),
            terminate_kdmapi_stream: lib.get(b"TerminateKDMAPIStream").unwrap(),
            reset_kdmapi_stream: lib.get(b"ResetKDMAPIStream").unwrap(),
            send_direct_data: lib.get(b"SendDirectData").unwrap(),
            send_direct_data_no_buf: lib.get(b"SendDirectDataNoBuf").unwrap(),
            is_stream_open: AtomicBool::new(false),
        }
    }
}

/// Struct that provides access to KDMAPI's stream functions
///
/// Automatically calls `TerminateKDMAPIStream` when dropped.
pub struct KDMAPIStream {
    binds: &'static KDMAPIBinds,
}

impl KDMAPIStream {
    /// Calls `ResetKDMAPIStream`
    pub fn reset(&self) {
        unsafe {
            (self.binds.reset_kdmapi_stream)();
        }
    }

    /// Calls `SendDirectData`
    pub fn send_direct_data(&self, data: u32) -> u32 {
        unsafe { (self.binds.send_direct_data)(data) }
    }

    /// Calls `SendDirectDataNoBuf`
    pub fn send_direct_data_no_buf(&self, data: u32) -> u32 {
        unsafe { (self.binds.send_direct_data_no_buf)(data) }
    }
}

impl Drop for KDMAPIStream {
    fn drop(&mut self) {
        unsafe {
            (self.binds.terminate_kdmapi_stream)();
        }
        self.binds
            .is_stream_open
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

lazy_static! {
    static ref KDMAPI_LIB: Library = load_kdmapi_lib();

    /// The dynamic library for KDMAPI. Is loaded when this field is accessed.
    pub static ref KDMAPI: KDMAPIBinds = load_kdmapi_binds(&KDMAPI_LIB);
}
