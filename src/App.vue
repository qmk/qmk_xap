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
    let id = event.payload.id
    // console.log("new device with id " + id)

    let xapDevice: XAPDevice = { id, info: event.payload.device, secure_status: "Disabled" };
    store.addDevice(id, xapDevice)
  })
  await listen('removed-device', (event: Event<FrontendEvent>) => {
    let id = event.payload.id
    console.log("removed device with id " + id)
    store.removeDevice(id)
  })
  await listen('secure-status-changed', (event: Event<FrontendEvent>) => {
    let id = event.payload.id
    console.log("secure status changed for device " + id)
    store.updateSecureStatus(id, event.payload.secure_status)
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
