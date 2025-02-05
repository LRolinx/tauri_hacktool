use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use sysinfo::System;
use widestring::U16CString;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HANDLE, HMODULE, HWND, MAX_PATH},
        System::{
            Diagnostics::{
                Debug::{ReadProcessMemory, WriteProcessMemory},
                ToolHelp::{
                    CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, MODULEENTRY32W,
                    TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32,
                },
            },
            Threading::{
                OpenProcess, PROCESS_ALL_ACCESS, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
            },
        },
        UI::WindowsAndMessaging::{FindWindowW, GetWindowInfo, WINDOWINFO},
    },
};

pub fn find_window_w(title: &str) -> Option<u32> {
    // 将 Rust 字符串转换为 UTF-16 并添加 null 终止符
    let title_wide: Vec<u16> = OsStr::new(title)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // 调用 FindWindowW
    match unsafe { FindWindowW(None, PCWSTR(title_wide.as_ptr())).map(|h| h.0 as u32) } {
        Ok(hwnd) => Some(hwnd), // 成功返回 HWND
        Err(_) => None,         // 失败返回 None
    }
}

pub fn get_window_info(window_handle: Option<HWND>) -> Option<WINDOWINFO> {
    let mut window_info = WINDOWINFO {
        cbSize: std::mem::size_of::<WINDOWINFO>() as u32, // 重要: 初始化 cbSize
        ..Default::default()
    };

    if unsafe { GetWindowInfo(window_handle?, &mut window_info) }.is_ok() {
        Some(window_info) // 成功返回结构体
    } else {
        None // 失败返回 None
    }
}

// pub fn get_module_handle_a(title: &str) -> Option<u32> {
//     // 将 Rust 字符串转换为 UTF-16 并添加 null 终止符
//     let title_wide: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
//     // 调用 FindWindowW
//     match unsafe { GetModuleHandleW( PCWSTR(title_wide.as_ptr())).map(|h| h.0 as u32) } {
//         Ok(hwnd) => Some(hwnd), // 成功返回 HWND
//         Err(_) => None,         // 失败返回 None
//     }
// }

/// 通过进程名获取PID
pub fn get_process_id(process_name: &str) -> Option<u32> {
    let mut system = System::new_all();
    system.refresh_all();

    for (pid, process) in system.processes() {
        if process.name().to_ascii_lowercase().to_str()? == process_name.to_lowercase() {
            return Some(pid.as_u32());
        }
    }
    None
}

/// 通过PID获取句柄
pub fn get_process_handle(pid: u32) -> Option<u32> {
    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid).map(|h| h.0 as u32);
        if handle.is_ok() {
            Some(handle.unwrap())
        } else {
            None
        }
    }
}

/// 获取指定进程中 `demo.exe` 的基地址
pub fn get_module_base_address(pid: u32, module_name: &str) -> Option<u32> {
    unsafe {
        let snapshot: HANDLE =
            CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid).ok()?;
        let mut module_entry = MODULEENTRY32W {
            dwSize: std::mem::size_of::<MODULEENTRY32W>() as u32,
            ..Default::default()
        };

        if Module32FirstW(snapshot, &mut module_entry).is_ok() {
            let target_module = U16CString::from_str(module_name).ok()?;
            loop {
                if module_entry.szModule.starts_with(target_module.as_slice()) {
                    return Some(module_entry.modBaseAddr as u32);
                }
                if !Module32NextW(snapshot, &mut module_entry).is_ok() {
                    break;
                }
            }
        }
    }
    None
}

#[derive(Serialize)]
struct MemoryReadResult {
    address: usize,
    bytes: Vec<u8>, // 转为字符串
    value: Vec<String>,
}

pub fn read_memory(process_handle: HANDLE, address: usize, size: Option<usize>) -> Option<String> {
    unsafe {
        // 默认读取 4 字节
        let size = size.unwrap_or(4);

        // 创建一个大小为 size 的 buffer，动态分配内存
        let mut buffer: Vec<u8> = vec![0; size];

        let mut bytes_read = 0;
        let result = ReadProcessMemory(
            process_handle,
            address as *const _,
            buffer.as_mut_ptr() as *mut _,
            size,
            Some(&mut bytes_read),
        );

        if result.is_ok() && bytes_read == size {
            // 如果读取了预期的字节数，返回字节数据
            let data = MemoryReadResult {
                address,
                bytes: buffer, // 直接存储字节数据
                value: [].to_vec(),
            };

            // 将结果序列化为 JSON 字符串
            Some(serde_json::to_string(&data).unwrap())
        } else {
            None
        }
    }
}

// pub fn read_memory_u32(process_handle: HANDLE, address: usize, size: Option<usize>) -> Option<String> {
//     unsafe {
//         // 默认读取 4 字节
//         let size = size.unwrap_or(4);

//         // 创建一个大小为 size 的 buffer，动态分配内存
//         let mut buffer: Vec<u8> = vec![0; size];

//         let mut bytes_read = 0;
//         let result = ReadProcessMemory(
//             process_handle,
//             address as *const _,
//             buffer.as_mut_ptr() as *mut _,
//             size,
//             Some(&mut bytes_read),
//         );

//         if result.is_ok() && bytes_read == size {
//             // 如果读取了预期的字节数，进行处理
//             let data = if size == 4 {
//                 // 如果只读取了 4 字节，解释为一个 u32
//                 let value = u32::from_le_bytes(buffer.try_into().unwrap()); // 处理为 u32
//                 MemoryReadResult { address, value }
//             } else {
//                 // 如果读取了其他字节，假设是字节数据（可以根据需求进一步解析）
//                 let value = buffer.iter().map(|&byte| byte as u32).collect::<Vec<u32>>();
//                 MemoryReadResult { address, value }
//             };

//             Some(serde_json::to_string(&data).unwrap())
//         } else {
//             None
//         }
//     }
// }

// pub fn read_memory_f32(process_handle: HANDLE, address: usize, size: Option<usize>) -> Option<String> {
//     unsafe {
//         // 默认读取 4 字节
//         let size = size.unwrap_or(4);

//         // 创建一个大小为 size 的 buffer，动态分配内存
//         let mut buffer: Vec<u8> = vec![0; size];

//         let mut bytes_read = 0;
//         let result = ReadProcessMemory(
//             process_handle,
//             address as *const _,
//             buffer.as_mut_ptr() as *mut _,
//             size,
//             Some(&mut bytes_read),
//         );

//         if result.is_ok() && bytes_read == size {
//             // 如果读取了预期的字节数，进行处理
//             let data = if size == 4 {
//                 // 如果只读取了 4 字节，解释为一个 f32
//                 let value = f32::from_le_bytes(buffer.try_into().unwrap()); // 处理为 f32
//                 MemoryReadResult { address, value }
//             } else {
//                 // 如果读取了其他字节，假设是字节数据（可以根据需求进一步解析）
//                 let value = buffer.iter().map(|&byte| byte as f32).collect::<Vec<f32>>();
//                 MemoryReadResult { address, value }
//             };

//             Some(serde_json::to_string(&data).unwrap())
//         } else {
//             None
//         }
//     }
// }

pub fn write_memory<T>(process_handle: HANDLE, address: usize, value: &T) -> Option<bool> {
    // // Some(util::read_memory::<[f32; 16]>(base_addr, offset));
    // let data_ptr = (base_addr + offset) as *const u32;
    // Some(unsafe { *data_ptr })

    unsafe {
        let mut bytes_written = 0;
        let result = WriteProcessMemory(
            process_handle,
            address as *mut _,
            value as *const _ as *const _,
            std::mem::size_of::<T>(),
            Some(&mut bytes_written),
        );

        if result.is_ok() && bytes_written == std::mem::size_of::<T>() {
            Some(true)
        } else {
            Some(false)
        }
    }
}

pub fn world_to_screen(
    worldPosition: [f32; 3],
    viewMatrix: [f32; 16],
    windowWidth: f32,
    windowHeight: f32,
) -> Option<[f32; 2]> {
    // 裁剪坐标
    let clipCoordsX = worldPosition[0] * viewMatrix[0]
        + worldPosition[1] * viewMatrix[1]
        + worldPosition[2] * viewMatrix[2]
        + viewMatrix[3];
    let clipCoordsY = worldPosition[0] * viewMatrix[4]
        + worldPosition[1] * viewMatrix[5]
        + worldPosition[2] * viewMatrix[6]
        + viewMatrix[7];
    let clipCoordsZ = worldPosition[0] * viewMatrix[8]
        + worldPosition[1] * viewMatrix[9]
        + worldPosition[2] * viewMatrix[10]
        + viewMatrix[11];
    let clipCoordsW = worldPosition[0] * viewMatrix[12]
        + worldPosition[1] * viewMatrix[13]
        + worldPosition[2] * viewMatrix[14]
        + viewMatrix[15];

    // 如果 clipCoordsW <= 0，物体在视图的背面，不显示
    if clipCoordsW <= 0.0 {
        return None;
    }

    // 转换为DNC坐标
    let ndcX = clipCoordsX / clipCoordsW;
    let ndcY = clipCoordsY / clipCoordsW;
    let ndcZ = clipCoordsZ / clipCoordsW;

    // 转换成屏幕坐标
    let screenX = (windowWidth / 2.0 * ndcX) + (ndcX + windowWidth / 2.0);
    let screenY = -(windowHeight / 2.0 * ndcY) + (ndcY + windowHeight / 2.0);


    Some([screenX, screenY])
}

// 计算物体到摄像机的世界空间距离，并返回一个与该距离反比的大小值（距离越近，size越大）
pub fn calculate_size_based_on_distance(worldPosition: [f32; 3], targetPosition: [f32; 3]) -> f32 {

    // 计算物体与摄像机之间的距离（欧几里得距离）
    let dx = worldPosition[0] - targetPosition[0];
    let dy = worldPosition[1] - targetPosition[1];
    let dz = worldPosition[2] - targetPosition[2];

    let distance = (dx.powi(2) + dy.powi(2) + dz.powi(2)).sqrt();

    // 返回与距离反比的大小值（距离越近，物体越大）
    let max_distance = 1000.0; // 最大距离，用于控制物体大小的衰减
    let min_size = 0.1; // 最小物体大小
    let max_size = 1.0; // 最大物体大小

    // 距离越近，物体越大
    let size = 1.0 / (1.0 + distance / max_distance); // 距离越小，size越大，反之越小

    // 确保物体不会变得太小
    // size.max(min_size);

	distance
}
