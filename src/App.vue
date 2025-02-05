<script setup lang="ts">
import { nextTick, onMounted, reactive, ref } from 'vue'
import p5js from 'p5'
import { findWindow, GetModuleBaseAddress, GetProcessHandle, GetProcessIDByName, getWindowInfo, ReadProcessMemoryF32, ReadProcessMemoryU32, WindowInfo, worldToScreen } from './hack'

const p5ref = ref()

// const windowHanld = ref<any>()
const windowInfo = ref<WindowInfo>({
  x: 0,
  y: 0,
  width: 0,
  height: 0,
  err: false,
})

const lastHomeNum = ref(0)

const userListBaseAddr = ref<any[]>([])

function reverseLinearNormalize(x: number, dataMin: number, dataMax: number, min: number, max: number) {
  // 计算归一化值 (这里是反向映射的)
  const normalized = (x - dataMin) / (dataMax - dataMin)

  // 将归一化值映射到 [max, min] 范围
  const mapped = normalized * (max - min) + min

  // 返回反转后的结果：x越小，返回值越接近max
  let data = max - mapped

  return data <= min ? min : data >= max ? max : data
}

onMounted(() => {

	new p5js((p5) => {
      p5.setup = async () => {
        // console.log(p5)
        p5.createCanvas(p5ref.value.offsetWidth, p5ref.value.offsetHeight)
        // 蓝蝶原本名称为 BlueStacks
        sessionStorage['windowHanld'] = await findWindow('Counter-Strike Source')
        sessionStorage['pId'] = await GetProcessIDByName('hl2.exe')
        sessionStorage['processHandle'] = await GetProcessHandle(parseInt(sessionStorage['pId']))
        sessionStorage['serverDLLBaseAddr'] = await GetModuleBaseAddress(parseInt(sessionStorage['pId']), 'server.dll')
        sessionStorage['engineDLLBaseAddr'] = await GetModuleBaseAddress(parseInt(sessionStorage['pId']), 'engine.dll')
      }

	  p5.draw = async () => {
      const windowHanld = parseInt(sessionStorage['windowHanld'])
      const processHandle = parseInt(sessionStorage['processHandle'])
      const serverDLLBaseAddr = parseInt(sessionStorage['serverDLLBaseAddr'])
      const engineDLLBaseAddr = parseInt(sessionStorage['engineDLLBaseAddr'])
      windowInfo.value = await getWindowInfo(windowHanld)

      const data = {
        homeNum: (await ReadProcessMemoryU32(processHandle, serverDLLBaseAddr + 0x4f2150)).value[0],
        cameraMatrix: (await ReadProcessMemoryF32(processHandle, engineDLLBaseAddr + 0x5b0d68, 64)).value,
      }

      if (lastHomeNum.value != data.homeNum) {
        // 更新房间基址地址
        const newUserListBaseAddr = []
        for (let i = 0; i < data.homeNum; i++) {
          newUserListBaseAddr.push((await ReadProcessMemoryU32(processHandle, serverDLLBaseAddr + 0x004f615c + i * 16)).value[0])
        }
        userListBaseAddr.value = newUserListBaseAddr
        lastHomeNum.value = data.homeNum
      }

      // if (windowInfo.err) {
      //   windowHanld.value = await findWindow('Counter-Strike')
      //   windowInfo = await getWindowInfo(windowHanld.value)
      // }
      p5.clear()

      if (windowInfo.value?.x == -32000 && windowInfo.value?.y == -32000 && windowInfo.value?.width == 0 && windowInfo.value?.height == 0) return
      p5.textSize(14)
      p5.noStroke()
      p5.fill('red')
      // p5.text(`${Date.now()}`, 0, 20)
      p5.text(`WinInfo->${JSON.stringify(windowInfo.value)}`, windowInfo.value.x, windowInfo.value.y)
      // p5.text(`房间人数->${data.homeNum}`, windowInfo.value.x, windowInfo.value.y + 20)
      // p5.text(`玩家坐标->${data.userPos[0]},${data.userPos[1]},${data.userPos[2]}`, windowInfo.value.x, windowInfo.value.y + 40)
      // p5.text(`敌人坐标->${data.enemyPos[0]},${data.enemyPos[1]},${data.enemyPos[2]}`, windowInfo.value.x, windowInfo.value.y + 60)
      // p5.text(`相机矩阵->${data.cameraMatrix}`, 0, 120)
      // p5.text(`敌人距离->${dir}`, windowInfo.value.x, windowInfo.value.y + 80)
      p5.noFill()
      p5.stroke('yellow')

      for (let i = 0; i < data.homeNum; i++) {
        // if (i == 0) continue
        // 血量
        //   const boolm = (await ReadProcessMemoryU32(processHandle, userListBaseAddr.value[i] + 0xe4)).value[0]

        ReadProcessMemoryU32(processHandle, userListBaseAddr.value[i] + 0xe4).then(async (boolm) => {
          if (boolm.value[0] != 1) {
            // 坐标信息
            //   const pos = (await ReadProcessMemoryF32(processHandle, userListBaseAddr.value[i] + 0x280, 12)).value
            ReadProcessMemoryF32(processHandle, userListBaseAddr.value[i] + 0x280, 12).then(async (pos) => {
              const enP = await worldToScreen(pos.value, data.cameraMatrix, windowInfo.value)
              //   const dir = await calculateSizeBasedOnistance(data.userPos, data.enemyPos)

              if (enP != undefined) {
                const bb = reverseLinearNormalize(0, 0, 3000, 0.1, 1)
                const rw = 50 * bb
                const rh = 100 * bb
                p5.rect(enP[0] + windowInfo.value.x - rw / 2, enP[1] + windowInfo.value.y - rh / 2, rw, rh)
                p5.text(boolm.value[0], enP[0] + windowInfo.value.x - rw / 2, enP[1] + windowInfo.value.y - rh / 2)
                p5.line(windowInfo.value.x + windowInfo.value.width / 2, windowInfo.value.y, enP[0] + windowInfo.value.x - rw / 2, enP[1] + windowInfo.value.y - rh / 2)
              }
            })
          }
        })
      }
    }
    }, p5ref.value)

})
</script>

<template>
  <main ref="p5ref" class="container"></main>
</template>

<style scoped></style>
<style>
body {
  margin: 0;
  padding: 0;
}

:root {
  overflow: hidden;
  /* border: 1px solid #008cff; */
}

.container {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

canvas {
  /* width: 100%;
  height: 100%; */
  /* border: 1px solid #ffee00; */
}
</style>
