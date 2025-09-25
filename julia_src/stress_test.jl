

begin
    using Libdl
    using Base.Threads

    root_dir = raw"C:\\Users\\haeso\\Documents\\project\\rust_julia_demo"
    const LIB_PATH = "$(root_dir)\\target\\release\\rust_julia_demo.dll"

    lib = dlopen(LIB_PATH)

    const set_callback_ptr = dlsym(lib, :set_callback)
    const run_callback_ptr = dlsym(lib, :run_callback)
    const work_start_ptr = dlsym(lib, :work_start)
    const work_stop_ptr = dlsym(lib, :work_stop)
    const is_work_running_ptr = dlsym(lib, :is_work_running)

    mutable struct Context
        lock::ReentrantLock
        prefix::String
        msg_vec::Vector{String}

        Context(prefix::String) = new(ReentrantLock(), prefix, String[])
    end
    const ctx = Context("{Julia Prefix}")
end

function first_callback(ptr::Ptr{UInt8})::Cint
    try
        data = unsafe_string(ptr)
        println("($(threadid())) [Julia] Received: $(data)")
        return 0
    catch e
        println("Error: $(e)")
        return 1
    end
end

FIRST_CALLBACK = @cfunction(first_callback, Cint, (Ptr{UInt8},))
@ccall $set_callback_ptr(FIRST_CALLBACK::Ptr{Cvoid})::Cvoid


function heavy_function(ptr::Ptr{UInt8})::Cint
    try
        tid = threadid()
        lock(ctx.lock)
        prefix = ctx.prefix
        unlock(ctx.lock)

        data = unsafe_string(ptr)
        println("$(prefix) ($tid) [Julia] Received: $(data)")
        for i in 1:300
            x = rand(Float64, 100, 100)
            if i == 150
                GC.gc()
            end
        end

        lock(ctx.lock)
        ctx.prefix = "{Last ThreadId: [$tid]}"         
        unlock(ctx.lock)       
    catch e
        println("Error: $e")
    finally
        # unlock(ctx.lock)
    end
    return 0
end

NEW_CALLBACK = @cfunction(heavy_function, Cint, (Ptr{UInt8},))
@ccall $set_callback_ptr(NEW_CALLBACK::Ptr{Cvoid})::Cvoid


println("Start Hard Work")
@ccall $work_start_ptr()::Cvoid


sleep(5)
@ccall $work_stop_ptr()::Cvoid
sleep(1)















