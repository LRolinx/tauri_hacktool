<script setup lang="ts">
import { onMounted, ref } from 'vue'
import {
	WindowInfo,
	findPidByName,
	findWinhwndByPid,
	getWindowInfoByWinhwnd,
	findModuleBaseAddressByPid,
	findProcessHandleByPid,
	readMemoryChain,
	byteArrayToF32,
	byteArrayToU32,
	worldToScreen,
	byteOffset,
} from './hack'
import { Application, Graphics, Text } from 'pixi.js'
import { SocketClient } from './Socket_Client'

let pid = 0
let phandle = 0
let winhwnd = 0
let serverDll = 0
let engineDll = 0

const socket = (window.socket = new SocketClient())

const windowInfo = ref<WindowInfo>({
	x: 0,
	y: 0,
	width: 0,
	height: 0,
	err: false,
})

const gameInfo = ref({
	playerPos: [0, 0, 0],
	homeNum: 0,
	homeNotDieNum: 0,
	cameraMatrix: Array(64),
	targetInfo: [] as { boloom: number; pos: number[] }[],
})

const pixiRef = ref()
const app = new Application()
const windowGraphics = new Graphics({ x: -999, y: -999 })
const windowInfoText = new Text()
// 建立16个透视框
const targetGraphiceList = new Array(16).fill(new Graphics({ x: 0, y: 0, visible: true }).rect(0, 0, 50, 100).stroke({ width: 2, color: '#375fad' }))

// 订阅窗口信息
const subWindowInfo = () => getWindowInfoByWinhwnd(winhwnd)
// 订阅房间人数
const subHomeNum = () => readMemoryChain('subHomeNum', phandle, engineDll, [0x6da960])
// 订阅存活人数
const subHomeNotDieNum = () => readMemoryChain('subHomeNotDieNum', phandle, serverDll, [0x703d0c])
// 订阅玩家信息
const subPlayInfo = () => readMemoryChain('subPlayInfo', phandle, serverDll, [0x0070a458, 0x384], 12)
// 订阅矩阵信息
const subMatrix = () => readMemoryChain('subMatrix', phandle, engineDll, [0x00698f18, 0x2d4], 64)
// 订阅目标位置
const subTarge = () => readMemoryChain('subTarge', phandle, serverDll, [0x0070a478, 0x164], 0x220 + 12)

socket!.onOpen = async () => {
	// Socket打开后开始订阅

	pid = await findPidByName('cstrike_win64.exe')
	phandle = await findProcessHandleByPid(pid)
	winhwnd = await findWinhwndByPid(pid)
	serverDll = await findModuleBaseAddressByPid(pid, 'server.dll')
	engineDll = await findModuleBaseAddressByPid(pid, 'engine.dll')

	subWindowInfo()
	subPlayInfo()
	subMatrix()
	subHomeNum()
	subHomeNotDieNum()
	subTarge()
}

socket!.onMessage = ({ data }: { data: string }) => {
	const res = data.split(' ')
	if (res[0] == 'getWindowInfoByWinhwnd') {
		// 拿到窗口信息
		const json = JSON.parse(res[1])
		windowInfo.value = json

		subWindowInfo()
	}

	if (res[0] == 'readMemoryChain') {
		// 拿到读取内存的信息
		if (res[1] == 'subPlayInfo') {
			// 订阅的玩家信息
			const json = JSON.parse(res[2])

			gameInfo.value.playerPos = byteArrayToF32(json.bytes)

			subPlayInfo()
		}

		if (res[1] == 'subMatrix') {
			// 订阅的矩阵
			const json = JSON.parse(res[2])

			gameInfo.value.cameraMatrix = byteArrayToF32(json.bytes)

			subMatrix()
		}

		if (res[1] == 'subHomeNum') {
			// 订阅的房间人数
			const json = JSON.parse(res[2])

			gameInfo.value.homeNum = byteArrayToU32(json.bytes)[0]

			subHomeNum()
		}

		if (res[1] == 'subHomeNotDieNum') {
			// 订阅的存活人数
			const json = JSON.parse(res[2])

			gameInfo.value.homeNotDieNum = byteArrayToU32(json.bytes)[0]

			subHomeNotDieNum()
		}

		if (res[1] == 'subTarge') {
			// 订阅目标
			const json = JSON.parse(res[2])
			//json.bytes
			const targetInfo = []

			// targetInfo.push(byteArrayToF32(json.bytes))

			for (let i = 0; i < 1; i++) {
				targetInfo.push({
					// 血量
					boloom: byteArrayToU32(byteOffset(json.bytes, 0x0, 4))[0],
					// 偏移0x220目标XYZ
					pos: byteArrayToF32(byteOffset(json.bytes, 0x220, 12)),
					// byteArrayToF32(byteOffset(json.bytes,4*35,12)),
				})
			}

			// console.log(targetInfo)

			gameInfo.value.targetInfo = targetInfo

			subTarge()
		}
	}
}

// 首次加载
onMounted(async () => {
	socket.open()

	// 初始化 pixijs
	await app.init({ background: '#000', backgroundAlpha: 0, resizeTo: pixiRef.value, preference: 'webgpu' })
	// 设置画布到div
	pixiRef.value.appendChild(app.canvas)

	windowInfoText.style = {
		fontSize: 14,
		fontWeight: 'bold',
		fill: 0xff0000,
		fontFamily:
			'-apple-system, BlinkMacSystemFont, Segoe UI, PingFang SC, Hiragino Sans GB, Microsoft YaHei, Helvetica Neue, Helvetica, Arial, sans-serif, Apple Color Emoji, Segoe UI Emoji, Segoe UI Symbol',
	}

	// 初始化时添加一些图形
	app.stage.addChild(windowInfoText, windowGraphics, ...targetGraphiceList)

	app.ticker.add(async () => {
		// 持续渲染

		// 渲染窗体边框

		windowGraphics.x = windowInfo.value.x
		windowGraphics.y = windowInfo.value.y
		windowGraphics.clear()
		windowGraphics.rect(0, 0, windowInfo.value.width, windowInfo.value.height).stroke({ width: 2, color: '#c3cbde' })

		// 渲染文字
		windowInfoText.x = windowInfo.value.x
		windowInfoText.y = windowInfo.value.y
		windowInfoText.text = `
    窗口X:${windowInfo.value.x} 窗口Y:${windowInfo.value.y} 窗口宽:${windowInfo.value.width} 窗口高:${windowInfo.value.height}
    房间人数:${gameInfo.value.homeNum} 存活人数:${gameInfo.value.homeNotDieNum}
    相机位置:${gameInfo.value.playerPos[0].toFixed(2)} ${gameInfo.value.playerPos[1].toFixed(2)} ${gameInfo.value.playerPos[2].toFixed(2)}

    `

		for (let i = 0; i < gameInfo.value.targetInfo.length; i++) {
			// 死亡隐藏方框
			if (gameInfo.value.targetInfo[i].boloom == 1) return (targetGraphiceList[i].visible = false)
			// 绘制透视框
			const enP = await worldToScreen(gameInfo.value.targetInfo[i].pos, gameInfo.value.cameraMatrix, windowInfo.value)

			if (enP != void 0) {
				const rw = 50
				const rh = 100

				targetGraphiceList[i].visible = true
				targetGraphiceList[i].x = enP[0] + windowInfo.value.x - rw / 2
				targetGraphiceList[i].y = enP[1] + windowInfo.value.y - rh / 2
			} else {
				targetGraphiceList[i].visible = false
			}
		}
	})
})
</script>

<template>
	<div ref="pixiRef" class="container"></div>
</template>

<style scoped></style>
<style>
body {
	margin: 0;
	padding: 0;
}

:root {
	overflow: hidden;
}

.container {
	width: 100vw;
	height: 100vh;
	overflow: hidden;
}
</style>
