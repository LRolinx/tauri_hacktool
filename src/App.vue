<script setup lang="ts">
import { onMounted, ref } from 'vue'
import p5js from 'p5'
import { findWindow, getModuleBaseAddress, getProcessHandle, getProcessIDByName, getWindowInfo, readProcessMemoryF32, readProcessMemoryU32, WindowInfo, worldToScreen } from './hack'

const p5Ref = ref()
const windowInfo = ref<WindowInfo>({
  x: 0,
  y: 0,
  width: 0,
  height: 0,
  err: false,
})

const lastHomeNum = ref(0)
const userListBaseAddr = ref<any[]>([])

onMounted(() => {
  new p5js((p5) => {
    p5.setup = async () => {
      p5.createCanvas(p5Ref.value.offsetWidth, p5Ref.value.offsetHeight)

      sessionStorage['windowHanld'] = await findWindow('Counter-Strike Source')
      sessionStorage['pId'] = await getProcessIDByName('hl2.exe')
      sessionStorage['processHandle'] = await getProcessHandle(parseInt(sessionStorage['pId']))
      sessionStorage['serverDLLBaseAddr'] = await getModuleBaseAddress(parseInt(sessionStorage['pId']), 'server.dll')
      sessionStorage['engineDLLBaseAddr'] = await getModuleBaseAddress(parseInt(sessionStorage['pId']), 'engine.dll')
    }

    p5.draw = async () => {
      const windowHanld = parseInt(sessionStorage['windowHanld'])
      const processHandle = parseInt(sessionStorage['processHandle'])
      const serverDLLBaseAddr = parseInt(sessionStorage['serverDLLBaseAddr'])
      const engineDLLBaseAddr = parseInt(sessionStorage['engineDLLBaseAddr'])
      windowInfo.value = await getWindowInfo(windowHanld)

      const data = {
        homeNum: (await readProcessMemoryU32(processHandle, serverDLLBaseAddr + 0x4f2150)).value[0],
        cameraMatrix: (await readProcessMemoryF32(processHandle, engineDLLBaseAddr + 0x5b0d68, 64)).value,
      }

      if (lastHomeNum.value != data.homeNum) {
        // 更新房间基址地址
        const newUserListBaseAddr = []
        for (let i = 0; i < data.homeNum; i++) {
          newUserListBaseAddr.push((await readProcessMemoryU32(processHandle, serverDLLBaseAddr + 0x004f615c + i * 16)).value[0])
        }
        userListBaseAddr.value = newUserListBaseAddr
        lastHomeNum.value = data.homeNum
      }

      p5.clear()

      if (windowInfo.value?.x == -32000 && windowInfo.value?.y == -32000 && windowInfo.value?.width == 0 && windowInfo.value?.height == 0) return
      p5.textSize(14)
      p5.noStroke()
      p5.fill('red')
      p5.text(`WinInfo->${JSON.stringify(windowInfo.value)}`, windowInfo.value.x, windowInfo.value.y)
      p5.noFill()
      p5.stroke('yellow')

      for (let i = 0; i < data.homeNum; i++) {
        readProcessMemoryU32(processHandle, userListBaseAddr.value[i] + 0xe4).then(async (boolm) => {
          if (boolm.value[0] != 1) {
            // 坐标信息
            readProcessMemoryF32(processHandle, userListBaseAddr.value[i] + 0x280, 12).then(async (pos) => {
              const enP = await worldToScreen(pos.value, data.cameraMatrix, windowInfo.value)

              if (enP != undefined) {
                const rw = 50
                const rh = 100
                p5.rect(enP[0] + windowInfo.value.x - rw / 2, enP[1] + windowInfo.value.y - rh / 2, rw, rh)
                p5.text(boolm.value[0], enP[0] + windowInfo.value.x - rw / 2, enP[1] + windowInfo.value.y - rh / 2)
                p5.line(windowInfo.value.x + windowInfo.value.width / 2, windowInfo.value.y, enP[0] + windowInfo.value.x - rw / 2, enP[1] + windowInfo.value.y - rh / 2)
              }
            })
          }
        })
      }
    }
  }, p5Ref.value)
})
</script>

<template>
  <main ref="p5Ref" class="container"></main>
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
