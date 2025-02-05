import { invoke } from '@tauri-apps/api/core'
/**
 * 黑客工具
 */

export type V2 = {
  x: number
  y: number
}

export type V3 = {
  x: number
  y: number
  z: number
}

export type WindowInfo = {
  x: number
  y: number
  width: number
  height: number
  err: boolean
}

export type MemoryReadResult = {
  address: number
  bytes: any[]
  value: any[]
}

/**
 * 查找窗口句柄
 */
export const findWindow = async (name: string): Promise<number> => {
  return await invoke('find_window_w', { name: name })
}

/**
 * 获取窗口信息
 */
export const getWindowInfo = async (handle: number): Promise<WindowInfo> => {
  return JSON.parse(await invoke('get_window_info', { handle: handle }))
}

/**
 * 通过进程名获取PID
 */
export const getProcessIDByName = async (name: string): Promise<number> => {
  return await invoke('get_process_id_by_name', { name: name })
}

/**
 * 通过pid获取进程句柄
 */
export const getProcessHandle = async (pid: number): Promise<number> => {
  return await invoke('get_process_handle', { pid: pid })
}

/**
 * 获取基础模块地址
 */
export const getModuleBaseAddress = async (pid: number, name: string): Promise<number> => {
  return await invoke('get_module_base_address', { pid: pid, name: name })
}

/**
 * 读取进程内存(u32)
 */
export const readProcessMemoryU32 = async (processHandle: number, baseAddress: number, size: number = 4): Promise<MemoryReadResult> => {
  const data = await readProcessMemory(processHandle, baseAddress, size)
  data.value = byteArrayToU32(data.bytes)

  return data
}

/**
 * 读取进程内存(浮点)
 */
export const readProcessMemoryF32 = async (processHandle: number, baseAddress: number, size: number = 4): Promise<MemoryReadResult> => {
  const data = await readProcessMemory(processHandle, baseAddress, size)
  data.value = byteArrayToF32(data.bytes)
  return data
}

/**
 * 写入进程内存
 */
export const writeProcessMemory = async (handle: number, baseAddress: number, buffer: number): Promise<boolean> => {
  return await invoke('write_memory', { handle: handle, baseAddress: baseAddress, buffer: buffer })
}

/**
 * 世界坐标转窗口坐标
 * @param world_position 世界坐标
 * @param viewMatrix 相机矩阵
 * @param windowInfo 窗口信息
 * @returns 屏幕坐标
 */
export const worldToScreen = async (worldPosition: number[], viewMatrix: number[], windowInfo: WindowInfo): Promise<number[]> => {
  return await invoke('world_to_screen', { worldPosition: worldPosition, viewMatrix: viewMatrix, windowWidth: windowInfo.width, windowHeight: windowInfo.height })
}

/**
 * 获取两点的距离
 * @param worldPosition
 * @param targetPosition
 * @returns
 */
export const calculateSizeBasedOnistance = async (worldPosition: number[], targetPosition: number[]): Promise<number> => {
  return await invoke('calculate_size_based_on_distance', { worldPosition: worldPosition, targetPosition: targetPosition })
}

/**
 * 读取进程内存(u32)
 */
const readProcessMemory = async (processHandle: number, baseAddress: number, size: number = 4): Promise<MemoryReadResult> => {
  return JSON.parse(await invoke('read_memory', { handle: processHandle, baseAddress: baseAddress, size }))
}

//字节转u32
const byteArrayToU32 = (byteArray: any[], isLittleEndian = true) => {
  const data: number[] = []
  const buffer = new ArrayBuffer(4) // 4 bytes for float32
  const view = new DataView(buffer)

  // 每次4字节转成一个32位浮点数
  for (let i = 0; i < byteArray.length; i += 4) {
    // 将字节写入 ArrayBuffer
    view.setUint8(0, byteArray[i])
    view.setUint8(1, byteArray[i + 1])
    view.setUint8(2, byteArray[i + 2])
    view.setUint8(3, byteArray[i + 3])

    // 获取32位浮点数，并根据字节序进行转换
    data.push(view.getUint32(0, isLittleEndian))
  }

  return data
}

// 字节转f32
const byteArrayToF32 = (byteArray: any[], isLittleEndian = true): number[] => {
  const data: number[] = []
  const buffer = new ArrayBuffer(4) // 4 bytes for float32
  const view = new DataView(buffer)

  // 每次4字节转成一个32位浮点数
  for (let i = 0; i < byteArray.length; i += 4) {
    // 将字节写入 ArrayBuffer
    view.setUint8(0, byteArray[i])
    view.setUint8(1, byteArray[i + 1])
    view.setUint8(2, byteArray[i + 2])
    view.setUint8(3, byteArray[i + 3])

    // 获取32位浮点数，并根据字节序进行转换
    data.push(view.getFloat32(0, isLittleEndian))
  }

  return data
}
