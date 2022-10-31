<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { RGBConfig } from "RGBConfig";
import { colord, HsvColor } from "colord";

export interface RGB {
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
const enabledEffects: ref<Array<number>> = ref([])

watch(currentColor, async (newColor: RGB, oldColor: RGB) => {
  // TODO DOESN'T WORK
  if (newColor === oldColor) {
    return
  }
  let hsv = colord(newColor).toHsv()
  currentConfig.value.hue = Math.ceil(hsv.h / 360 * 255)
  currentConfig.value.sat = Math.ceil(hsv.s / 100 * 255)
  currentConfig.value.val = Math.ceil(hsv.v / 100 * 255)
  await setConfig()
})

onMounted(async () => {
  await getConfig()
  await getRGBlightEffects()
  let hsv = colord({ h: Math.ceil(currentConfig.value.hue / 255 * 360), s: Math.ceil(currentConfig.value.sat / 255 * 100), v: Math.ceil(currentConfig.value.val / 255 * 100) })
  currentColor.value = hsv.toRgb()
})

async function getConfig() {
  await invoke('get_rgblight_config', { id: 'bf7a8aff-57a1-4522-9dfa-c93925d85c72' })
    .then((config: RGBConfig) => { currentConfig.value = config })
    .catch((error) => console.error(error))
}

async function getRGBlightEffects() {
  await invoke('get_rgblight_effects', { id: 'bf7a8aff-57a1-4522-9dfa-c93925d85c72' })
    .then((effects: Array<number>) => { enabledEffects.value = effects })
    .catch((error) => console.error(error))
}

async function setConfig() {
  console.log("set config")
  console.log(currentConfig)
  await invoke('set_rgblight_config', { id: 'bf7a8aff-57a1-4522-9dfa-c93925d85c72', arg: currentConfig.value })
    .catch((error) => console.error(error))
}

async function saveConfig() {
  await invoke('save_rgblight_config', { id: 'bf7a8aff-57a1-4522-9dfa-c93925d85c72' })
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

  <div class="q-gutter-md q-pa-md">
    <h2>RGB</h2>
    <q-select v-model.lazy.number="currentConfig.mode" @update:modelValue="changeMode" :options="enabledEffects"
      label="Mode" emit-value />
    <q-color v-model.lazy="currentColor" default-view="palette" format-model="rgb" no-header class="rgbPicker" />
    <q-btn color="white" text-color="black" label="Save" @click="saveConfig" />
  </div>
</template>
