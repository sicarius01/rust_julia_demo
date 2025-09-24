

using Libdl
using Base.Threads

# 동적 라이브러리 로드
root_dir = "C:\\Users\\haeso\\Documents\\project\\rust_julia_demo"
const LIB_PATH = "$(root_dir)\\target\\release\\rust_julia_demo.dll"

# 라이브러리가 존재하는지 확인
if !isfile(LIB_PATH)
    error("라이브러리 파일을 찾을 수 없습니다: $LIB_PATH")
end

# 라이브러리 로드
lib = dlopen(LIB_PATH)

println("Rust 라이브러리가 성공적으로 로드되었습니다.")

# Rust 함수들의 포인터 가져오기
const set_callback_ptr = dlsym(lib, :set_callback)
const start_work_ptr = dlsym(lib, :start_work)
const stop_work_ptr = dlsym(lib, :stop_work)
const is_work_running_ptr = dlsym(lib, :is_work_running)


function julia_callback(data_ptr::Ptr{UInt8})
    # lock(ctx.lock)
    # suffix = ctx.suffix
    # unlock(ctx.lock)
    try
        len_u8 = unsafe_load(data_ptr)::UInt8
        len = Int(len_u8)
        body = data_ptr + 1

        bytes = unsafe_wrap(Vector{UInt8}, body, len; own=false)
        io = IOBuffer(bytes; read=true, write=false, truncate=false)

        dest = Vector{UInt8}(undef, len)
        read!(io, dest)
        @show dest String(dest)
    catch e
        println("[Julia] Error: $(e)")
    end
    nothing
end


callback_c = @cfunction(julia_callback, Cvoid, (Ptr{UInt8},))

function set_callback()
    @ccall $set_callback_ptr(callback_c::Ptr{Cvoid})::Cvoid
    println("[Julia] Callback set")
end

function start_work(sb::String)
    result = @ccall $start_work_ptr(sb::Cstring)::Cint
    result == 0 ? println("[Julia] Work Start: $(sb)") : println("[Julia] Work Start Fail: $(sb)")
end

function stop_work()
    result = @ccall $stop_work_ptr()::Cint
    if result == 0
        println("[Julia] Work Stop")
        return true
    else
        println("[Julia] Work Stop Fail")
        return false
    end
end

function is_work_running()
    result = @ccall $is_work_running_ptr()::Cint
    return result == 1
end


# is_run = Atomic{Bool}(true)


set_callback()
start_work("MBSB")



sleep(5)
# is_run[] = false
stop_work()
sleep(3)


# @spawn begin
#     start_work("MBSB")
#     while is_run[] && is_work_running()
#         sleep(1)
#     end
# end























