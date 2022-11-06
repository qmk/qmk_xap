<script setup lang="ts">
import { watch, reactive, computed, onMounted, nextTick } from 'vue'
import { watchPausable } from '@vueuse/core'
import { storeToRefs } from 'pinia'
import ColorPicker from '@radial-color-picker/vue-color-picker'

import { RGBConfig } from '@bindings/RGBConfig'
import { useXAPDeviceStore } from '@/stores/devices'
import { XAPDevice } from '@/stores/devices'
import { saveConfig, getConfig, setConfig } from '@/commands/lighting/rgblight'
import { notifyError } from '@/utils/utils'

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

onMounted(async () => {
  pause()
  if (currentDevice.value) {
    try {
      currentConfig.value = await getConfig(currentDevice.value.id)
    } catch (err) {
      notifyError(err)
    }
  }
  await nextTick()
  resume()
})

watch(currentDevice, async (device: XAPDevice) => {
  pause()
  try {
    if (currentDevice.value) {
      currentConfig.value = await getConfig(device.id)
    }
  } catch (err) {
    notifyError(err)
  }
  await nextTick()
  resume()
})

const { stop, pause, resume } = watchPausable(currentConfig, async (config: RGBConfig) => {
  try {
    await setConfig(currentDevice.value.id, config)
  } catch (err) {
    notifyError(err)
  }
})

async function save() {
  try {
    await saveConfig(currentDevice.value.id)
  } catch (err) {
    notifyError(err)
  }
}

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
          <q-btn color="white" text-color="black" label="Save" @click="save" />
        </div>
      </div>
    </div>
  </q-page>
</template>

<style>
@import '@radial-color-picker/vue-color-picker/dist/vue-color-picker.min.css';
</style>
