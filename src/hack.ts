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
 * 通过进程名来查找进程pid
 */
export const findPidByName = async (name: string) => {
	if (name == void 0) return
	return JSON.parse(await invoke('findPidByName', { name }))
}

/**
 * 通过进程pid查找主窗口句柄
 */
export const findWinhwndByPid = async (pid: number) => {
	if (pid == void 0) return
	return JSON.parse(await invoke('findWinhwndByPid', { pid }))
}

/**
 * 获取基础模块地址
 */
export const findModuleBaseAddressByPid = async (pid: number, moduleName: string) => {
	if (pid == void 0 || moduleName == void 0) return
	return JSON.parse(await invoke('findModuleBaseAddressByPid', { pid, moduleName }))
}

/**
 * 通过进程pid获取进程句柄
 */
export const findProcessHandleByPid = async (pid: number) => {
	if (pid == void 0) return
	return JSON.parse(await invoke('findProcessHandleByPid', { pid: pid }))
}

/**
 * 通过窗口句柄获取窗口信息（通过Socket返回值）
 */
export const getWindowInfoByWinhwnd = (handle?: number) => {
	if (handle == void 0) return
	window.socket!.send(`getWindowInfoByWinhwnd ${handle}`)
	//   return JSON.parse(await invoke('get_window_info', { handle: handle }))
}



/**
 * 读取内存+多层偏移（通过Socket返回值）
 */
export const readMemoryChain = (id: string, processHandle: number, baseAddress: number, offsets: number[] = [0x0], size: number = 4) => {
	window.socket!.send(`readMemoryChain ${id} ${processHandle} ${baseAddress} ${offsets} ${size}`)
}



/**
 * 世界坐标转窗口坐标
 * @param world_position 世界坐标
 * @param viewMatrix 相机矩阵
 * @param windowInfo 窗口信息
 * @returns 屏幕坐标
 */
export const worldToScreen = async (worldPosition: number[], viewMatrix: number[], windowInfo: WindowInfo): Promise<number[] | undefined> => {
	// return await invoke('world_to_screen', { worldPosition: worldPosition, viewMatrix: viewMatrix, windowWidth: windowInfo.width, windowHeight: windowInfo.height })
	try {
		// 裁剪坐标
		let clipCoordsX = worldPosition[0] * viewMatrix[0] + worldPosition[1] * viewMatrix[1] + worldPosition[2] * viewMatrix[2] + viewMatrix[3]
		let clipCoordsY = worldPosition[0] * viewMatrix[4] + worldPosition[1] * viewMatrix[5] + worldPosition[2] * viewMatrix[6] + viewMatrix[7]
		// let clipCoordsZ = worldPosition[0] * viewMatrix[8]
		// 	+ worldPosition[1] * viewMatrix[9]
		// 	+ worldPosition[2] * viewMatrix[10]
		// 	+ viewMatrix[11];
		let clipCoordsW = worldPosition[0] * viewMatrix[12] + worldPosition[1] * viewMatrix[13] + worldPosition[2] * viewMatrix[14] + viewMatrix[15]

		// 如果 clipCoordsW <= 0，物体在视图的背面，不显示
		if (clipCoordsW <= 0) {
			return
		}

		// 转换为DNC坐标
		let ndcX = clipCoordsX / clipCoordsW
		let ndcY = clipCoordsY / clipCoordsW
		// let ndcZ = clipCoordsZ / clipCoordsW;

		// 转换成屏幕坐标
		let screenX = (windowInfo.width / 2.0) * ndcX + (ndcX + windowInfo.width / 2.0)
		let screenY = -((windowInfo.height / 2.0) * ndcY) + (ndcY + windowInfo.height / 2.0)


		return [screenX, screenY]
	} catch {
		return
	}
}

/**内存偏移工具 offset使用数字为10进制 使用0x00为16进制
 */
export const byteOffset = (byteArray: any[], offset: number, byteNum: number) => {
	const offsetByte = []
	let currentIndex = 0
	for (let i = offset; i < byteArray.length; i++) {
		offsetByte.push(byteArray[i])
		currentIndex++
		if (currentIndex >= byteNum) break
	}
	return offsetByte
}

/**字节转整数 */
export const byteArrayToU32 = (byteArray: any[], isLittleEndian = true) => {
	const data: number[] = []
	const buffer = new ArrayBuffer(4) // 4 bytes for float32
	const view = new DataView(buffer)

	// 每次4字节转成一个32位整数
	for (let i = 0; i < byteArray.length; i += 4) {
		// 将字节写入 ArrayBuffer
		view.setUint8(0, byteArray[i])
		view.setUint8(1, byteArray[i + 1])
		view.setUint8(2, byteArray[i + 2])
		view.setUint8(3, byteArray[i + 3])

		// 获取32位整数，并根据字节序进行转换
		data.push(view.getUint32(0, isLittleEndian))
	}

	return data
}

/**字节转浮点数 */
export const byteArrayToF32 = (byteArray: any[], isLittleEndian = true): number[] => {
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


// /**
//  * 获取两点的距离
//  * @param worldPosition
//  * @param targetPosition
//  * @returns
//  */
// export const calculateSizeBasedOnistance = async (worldPosition: number[], targetPosition: number[]): Promise<number> => {
// 	return await invoke('calculate_size_based_on_distance', { worldPosition: worldPosition, targetPosition: targetPosition })
// }

// /**
//  * 读取进程内存(u32)
//  */
// const readProcessMemory = async (processHandle: number, baseAddress: number, size: number = 4): Promise<MemoryReadResult> => {
// 	return JSON.parse(await invoke('read_memory', { handle: processHandle, baseAddress: baseAddress, size }))
// }

// // 根据x大小返回限制范围
// export const reverseLinearNormalize = (x: number, dataMin: number, dataMax: number, min: number, max: number) => {
// 	// 计算归一化值 (这里是反向映射的)
// 	const normalized = (x - dataMin) / (dataMax - dataMin)

// 	// 将归一化值映射到 [max, min] 范围
// 	const mapped = normalized * (max - min) + min

// 	// 返回反转后的结果：x越小，返回值越接近max
// 	let data = max - mapped

// 	return data <= min ? min : data >= max ? max : data
// }

// /**
//  * 读取进程内存(u32)
//  */
// export const readProcessMemoryU32 = (processHandle: number, baseAddress: number, size: number = 4) => {
// 	window.socket!.send(`readProcessMemoryU32 ${processHandle} ${baseAddress} ${size}`)
// 	// const data = await readProcessMemory(processHandle, baseAddress, size)
// 	// data.value = byteArrayToU32(data.bytes)

// 	// return data
// }

// /**
//  * 读取进程内存(浮点)
//  */
// export const readProcessMemoryF32 = (processHandle: number, baseAddress: number, size: number = 4) => {
// 	window.socket!.send(`readProcessMemoryU32 ${processHandle} ${baseAddress} ${size}`)
// 	// const data = await readProcessMemory(processHandle, baseAddress, size)
// 	// data.value = byteArrayToF32(data.bytes)
// 	// return data
// }

// /**
//  * 写入进程内存
//  */
// export const writeProcessMemory = (handle: number, baseAddress: number, buffer: number) => {
// 	window.socket!.send(`writeProcessMemory ${handle} ${baseAddress} ${buffer}`)
// 	// return await invoke('write_memory', { handle: handle, baseAddress: baseAddress, buffer: buffer })
// }

// /**
//  * 通过进程名获取PID
//  */
// export const getProcessIDByName = (name: string) => {
// 	window.socket!.send(`getProcessIDByName ${name}`)
// 	//   return await invoke('get_process_id_by_name', { name: name })
// }
