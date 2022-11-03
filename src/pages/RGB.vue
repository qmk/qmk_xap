<script setup lang="ts">
import { ref, watch } from "vue"
import { storeToRefs } from 'pinia'
import { invoke } from "@tauri-apps/api/tauri"
import { colord } from "colord"

import { RGBConfig } from "@bindings/RGBConfig"
import { useXAPDeviceStore } from '@/stores/devices'
import { XAPDevice } from "@/stores/devices"

const store = useXAPDeviceStore()
const { currentDevice } = storeToRefs(store)

interface RGB {
  r: number,
  g: number,
  b: number
}

const currentConfig: ref<RGBConfig> = ref({
  enable: 1,
  mode: 1,
  hue: 255,
  sat: 255,
  val: 255,
  speed: 60
})

const currentColor: ref<RGB> = ref({ r: 33, g: 128, b: 255 })

watch(currentColor, async (color: RGB) => {
  let hsv = colord(color).toHsv()
  currentConfig.value.hue = Math.ceil(hsv.h / 360 * 255)
  currentConfig.value.sat = Math.ceil(hsv.s / 100 * 255)
  currentConfig.value.val = Math.ceil(hsv.v / 100 * 255)
  await setConfig()
})

watch(currentDevice, async (device: XAPDevice | null) => {
  if (device != null) {
    await getConfig()
    let hsv = colord({ h: Math.ceil(currentConfig.value.hue / 255 * 360), s: Math.ceil(currentConfig.value.sat / 255 * 100), v: Math.ceil(currentConfig.value.val / 255 * 100) })
    currentColor.value = hsv.toRgb()
    return;
  }
})

async function getConfig() {
  await invoke('rgblight_config_get', { id: currentDevice.value.id })
    .then((config: RGBConfig) => { currentConfig.value = config })
    .catch((error) => console.error(error))
}

async function setConfig() {
  console.log("set config")
  console.log(currentConfig.value)
  await invoke('rgblight_config_set', { id: currentDevice.value.id, arg: currentConfig.value })
    .catch((error) => console.error(error))
}

async function saveConfig() {
  await invoke('rgblight_config_save', { id: currentDevice.value.id })
    .catch((error) => console.error(error))
}

// WTF - why is this even necessary
async function changeMode(mode: number) {
  console.log(mode)
  currentConfig.value.mode = mode
  await setConfig()
}
</script>

<template>
  <q-page>
    <div class="q-gutter-md q-pa-md">
      <q-select v-model.lazy.number="currentConfig.mode" @update:modelValue="changeMode"
        :options="currentDevice?.info?.lighting?.rgblight?.effects" label="Mode" emit-value />
      <q-color v-model.lazy="currentColor" default-view="palette" format-model="rgb" no-header class="rgbPicker" />
      <q-btn color="white" text-color="black" label="Save" @click="saveConfig" />
    </div>
  </q-page>
</template>
