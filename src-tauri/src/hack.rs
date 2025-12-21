use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::{CloseHandle, LPARAM};
use windows::Win32::System::Diagnostics::ToolHelp::Process32NextW;
use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use sysinfo::System;
use widestring::U16CString;
use windows::{
    Win32::{
        Foundation::{HANDLE, HMODULE, HWND, MAX_PATH},
        System::{
            Diagnostics::{
                Debug::{ReadProcessMemory, WriteProcessMemory},
                ToolHelp::{
                    CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, PROCESSENTRY32W, Process32FirstW, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS
                },
            },
            Threading::{
                OpenProcess, PROCESS_ALL_ACCESS, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
            },
        },
        UI::WindowsAndMessaging::{FindWindowW, GetWindowInfo, WINDOWINFO},
    }, core::PCWSTR
};
use windows::core::BOOL;
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindow, GetWindowThreadProcessId, IsWindowVisible, GW_OWNER};



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





/// 通过进程名来查找进程pid
pub fn find_pid_by_name(name: &str) -> Option<u32> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let exe = OsString::from_wide(
                    &entry.szExeFile[..entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(0)],
                )
                .to_string_lossy()
                .to_string();

                if exe.eq_ignore_ascii_case(name) {
                    CloseHandle(snapshot);
                    return Some(entry.th32ProcessID);
                }

                if !Process32NextW(snapshot, &mut entry).is_ok() {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
        None
    }
}

// /// 通过进程名获取PID 这个太慢了不建议使用
// pub fn find_pid_by_name(process_name: &str) -> Option<u32> {
//     let mut system = System::new_all();
//     system.refresh_all();
//
//     for (pid, process) in system.processes() {
//         if process.name().to_ascii_lowercase().to_str()? == process_name.to_lowercase() {
//             return Some(pid.as_u32());
//         }
//     }
//     None
// }


/// 通过进程pid获取进程句柄
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



/// 通过进程pid查找主窗口句柄
pub fn find_winhwnd_by_pid(pid: u32) -> Option<HWND> {
    find_all_hwnds_by_pid(pid).into_iter().next()
}

struct EnumContext {
    target_pid: u32,
    result: Vec<HWND>,
}

/// 通过进程pid查找全部窗口
pub fn find_all_hwnds_by_pid(pid: u32) -> Vec<HWND> {
    let mut ctx = EnumContext {
        target_pid: pid,
        result: Vec::new(),
    };

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let ctx = &mut *(lparam.0 as *mut EnumContext);

        let mut window_pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut window_pid));

        if window_pid == ctx.target_pid
            && IsWindowVisible(hwnd).as_bool()
            // && GetWindow(hwnd, GW_OWNER).0 == 0
        {
            ctx.result.push(hwnd);
        }

        BOOL(1)
    }

    unsafe {
        EnumWindows(
            Some(enum_proc),
            LPARAM(&mut ctx as *mut _ as isize),
        );
    }

    ctx.result
}

/// 获取指定进程中 `demo.exe` 的基地址
/// 修复64位地址被截断成32位地址，导致64位程序获取不了正确的地址
pub fn get_module_base_address(pid: u32, module_name: &str) -> Option<usize> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(
            TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32,
            pid,
        ).ok()?;

        let mut entry = MODULEENTRY32W {
            dwSize: std::mem::size_of::<MODULEENTRY32W>() as u32,
            ..Default::default()
        };

        if Module32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let name = OsString::from_wide(
                    &entry.szModule[..entry.szModule.iter().position(|&c| c == 0).unwrap_or(0)]
                )
                    .to_string_lossy()
                    .to_string();

                if name.eq_ignore_ascii_case(module_name) {
                    CloseHandle(snapshot);
                    return Some(entry.modBaseAddr as usize);
                }

                if !Module32NextW(snapshot, &mut entry).is_ok() {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
        None
    }
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

// 读取内存+多层偏移
pub fn read_memory_chain(
    process_handle: HANDLE,
    base_address: usize,
    offsets: &[usize],
    size: Option<usize>,
) -> Option<String> {
    unsafe {
        if offsets.is_empty() {
            return None;
        }

        let size = size.unwrap_or(4);
        let mut addr = base_address;

        // 前 n-1 层：指针解引用
        for &offset in &offsets[..offsets.len() - 1] {
            let mut ptr: usize = 0;
            let mut read = 0;

            let ok = ReadProcessMemory(
                process_handle,
                (addr + offset) as *const _,
                &mut ptr as *mut _ as *mut _,
                std::mem::size_of::<usize>(),
                Some(&mut read),
            );

            if !ok.is_ok() || read != std::mem::size_of::<usize>() || ptr == 0 {
                return None;
            }

            addr = ptr;
        }

        // 最后一层：直接偏移
        let final_address = addr + offsets[offsets.len() - 1];

        // 读取最终数据
        let mut buffer = vec![0u8; size];
        let mut bytes_read = 0;

        let ok = ReadProcessMemory(
            process_handle,
            final_address as *const _,
            buffer.as_mut_ptr() as *mut _,
            size,
            Some(&mut bytes_read),
        );

        if ok.is_ok() && bytes_read == size {
            let data = MemoryReadResult {
                address: final_address,
                bytes: buffer,
                value: Vec::new(),
            };
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
