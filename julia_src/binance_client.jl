# Binance WebSocket Julia 클라이언트
# Rust 동적 라이브러리와 연동

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
const start_websocket_ptr = dlsym(lib, :start_websocket)
const stop_websocket_ptr = dlsym(lib, :stop_websocket)
const is_websocket_running_ptr = dlsym(lib, :is_websocket_running)

# Julia callback 함수 정의
function julia_callback(data_ptr::Ptr{Cchar})
    try
        # C 문자열을 Julia 문자열로 변환
        if data_ptr == C_NULL
            println("[Julia] (null)")
        else
            data = unsafe_string(data_ptr)
            println("[Julia] 받은 데이터: $data")
        end
    catch e
        println("[Julia] 데이터 처리 중 오류: $e")
    end
    nothing
end

# Callback 함수를 C 함수 포인터로 변환
callback_c = @cfunction(julia_callback, Cvoid, (Ptr{Cchar},))

# Rust에 callback 함수 설정
function set_callback()
    ccall(set_callback_ptr, Cvoid, (Ptr{Cvoid},), callback_c)
    println("[Julia] Callback 함수가 Rust에 등록되었습니다.")
end

# WebSocket 연결 시작
function start_websocket(symbol::String = "BTCUSDT")
    # Cstring 타입으로 NUL-종료 문자열을 안전하게 전달
    result = ccall(start_websocket_ptr, Cint, (Cstring,), symbol)
    if result == 0
        println("[Julia] WebSocket 연결 시작: $symbol")
        return true
    else
        println("[Julia] WebSocket 연결 실패")
        return false
    end
end

# WebSocket 연결 중지
function stop_websocket()
    result = ccall(stop_websocket_ptr, Cint, ())
    if result == 0
        println("[Julia] WebSocket 연결 중지")
        return true
    else
        println("[Julia] WebSocket 연결 중지 실패")
        return false
    end
end

# WebSocket 연결 상태 확인
function is_websocket_running()
    result = ccall(is_websocket_running_ptr, Cint, ())
    return result == 1
end

# 메인 실행 함수
function main(is_run::Atomic{Bool})
    println("=== Binance WebSocket Julia 클라이언트 ===")
    
    # Callback 설정
    set_callback()
    
    # WebSocket 연결 시작 (기본값: BTCUSDT)
    symbol = "BTCUSDT"  # 원하는 심볼로 변경 가능 (예: "ETHUSDT", "ADAUSDT")
    
    println("WebSocket 연결을 시작합니다...")
    if start_websocket(symbol)
        println("WebSocket이 성공적으로 시작되었습니다.")
        println("데이터 수신을 위해 대기 중... (Ctrl+C로 종료)")
        
        try
            # 무한 루프로 데이터 수신 대기
            while is_websocket_running()
                sleep(1)  # 1초마다 상태 확인
                if !is_run[]
                    throw("Do Not Run")
                end
            end
        catch InterruptException
            println("\n[Julia] 사용자에 의해 중단되었습니다.")
        finally
            # 정리 작업
            stop_websocket()
            if isinteractive()
                println("[Julia] REPL 모드: dlclose 생략")
            else
                dlclose(lib)
            end
            println("[Julia] 리소스가 정리되었습니다.")
        end
    else
        println("WebSocket 연결에 실패했습니다.")
        if isinteractive()
            println("[Julia] REPL 모드: dlclose 생략")
        else
            dlclose(lib)
        end
    end
end

is_run = Atomic{Bool}(true)






@spawn main(is_run)


sleep(5)
is_run[] = false

sleep(3)
println("END")





