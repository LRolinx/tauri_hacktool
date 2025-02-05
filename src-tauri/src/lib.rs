mod hack;
use std::ffi::c_void;

use serde::{Deserialize, Serialize};
use tauri::Manager;
use windows::Win32::Foundation::{HANDLE, HWND};

#[derive(Serialize, Deserialize)]
struct WindowInfo {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    err: bool,
}

// 通过窗口名获取窗口句柄
#[tauri::command]
fn find_window_w(name: &str) -> Option<u32> {
    let window_handle = hack::find_window_w(name)?;
    Some(window_handle)
}

// 通过u32获取窗口信息
#[tauri::command]
fn get_window_info(handle: u32) -> Option<String> {
    // 将 u32 转换为 HWND
    let hwnd = HWND(handle as *mut c_void);
    // let window_info = hack::get_window_info(Some(hwnd));

    if let Some(window_info) = hack::get_window_info(Some((hwnd))) {
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
    let pid = hack::get_process_id(name);
    pid
}

// 通过PID获取句柄
#[tauri::command]
fn get_process_handle(pid: u32) -> Option<u32> {
    // // 将 u32 转换为 HWND
    // let hwnd = HWND(pid as *mut c_void);
    let process_handle = hack::get_process_handle(pid);
    process_handle
}

// 通过模块名获取基址
#[tauri::command]
fn get_module_base_address(pid: u32, name: &str) -> Option<u32> {
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
            find_window_w,
            get_window_info,
            get_process_id_by_name,
            get_module_base_address,
            get_process_handle,
            read_memory,
            // read_memory_f32,
            // read_memory_u32,
            write_memory,
            world_to_screen,
			calculate_size_based_on_distance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
