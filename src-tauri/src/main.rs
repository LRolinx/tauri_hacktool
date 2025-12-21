// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use windows::Win32::Foundation::{HANDLE, HWND};
use std::ffi::c_void;
use crate::hack::{find_winhwnd_by_pid, find_pid_by_name, get_module_base_address, get_process_handle, get_window_info, read_memory, read_memory_chain};


mod hack;

fn main() {

   //  let pid = find_pid_by_name("cstrike_win64.exe");
   //  // let winhwnd = find_winhwnd_by_pid(pid.unwrap());
   //  // let info = get_window_info(winhwnd);
   //  //
   //  //
   //  // let serverDll = get_module_base_address(pid.unwrap(), "server.dll");
   //  //
   //  let phwnd = get_process_handle(pid.unwrap());
   //  let hwnd = HANDLE(phwnd.unwrap() as *mut c_void);
   //  // let ss = read_memory(hwnd,serverDll.unwrap()+0x0070A458,Some(4));
   //  // // let ss = read_memory(hwnd,ss.,Some(4));
   //  //
   //  // let new = read_memory_chain(hwnd,serverDll.unwrap(),&[0x0070A458,0x384],Some(4));
   //  // let ssc = read_memory(hwnd,+0x384,Some(4));
   //  //
   //  // serverDll.unwrap() + 0x0070A458
   //
   //  // readMemoryChain demo 1560 140708431659008 7382104,900 4
   // let args = ["demo","1560","140708431659008","7382104,900","4"];
   //
   //  // 读内存带偏移
   //  let str_id: &str = &args[0];
   //  let str_processHandle: &str = &args[1];
   //  let str_baseAddress: &str = &args[2];
   //  let str_offsets: &str = &args[3];
   //  let str_size: &str = &args[4];
   //
   //  let ph: u32 = str_processHandle.parse().unwrap();
   //  let process_handle = HANDLE(ph as *mut c_void);
   //  let base_address = str_baseAddress.parse().unwrap();
   //  let offsets = &[0x0070A458,0x384];
   //  let size = str_size.parse().unwrap();
   //
   //  let me = read_memory(process_handle,base_address,Some(size));
   //
   //  let memory_data = read_memory_chain(
   //      hwnd,
   //      base_address,
   //      offsets,
   //      Some(8),
   //  );


    tauri_hacktool_lib::run();
}
