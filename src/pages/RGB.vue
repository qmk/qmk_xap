<script setup lang="ts">
import { watch, reactive, computed, onMounted, nextTick } from "vue"
import { watchPausable } from '@vueuse/core'
import { storeToRefs } from 'pinia'
import { invoke } from "@tauri-apps/api/tauri"
import ColorPicker from '@radial-color-picker/vue-color-picker'


import { RGBConfig } from "@bindings/RGBConfig"
import { useXAPDeviceStore } from '@/stores/devices'
import { XAPDevice } from "@/stores/devices"

const store = useXAPDeviceStore()
const { currentDevice } = storeToRefs(store)

const currentConfig: reactive<RGBConfig> = reactive({
  enable: 1,
  mode: 1,
  hue: 255,
  sat: 255,
  val: 255,
  speed: 255
})

const hue = computed({
  get() {
    return Math.ceil(currentConfig.hue / 255 * 360)
  },
  set(h: number) {
    currentConfig.hue = Math.ceil(h / 360 * 255)
  }
})

async function updateHue(h: number) {
  hue.value = h
}


async function saveConfig() {
  await invoke('rgblight_config_save', { id: currentDevice.value.id })
    .catch((error) => console.error(error))
}

async function getConfig(): Promise<RGBConfig> {
  return await invoke('rgblight_config_get', { id: currentDevice.value.id })
    .then((config: RGBConfig) => {
      return config
    })
    .catch((error) => console.error(error))
}


onMounted(async () => {
  pause()
  currentConfig.value = await getConfig()
  await nextTick()
  resume()
})

watch(currentDevice, async ({ }) => {
  pause()
  currentConfig.value = await getConfig()
  await nextTick()
  resume()
})

const { stop, pause, resume } = watchPausable(currentConfig, async (config: RGBConfig) => {
  await invoke('rgblight_config_set', { id: currentDevice.value.id, arg: config })
    .catch((error) => console.error(error))
})

</script>

<template>
  <q-page>
    <div class="q-gutter-md q-pa-md">
      <div class="row flex-center">
        <div class="col-4">
          <color-picker :hue="hue" @input="updateHue" />
        </div>
        <div class="col q-gutter-y-sm">
          <q-select v-model.number.lazy="currentConfig.mode" :options="currentDevice.info.lighting.rgblight.effects"
            label="Mode" emit-value />
          <q-badge>
            Hue
          </q-badge>
          <q-slider v-model.number.lazy="currentConfig.hue" :min="0" :max="255" label marker-labels :markers="32" />
          <q-badge>
            Saturation
          </q-badge>
          <q-slider v-model.number.lazy="currentConfig.sat" :min="0" :max="255" label marker-labels :markers="32" />
          <q-badge>
            Value
          </q-badge>
          <q-slider v-model.number.lazy="currentConfig.val" :min="0" :max="255" label marker-labels :markers="32" />
          <q-badge>
            Speed
          </q-badge>
          <q-slider v-model.number.lazy="currentConfig.speed" :min="0" :max="255" label marker-labels :markers="32" />
          <q-btn color="white" text-color="black" label="Save" @click="saveConfig" />
        </div>
      </div>
    </div>
  </q-page>
</template>

<style>
@import '@radial-color-picker/vue-color-picker/dist/vue-color-picker.min.css';
</style>
