
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::thread;
use std::time::Duration;
use std::sync::{Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};

use futures_util::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

// Julia callback 함수 타입 정의
type JuliaCallback = extern "C" fn(*const c_char);

// 전역 callback 함수 포인터
static CALLBACK: OnceLock<JuliaCallback> = OnceLock::new();
static WEBSOCKET_RUNNING: AtomicBool = AtomicBool::new(false);
static THREAD_HANDLE: Mutex<Option<thread::JoinHandle<()>>> = Mutex::new(None);

// Binance 응답 데이터 구조체
#[derive(Serialize, Deserialize, Debug)]
struct BinanceTickerData {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "E")]
    event_time: u64,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "c")]
    close_price: String,
    #[serde(rename = "o")]
    open_price: String,
    #[serde(rename = "h")]
    high_price: String,
    #[serde(rename = "l")]
    low_price: String,
    #[serde(rename = "v")]
    volume: String,
    #[serde(rename = "q")]
    quote_volume: String,
}

// Julia에서 호출할 callback 함수를 설정하는 함수
#[no_mangle]
pub extern "C" fn set_callback(callback: JuliaCallback) {
    let _ = CALLBACK.set(callback);
    println!("Callback function has been set from Julia");
}

// Julia로 데이터를 전송하는 함수
fn send_to_julia(data: &str) {
    if let Some(&callback) = CALLBACK.get() {
        if let Ok(c_string) = CString::new(data) {
            callback(c_string.as_ptr());
        }
    }
}

// WebSocket 연결을 시작하는 함수
#[no_mangle]
pub extern "C" fn start_websocket(symbol: *const c_char) -> i32 {
    if WEBSOCKET_RUNNING.swap(true, Ordering::SeqCst) {
        return -1; // 이미 실행 중
    }

    let symbol_str = unsafe {
        CStr::from_ptr(symbol).to_str().unwrap_or("BTCUSDT")
    };

    // 새로운 스레드에서 WebSocket 연결 실행
    let symbol_owned = symbol_str.to_lowercase();
    let handle = thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        rt.block_on(async {
            // if let Err(e) = run_websocket(&symbol_owned).await {
            //     let error_msg = format!("WebSocket error: {}", e);
            //     println!("{}", error_msg);
            //     send_to_julia(&error_msg);
            // }
            // else {
            //     println!("run_websocket 성공");
            // }
            
            match run_websocket(&symbol_owned).await {
                Ok(_) => {
                    println!("run_websocket 성공");
                }
                Err(e) => {
                    let error_msg = format!("WebSocket error: {}", e);
                    println!("{}", error_msg);
                    send_to_julia(&error_msg);
                }
            }
            println!("run_websocket 끝");
        });
        println!("rt 런타임 끝");
    });
    *THREAD_HANDLE.lock().unwrap() = Some(handle);

    0 // 성공
}

// WebSocket 연결을 중지하는 함수
#[no_mangle]
pub extern "C" fn stop_websocket() -> i32 {
    println!("stop_websocket 시작");
    WEBSOCKET_RUNNING.store(false, Ordering::SeqCst);
    println!("stop_websocket - store 함");
    if let Some(handle) = THREAD_HANDLE.lock().unwrap().take() {
        println!("stop_websocket - THREAD_HANDLE 락 획득");
        let _ = handle.join().unwrap();
        println!("stop_websocket - join 완료");
    }
    println!("WebSocket connection stopped");
    0 // 성공
}

// WebSocket 실행 함수
async fn run_websocket(symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Binance 선물 WebSocket URL
    let url = format!("wss://fstream.binance.com/ws/{}@ticker", symbol);
    let url = Url::parse(&url)?;

    println!("Connecting to Binance WebSocket: {}", url);
    send_to_julia(&format!("Connecting to Binance WebSocket for symbol: {}", symbol));

    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    println!("WebSocket connected successfully");
    send_to_julia("WebSocket connected successfully");

    while WEBSOCKET_RUNNING.load(Ordering::Relaxed) {
        if let Ok(next_item) = tokio::time::timeout(Duration::from_millis(200), read.next()).await {
            if let Some(msg) = next_item {
            match msg {
                Ok(Message::Text(text)) => {
                    // JSON 데이터 파싱
                    match serde_json::from_str::<BinanceTickerData>(&text) {
                        Ok(ticker_data) => {
                            let formatted_data = format!(
                                "Symbol: {}, Price: {}, Volume: {}, Time: {}",
                                ticker_data.symbol,
                                ticker_data.close_price,
                                ticker_data.volume,
                                ticker_data.event_time
                            );
                            println!("Received: {}", formatted_data);
                            send_to_julia(&formatted_data);
                        }
                        Err(e) => {
                            println!("Failed to parse JSON: {}", e);
                            send_to_julia(&format!("JSON parse error: {}", e));
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("WebSocket connection closed by server");
                    send_to_julia("WebSocket connection closed by server");
                    break;
                }
                Err(e) => {
                    println!("WebSocket error: {}", e);
                    send_to_julia(&format!("WebSocket error: {}", e));
                    break;
                }
                _ => {}
            }
            }
        }
        // 타임아웃이면 루프 재확인 (즉시 종료 반영)
    }
    println!("웹소캣 while문 끝");

    // 종료 직전 Close 프레임 전송 시도 (best-effort)
    let _ = write.send(Message::Close(None)).await;
    let _ = write.flush().await;
    WEBSOCKET_RUNNING.store(false, Ordering::SeqCst);
    
    println!("WebSocket connection terminated");
    // send_to_julia("WebSocket connection terminated");
    Ok(())
}

// 연결 상태를 확인하는 함수
#[no_mangle]
pub extern "C" fn is_websocket_running() -> i32 {
    if WEBSOCKET_RUNNING.load(Ordering::Relaxed) { 1 } else { 0 }
}

// #[unsafe(no_mangle)]
// pub extern "C" fn add_i64(a: i64, b: i64) -> i64 {
//     a + b
// }

// #[unsafe(no_mangle)]
// pub extern "C" fn print_with_prefix(s: *const c_char) {
//     // 안전: 포인터 유효성은 호출자(Julia)가 보장해야 함
//     if s.is_null() {
//         eprintln!("[rust] (null)");
//         return;
//     }
//     let cstr = unsafe { CStr::from_ptr(s) };
//     match cstr.to_str() {
//         Ok(txt) => {
//             // 실제 환경에선 println! 대신, 로그/큐 적재 등으로 바꿔도 됨
//             println!("[rust] prefix:: {}", txt);
//         }
//         Err(_) => eprintln!("[rust] invalid utf-8"),
//     }
// }
