<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { findWindow, getModuleBaseAddress, getProcessHandle, getProcessIDByName, getWindowInfo, readProcessMemoryF32, readProcessMemoryU32, WindowInfo, worldToScreen } from './hack'
import { Application, Graphics, Text } from 'pixi.js'

const pixiRef = ref()
const app = new Application()
const windowGraphics = new Graphics()
const windowInfoText = new Text()

const windowInfo = ref<WindowInfo>({
  x: 0,
  y: 0,
  width: 0,
  height: 0,
  err: false,
})

const lastHomeNum = ref(0)
const userListBaseAddr = ref<any[]>([])

// 初始化
const init = async () => {
  findWindow('任务管理器')
  getProcessIDByName('hl2.exe')

  //   sessionStorage['processHandle'] = await getProcessHandle(parseInt(sessionStorage['pId']))
  //   sessionStorage['serverDLLBaseAddr'] = await getModuleBaseAddress(parseInt(sessionStorage['pId']), 'server.dll')
  //   sessionStorage['engineDLLBaseAddr'] = await getModuleBaseAddress(parseInt(sessionStorage['pId']), 'engine.dll')

  // 初始化socket消息监听
  window.ws!.onmessage = ({ data }: { data: string }) => {
    // console.log(data)
    const res = data.split(' ')
    if (res[0] == 'findWindow') {
      // 返回的查找窗口
      sessionStorage['windowHanld'] = res[1]
    }

    if (res[0] == 'getWindowInfo') {
      const json = JSON.parse(data.replace('getWindowInfo ', ''))
      windowInfo.value = json

      // 渲染窗体边框
      windowGraphics.clear()
      windowGraphics.rect(windowInfo.value.x, windowInfo.value.y, windowInfo.value.width, windowInfo.value.height).stroke({ width: 2, color: '#c3cbde' })

      // 渲染文字
      windowInfoText.x = windowInfo.value.x
      windowInfoText.y = windowInfo.value.y
      windowInfoText.text = JSON.stringify(windowInfo.value)
    }
  }

  // 初始化 pixijs
  await app.init({ background: '#fff', backgroundAlpha: 0, resizeTo: pixiRef.value, preference: 'webgpu' })
  // 设置画布到div
  pixiRef.value.appendChild(app.canvas)

  windowInfoText.style = {
    fontSize: 14,
    fontWeight: 'bold',
    fill: 0xff0000,
    fontFamily:
      '-apple-system, BlinkMacSystemFont, Segoe UI, PingFang SC, Hiragino Sans GB, Microsoft YaHei, Helvetica Neue, Helvetica, Arial, sans-serif, Apple Color Emoji, Segoe UI Emoji, Segoe UI Symbol',
  }

  //   rootContainer.addChild(windowInfoText)

  // 初始化时添加一些图形
  app.stage.addChild(windowInfoText, windowGraphics)

  app.ticker.add(() => {
    // 这里持续获取请求
    getWindowInfo(sessionStorage['windowHanld'])
  })
}

onMounted(init)

//   new p5js((p5: p5) => {
//     p5.setup = async () => {
//       p5.createCanvas(p5Ref.value.offsetWidth, p5Ref.value.offsetHeight)

//       //   sessionStorage['windowHanld'] = await findWindow('任务管理器')
//       //   console.log(sessionStorage['windowHanld'])
//       //   sessionStorage['pId'] = await getProcessIDByName('hl2.exe')
//       //   sessionStorage['processHandle'] = await getProcessHandle(parseInt(sessionStorage['pId']))
//       //   sessionStorage['serverDLLBaseAddr'] = await getModuleBaseAddress(parseInt(sessionStorage['pId']), 'server.dll')
//       //   sessionStorage['engineDLLBaseAddr'] = await getModuleBaseAddress(parseInt(sessionStorage['pId']), 'engine.dll')
//     }

//     p5.draw = async () => {
//     //   const windowHanld = parseInt(sessionStorage['windowHanld'])
//     //   windowInfo.value = await getWindowInfo(windowHanld)
// 	window.ws!.send(`getWindowInfo ${sessionStorage['windowHanld']}`)

//       // 如果窗口是最小化则不继续绘制
//       p5.clear()
//       if (windowInfo.value?.x == -32000 && windowInfo.value?.y == -32000 && windowInfo.value?.width == 0 && windowInfo.value?.height == 0) return
//       p5.textSize(14)
//     //   p5.noStroke()
//       p5.fill('red')
//       p5.text(`WinInfo->${JSON.stringify(windowInfo.value)}`, windowInfo.value.x, windowInfo.value.y)
//       p5.noFill()

//       // 下面是绘制方框的逻辑

//     //   const processHandle = parseInt(sessionStorage['processHandle'])
//     //   const serverDLLBaseAddr = parseInt(sessionStorage['serverDLLBaseAddr'])
//     //   const engineDLLBaseAddr = parseInt(sessionStorage['engineDLLBaseAddr'])

//       // 给窗口画个框
//       p5.rect(windowInfo.value.x, windowInfo.value.y, windowInfo.value.width, windowInfo.value.height)

//     //   // 没找到对应的基址跳出
//     //   if (processHandle == void 0 || serverDLLBaseAddr == void 0 || engineDLLBaseAddr == void 0) return

//     //   const data = {
//     //     homeNum: (await readProcessMemoryU32(processHandle, serverDLLBaseAddr + 0x4f2150)).value[0],
//     //     cameraMatrix: (await readProcessMemoryF32(processHandle, engineDLLBaseAddr + 0x5b0d68, 64)).value,
//     //   }

//     //   if (lastHomeNum.value != data.homeNum) {
//     //     // 更新房间基址地址
//     //     const newUserListBaseAddr = []
//     //     for (let i = 0; i < data.homeNum; i++) {
//     //       newUserListBaseAddr.push((await readProcessMemoryU32(processHandle, serverDLLBaseAddr + 0x004f615c + i * 16)).value[0])
//     //     }
//     //     userListBaseAddr.value = newUserListBaseAddr
//     //     lastHomeNum.value = data.homeNum
//     //   }

//     //   p5.stroke('yellow')

//     //   for (let i = 0; i < data.homeNum; i++) {
//     //     readProcessMemoryU32(processHandle, userListBaseAddr.value[i] + 0xe4).then(async (boolm) => {
//     //       if (boolm.value[0] != 1) {
//     //         // 坐标信息
//     //         readProcessMemoryF32(processHandle, userListBaseAddr.value[i] + 0x280, 12).then(async (pos) => {
//     //           const enP = await worldToScreen(pos.value, data.cameraMatrix, windowInfo.value)

//     //           if (enP != undefined) {
//     //             const rw = 50
//     //             const rh = 100
//     //             p5.rect(enP[0] + windowInfo.value.x - rw / 2, enP[1] + windowInfo.value.y - rh / 2, rw, rh)
//     //             p5.text(boolm.value[0], enP[0] + windowInfo.value.x - rw / 2, enP[1] + windowInfo.value.y - rh / 2)
//     //             p5.line(windowInfo.value.x + windowInfo.value.width / 2, windowInfo.value.y, enP[0] + windowInfo.value.x - rw / 2, enP[1] + windowInfo.value.y - rh / 2)
//     //           }
//     //         })
//     //       }
//     //     })
//     //   }
//     }
//   }, p5Ref.value)
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
