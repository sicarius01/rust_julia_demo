


using Libdl
using Base.Threads

# 동적 라이브러리 로드
root_dir = "C:\\Users\\haeso\\Documents\\project\\rust_julia_demo"
const LIB_PATH = "$(root_dir)\\target\\release\\rust_julia_demo.dll"

# 라이브러리 로드
lib = dlopen(LIB_PATH)


# Rust 함수들의 포인터 가져오기
const set_callback_ptr = dlsym(lib, :set_callback)
const run_callback_with_jl_sys_ptr = dlsym(lib, :run_callback_with_jl_sys)

function my_callback(ptr::Ptr{UInt8})::Cint
    data = unsafe_string(ptr)
    println("[Julia] Received: ", data)
    return 0 # 성공
end

# 3. 콜백 함수를 C 포인터로 변환
const C_CALLBACK = @cfunction(my_callback, Cint, (Ptr{UInt8},))

const DATA = b"Hello from Julia!"

println("Calling Rust function...")

# 5. ccall로 Rust 함수 호출
status = ccall(
    (:run_callback_with_jl_sys, LIB_PATH), # 호출할 함수
    Cint,                                  # 반환 타입
    (Ptr{Cvoid}, Ptr{UInt8}),              # 인자 타입들
    C_CALLBACK,                            # 콜백 포인터
    pointer(b"hello")                          # 데이터 포인터
)
println("...Rust function finished with status: ", status)

status2 = @ccall $run_callback_with_jl_sys_ptr(C_CALLBACK::Ptr{Cvoid}, pointer(DATA)::Ptr{UInt8})::Cint

println("...Rust function finished with status2: ", status2)












