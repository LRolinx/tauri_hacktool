// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod hack;

fn main() {
    // let windowHanld = hack::find_window_w("BlueStacks");

    // // let windowinfo = hack::get_window_info(windowHanld);

	// if let Some(window_info) = hack::get_window_info(windowHanld) {
	// 	let left = window_info.rcClient.left;
	// 	let top = window_info.rcClient.top;
	// 	println!("left: {}, top: {}", left, top);
	// } else {
	// 	println!("获取窗口信息失败");
	// }

    // print!("{}", windowHanld);
    tauri_hacktool_lib::run();
}
