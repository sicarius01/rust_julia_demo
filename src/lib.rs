

pub mod lib_gemini;
// pub mod lib_gpt;

// pub mod gc_unsafe_call;





// // use std::ffi::{CStr, CString};
// use std::thread;
// use std::time::Duration;
// // use std::sync::{Mutex, OnceLock};
// use std::sync::atomic::{AtomicBool, Ordering, AtomicPtr};

// // use futures_util::{StreamExt, SinkExt};
// // use serde::{Deserialize, Serialize};
// // use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
// // use url::Url;


// type JuliaCallback = extern "C" fn(*const u8);
// extern "C" fn default_cb(_s: *const u8) {}


// static CALLBACK: AtomicPtr<()> = AtomicPtr::new(default_cb as *mut ());
// static WORK_RUNNING: AtomicBool = AtomicBool::new(false);


// #[no_mangle]
// pub extern "C" fn set_callback(callback: JuliaCallback) {
//     CALLBACK.store(callback as *mut (), Ordering::Release);
//     println!("Callback function has been set from Julia");
// }

// fn get_buf_with_cap(cap: usize) -> Vec<u8> {
//     Vec::with_capacity(cap)
// }

// fn send_to_julia() {
//     let mut buf = get_buf_with_cap(64);
//     buf.extend_from_slice(b"\x05Hello");
//     let p = CALLBACK.load(Ordering::Acquire);
//     let f: JuliaCallback = unsafe { std::mem::transmute(p) };
//     f(buf.as_ptr());
// }

// #[no_mangle]
// pub extern "C" fn start_work() -> i32 {
//     if WORK_RUNNING.swap(true, Ordering::SeqCst) {
//         return -1;
//     }
//     let _handle = thread::spawn(move || {
//         let rt = tokio::runtime::Builder::new_multi_thread()
//             .worker_threads(4)
//             .enable_all()
//             .on_thread_start(|| {
//                 println!("Tokio thread started");
//             })
//             .on_thread_stop(|| {
//                 println!("Tokio thread stopped");
//             })
//             .build()
//             .expect("Failed to create Tokio runtime");
        
//         rt.block_on(async {
//             let mut _i: i64 = 0;
//             let period = Duration::from_secs(1);

//             while WORK_RUNNING.load(Ordering::SeqCst) {
//                 send_to_julia();
//                 _i += 1;
//                 tokio::time::sleep(period).await;
//             }
//             println!("END while in tr");
//             send_to_julia();
//         });
//     });
//     0
// }

// #[no_mangle]
// pub extern "C" fn stop_work() {
//     WORK_RUNNING.store(false, Ordering::SeqCst);
//     println!("Work has been signaled to stop");
// }

// #[no_mangle]
// pub extern "C" fn is_work_running() -> i32 {
//     if WORK_RUNNING.load(Ordering::Relaxed) {
//         1
//     } else {
//         0
//     }
// }





