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

let currentConfig: ref<RGBConfig> = ref({
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

watch(currentConfig, async (newConfig: RGBConfig, oldConfig: RGBConfig) => {
  await setConfig()
})

onMounted(async () => {
  await getConfig()
  await getRGBlightEffects()
  let hsv = colord({ h: Math.ceil(currentConfig.value.hue / 255 * 360), s: Math.ceil(currentConfig.value.sat / 255 * 100), v: Math.ceil(currentConfig.value.val / 255 * 100) })
  currentColor.value = hsv.toRgb()
})

async function getConfig() {
  await invoke('get_rgblight_config')
    .then((config: RGBConfig) => { currentConfig.value = config })
    .catch((error) => console.error(error))
}

async function getRGBlightEffects() {
  await invoke('get_rgblight_effects')
    .then((effects: Array<number>) => { enabledEffects.value = effects })
    .catch((error) => console.error(error))
}

async function setConfig() {
  await invoke('set_rgblight_config', { arg: currentConfig.value })
    .catch((error) => console.error(error))
}

async function saveConfig() {
  await invoke('save_rgblight_config')
    .catch((error) => console.error(error))
}

</script>

<template>
  <h2>RGB</h2>
  <div class="d-flex flex-column">
    <v-select label="Mode" @select="setConfig()" :items="enabledEffects" v-model="currentConfig.mode"></v-select>
    <v-color-picker hide-mode-switch show-swatches :modes="['rgb']" v-model="currentColor" />
    <v-btn type="button" @click="saveConfig()">Save</v-btn>
  </div>
</template>
