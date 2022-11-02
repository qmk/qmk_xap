<script setup lang="ts">

import { storeToRefs } from 'pinia'
import { onMounted, computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { listen, Event } from '@tauri-apps/api/event'

import { useXAPDeviceStore, XAPDevice } from '@/stores/devices'
import { FrontendEvent } from '@bindings/FrontendEvent'
import router from '@/router/routes'

const store = useXAPDeviceStore()
const { currentDevice, devices } = storeToRefs(store)
const devicesA: ref<Array<XAPDevice>> = computed(() => Array.from(devices.value.values()))
const route = useRoute()

onMounted(async () => {
  await listen('new-device', (event: Event<FrontendEvent>) => {
    console.log("new device with id" + event.payload.id)

    let xapDevice: XAPDevice = { id: event.payload.id, info: event.payload.device };
    store.addDevice(event.payload.id, xapDevice)
  })
  await listen('removed-device', (event: Event<FrontendEvent>) => {
    console.log("removed device with id" + event.payload.id)
    store.removeDevice(event.payload.id)
  })
})

watch(currentDevice, async (device: XAPDevice) => {
  if (device == null && devices.value.size == 0) {
    router.push('/intermission')
  } else if (device != null && route.path == '/intermission') {
    router.push('/device')
  }
})

</script>

<template>
  <q-layout view="hHh LpR fff">

    <q-header elevated class="bg-primary text-white" height-hint="98">
      <q-toolbar>
        <q-toolbar-title>
          <q-avatar>
            <img src="src/assets/qmk.svg">
          </q-avatar>
          QMK XAP GUI
        </q-toolbar-title>
        <q-select label="XAP Device" v-if="currentDevice != null" filled v-model="currentDevice" :options="devicesA"
          :option-label="(device: XAPDevice) => (device.info.qmk.manufacturer ?? 'unknown manufacturer') + ': ' + (device.info.qmk.product_name ?? 'unknown product')"
          emit-value />
      </q-toolbar>

      <q-tabs align="left">
        <q-route-tab label="Device" v-if="currentDevice != null" to="/device" exact />
        <q-route-tab label="Keymap" v-if="currentDevice?.keymap != null" to="/keymap" exact />
        <q-route-tab label="RGB" v-if="currentDevice?.info?.lighting?.rgblight != null" to="/rgb" exact />
      </q-tabs>
    </q-header>

    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
</template>
