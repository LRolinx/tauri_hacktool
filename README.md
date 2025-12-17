# Tauri + Vue + TypeScript

![alt text](Snipaste_2025-02-05_21-11-56.png)
使用 Vue 前端和 Rust 后端打造 FPS 透视
前端使用 pixijs 进行绘制
后端封装内存读取工具，前端快速编写代码无需重启调试
Vue 前端与后端实时通讯获取内存数据并在前端渲染

## 命令
```bash
    # 安装依赖
	pnpm i

    # 运行项目
    pnpm tauri dev
```

hack.ts 可用函数 以下函数会通过WebSocket请求 需要监听WebSocket消息返回值

```ts
findWindow(窗口名称) // 查找窗口名返回窗口句柄

getWindowInfo(窗口句柄) // 获取窗口信息

getProcessIDByName(进程名) // 通过进程名获取PID

getProcessHandle(进程PID) // 通过进程PID获取进程句柄

getModuleBaseAddress(进程PID,模块名) // 获取基础模块地址

readProcessMemoryU32(进程句柄,内存地址,大小) // 读取进程内存(u32) 可选参数:大小默认4

readProcessMemoryF32(进程句柄,内存地址,大小) // 读取进程内存(f32) 可选参数:大小默认4

writeProcessMemory(进程句柄,内存地址,buffer数据) // 写内存数据 *未完成不可用

worldToScreen(世界坐标,相机矩阵,窗口信息) // 世界坐标转屏幕坐标

calculateSizeBasedOnistance(原点坐标，目标坐标) // 获取两个坐标的距离
```

作者声明
[仅供学习 请勿非法用途 从事非法用途出现任何问题与作者无关]
![alt text](image.png)
