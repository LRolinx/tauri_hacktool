mod hack;
use serde::{Deserialize, Serialize};
use std::char::from_u32;
use std::ffi::c_void;
use tauri::Manager;
use windows::Win32::Foundation::{HANDLE, HWND};

use crate::hack::{find_pid_by_name, find_winhwnd_by_pid, read_memory_chain};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

#[derive(Serialize, Deserialize)]
struct WindowInfo {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    err: bool,
}

/**通过进程名来查找进程pid*/
#[tauri::command]
fn findPidByName(name: &str) -> Option<u32> {
    let pid = hack::find_pid_by_name(name);
    pid
}

/**通过进程pid查找主窗口句柄*/
#[tauri::command]
fn findWinhwndByPid(pid: u32) -> Option<u32> {
    let winhwnd = crate::hack::find_winhwnd_by_pid(pid);
    Some(winhwnd.unwrap().0 as u32)
}

/**通过进程pid获取基础模块地址*/
#[tauri::command]
fn findModuleBaseAddressByPid(pid: u32, module_name: &str) -> Option<usize> {
    let value = get_module_base_address(pid, module_name);
    value
}

// 通过窗口句柄获取窗口信息
fn get_window_info(hwnd: Option<HWND>) -> Option<String> {
    // 将 u32 转换为 HWND
    // let hwnd = HWND(handle as *mut c_void);
    // let window_info = hack::get_window_info(Some(hwnd));

    if let Some(window_info) = hack::get_window_info(hwnd) {
        let data = WindowInfo {
            x: window_info.rcClient.left,
            y: window_info.rcClient.top,
            width: window_info.rcClient.right - window_info.rcClient.left,
            height: window_info.rcClient.bottom - window_info.rcClient.top,
            err: false,
        };
        Some(serde_json::to_string(&data).unwrap())
    } else {
        let data = WindowInfo {
            x: -32000,
            y: -32000,
            width: 0,
            height: 0,
            err: true,
        };
        Some(serde_json::to_string(&data).unwrap())
    }
}

// 通过模块名获取基址
#[tauri::command]
fn get_process_id_by_name(name: &str) -> Option<u32> {
    let pid = hack::find_pid_by_name(name);
    pid
}

// 通过进程pid获取进程句柄
#[tauri::command]
fn findProcessHandleByPid(pid: u32) -> Option<u32> {
    // // 将 u32 转换为 HWND
    // let hwnd = HWND(pid as *mut c_void);
    let process_handle = hack::get_process_handle(pid);
    process_handle
}

// 通过模块名获取基址
#[tauri::command]
fn get_module_base_address(pid: u32, name: &str) -> Option<usize> {
    let module_base_addr = hack::get_module_base_address(pid, name)?;
    Some(module_base_addr)
}

// 读内存
#[tauri::command]
fn read_memory(handle: u32, baseAddress: usize, size: Option<usize>) -> Option<String> {
    // 将 u32 转换为 HWND
    let hwnd = HANDLE(handle as *mut c_void);
    let memory_data = hack::read_memory(hwnd, baseAddress, size)?;
    Some(memory_data)
}

// // 读内存(u32)
// #[tauri::command]
// fn read_memory_u32(handle: u32, baseAddress: usize) -> Option<String> {
//     // 将 u32 转换为 HWND
//     let hwnd = HANDLE(handle as *mut c_void);
//     let memory_data = hack::read_memory_u32(hwnd, baseAddress)?;
//     Some(memory_data)
// }

// // 读内存(float)
// #[tauri::command]
// fn read_memory_f32(handle: u32, baseAddress: usize) -> Option<String> {
//     // 将 u32 转换为 HWND
//     let hwnd = HANDLE(handle as *mut c_void);
//     let memory_data = hack::read_memory_f32(hwnd, baseAddress)?;
//     Some(memory_data)
// }

// 写内存
#[tauri::command]
fn write_memory(baseAddr: u32, offset: usize, value: &str) -> Option<bool> {
    // 将 u32 转换为 HWND
    // let hwnd = HANDLE(base_addr as *mut c_void);
    // let memory_data = hack::write_memory(hwnd, offset, value)?;
    // Some(memory_data)
    Some(false)
}

// 世界坐标转屏幕
#[tauri::command]
fn world_to_screen(
    worldPosition: [f32; 3],
    viewMatrix: [f32; 16],
    windowWidth: f32,
    windowHeight: f32,
) -> Option<[f32; 2]> {
    hack::world_to_screen(worldPosition, viewMatrix, windowWidth, windowHeight)
}

// 得到两点之间的距离
#[tauri::command]
fn calculate_size_based_on_distance(
    worldPosition: [f32; 3],
    targetPosition: [f32; 3],
) -> Option<f32> {
    Some(hack::calculate_size_based_on_distance(
        worldPosition,
        targetPosition,
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // 单独起一个线程跑 tokio
            std::thread::spawn(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(start_ws_server());
            });

            let window = app.get_webview_window("main").unwrap();
            #[cfg(windows)]
            {
                use windows::Win32::Foundation::HWND;
                let hwnd = window.hwnd().unwrap().0;
                let hwnd = HWND(hwnd);
                unsafe {
                    let mut style_ex = WINDOW_EX_STYLE(GetWindowLongW(hwnd, GWL_EXSTYLE) as u32);
                    style_ex |= WS_EX_APPWINDOW // for taskbar
                | WS_EX_COMPOSITED
                | WS_EX_LAYERED
                | WS_EX_TRANSPARENT
                | WS_EX_TOPMOST;
                    use windows::Win32::UI::WindowsAndMessaging::*;
                    let nindex = GWL_EXSTYLE;
                    let _pre_val = SetWindowLongA(hwnd, nindex, style_ex.0 as i32);
                }
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            findPidByName,
            findProcessHandleByPid,
            findWinhwndByPid,
            findModuleBaseAddressByPid,
            get_process_id_by_name,
            get_module_base_address,
            read_memory,
            write_memory,
            world_to_screen,
            calculate_size_based_on_distance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// 启动Websocket 服务器
async fn start_ws_server() {
    let addr = "127.0.0.1:9001";
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("WS server started at {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let ws = accept_async(stream).await.unwrap();
            let (mut write, mut read) = ws.split();

            while let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    // println!("Recv: {}", text);
                    let mut parts = text.split_whitespace();

                    // 获取命令
                    if let Some(command) = parts.next() {
                        // 参数
                        let args: Vec<&str> = parts.collect();

                        match command {
                            "getWindowInfoByWinhwnd" => {
                                // 通过进程名获取窗口信息
                                let str_winhwnd: &str = &args[0];
                                let winhwnd: u32 = str_winhwnd.parse().unwrap();
                                let hwnd = HWND(winhwnd as *mut c_void);

                                let info = get_window_info(Some(hwnd));
                                let text_message =
                                    format!("getWindowInfoByWinhwnd {}", info.unwrap());

                                write.send(Message::text(text_message)).await.unwrap();
                            }

                            "getProcessIDByName" => {
                                let name: &str = &args[0];
                                // 查找窗口句柄
                                let value = get_process_id_by_name(name);
                                let text_message = format!("getProcessIDByName {}", value.unwrap());

                                write.send(Message::text(text_message)).await.unwrap();
                            }

                            "readMemoryChain" => {
                                // 读内存带偏移
                                let str_id: &str = &args[0];
                                let str_processHandle: &str = &args[1];
                                let str_baseAddress: &str = &args[2];
                                // 字符串数字转u32
                                let str_offsets = &args[3].split(",").map(|s| s.parse().unwrap()).collect::<Vec<usize>>();
                                let str_size: &str = &args[4];

                                let ph: u32 = str_processHandle.parse().unwrap();
                                let process_handle = HANDLE(ph as *mut c_void);
                                let base_address = str_baseAddress.parse().unwrap();
                                let offsets = str_offsets;
                                let size = str_size.parse().unwrap();

                                let memory_data = read_memory_chain(
                                    process_handle,
                                    base_address,
                                    offsets,
                                    Some(size),
                                );

                                let text_message =
                                    format!("readMemoryChain {str_id} {}", memory_data.unwrap());

                                write.send(Message::text(text_message)).await.unwrap();
                            }

                            _ => println!("无效命令"),
                        }
                    } else {
                        println!("无法识别的格式")
                    }
                }
            }
        });
    }
}
