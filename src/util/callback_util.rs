

use std::{
    mem::transmute,
    thread,
    // thread::sleep,
    time::Duration,
    os::raw::{c_int, /*c_void*/},
    sync::atomic::{AtomicBool, Ordering, AtomicPtr},
};
use jl_sys::{
    jl_adopt_thread,
    jlrs_gc_unsafe_enter,
    jlrs_gc_unsafe_leave,
    jlrs_get_ptls_states,
    jlrs_gc_safe_enter,
};
use tokio::{self, runtime};


type JuliaCallback = unsafe extern "C" fn(data: *const u8) -> c_int;
extern "C" fn default_cb(_s: *const u8) -> c_int { 0 }

static IS_RUN: AtomicBool = AtomicBool::new(false);
static CALLBACK: AtomicPtr<()> = AtomicPtr::new(default_cb as *mut ());


#[no_mangle]
pub extern "C" fn set_callback(callback: JuliaCallback) {
    CALLBACK.store(callback as *mut (), Ordering::Release);
    println!("Callback function has been set from Julia");
}

#[no_mangle]
pub unsafe extern "C" fn run_callback(data_ptr: *const u8) -> c_int {
    let ptls = jlrs_get_ptls_states();
    let old_state = jlrs_gc_unsafe_enter(ptls);
    let callback: JuliaCallback = unsafe { transmute(CALLBACK.load(Ordering::Acquire)) };
    let status = callback(data_ptr);
    jlrs_gc_unsafe_leave(ptls, old_state);
    status
}

#[no_mangle]
pub extern "C" fn work_start() {
    if IS_RUN.load(Ordering::Acquire) {
        println!("Work is already running");
        return;
    }

    IS_RUN.store(true, Ordering::Release);
    let _th = thread::spawn(|| {
        let rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .on_thread_start(|| {
                unsafe {
                    jl_adopt_thread();
                    jlrs_gc_safe_enter(jlrs_get_ptls_states());
                }
                println!("Thread #{:?} Initialized", thread::current().id());
            })
            .on_thread_stop(|| {
                println!("Thread #{:?} Stopped", thread::current().id());
            })
            .build().expect("Failed to create Tokio runtime");

        rt.block_on(async {
            let mut i = 0;
            while IS_RUN.load(Ordering::Acquire) {
                rt.spawn(async {
                    let mut buf = Vec::with_capacity(64);
                    buf.extend_from_slice(b"\x08Hello hi\x00");
                    unsafe { run_callback(buf.as_ptr()) };
                });
                i += 1;
                tokio::time::sleep(Duration::from_millis(1)).await;
                if i > 1_000_000 { 
                    println!("i > 1_000_000");
                    break; 
                }
            }
        })
    });
}

#[no_mangle]
pub extern "C" fn work_stop() {
    IS_RUN.store(false, Ordering::Release);
    println!("Work has been signaled to stop");
}

#[no_mangle]
pub extern "C" fn is_work_running() -> i32 {
    if IS_RUN.load(Ordering::Acquire) {
        1
    } else {
        0
    }
}













