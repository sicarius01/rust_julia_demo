
use std::os::raw::{c_int, c_void};
use std::sync::atomic::{Ordering, AtomicPtr};
// jl-sys에서 필요한 C 함수들을 모두 가져옵니다.
use jl_sys::{
    jlrs_gc_unsafe_enter, 
    jlrs_gc_unsafe_leave, 
    jlrs_get_ptls_states
};

// Julia에서 넘어올 C 함수 포인터의 타입 정의
type JuliaCallback = unsafe extern "C" fn(data: *const u8) -> c_int;

extern "C" fn default_cb(_s: *const u8) {}
static CALLBACK: AtomicPtr<()> = AtomicPtr::new(default_cb as *mut ());

#[no_mangle]
pub extern "C" fn set_callback(callback: JuliaCallback) {
    CALLBACK.store(callback as *mut (), Ordering::Release);
    println!("Callback function has been set from Julia");
}

#[no_mangle]
pub unsafe extern "C" fn run_callback_with_jl_sys(
    callback_ptr: *mut c_void,
    data_ptr: *const u8,
) -> c_int {
    // 1. 현재 스레드의 상태 포인터(ptls)를 가져옵니다.
    let ptls = jlrs_get_ptls_states();
    
    // 2. GC-unsafe 영역으로 진입하고, 이전 상태(old_state)를 저장합니다.
    let old_state = jlrs_gc_unsafe_enter(ptls);

    // 3. C 포인터를 Rust 함수 타입으로 변환하고 콜백을 호출합니다.
    let callback = std::mem::transmute::<*mut c_void, JuliaCallback>(callback_ptr);
    let status = callback(data_ptr);

    // 4. 저장해뒀던 이전 상태로 GC 상태를 복원합니다.
    jlrs_gc_unsafe_leave(ptls, old_state);

    // 5. 결과 반환
    status
}