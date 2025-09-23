


root_dir = "C:\\Users\\haeso\\Documents\\project\\rust_julia_demo"
lib_path = "$(root_dir)\\target\\release\\rust_julia_demo.dll"



println("Using library at: ", abspath(lib_path))

# --- 예제 1: i64 두 개 더하기 ---
println("\n--- Testing `add_i64` function ---")

# ccall((:function_name, "library_path"), ReturnType, (ArgType1, ArgType2), arg1, arg2)
result = ccall((:add_i64, lib_path), Int64, (Int64, Int64), 10, 22)

println("[Julia] 10 + 22 = ", result)
@assert result == 32


# --- 예제 2: 문자열 전달하여 출력하기 ---
println("\n--- Testing `print_with_prefix` function ---")

# Julia의 String을 C 스타일 문자열로 전달하려면 Cstring 타입을 사용합니다.
# 반환 타입이 void인 경우 Cvoid를 사용합니다.
my_name = "Julia User"
ccall((:print_with_prefix, lib_path), Cvoid, (Cstring,), my_name)

println("[Julia] String was sent to Rust.")






