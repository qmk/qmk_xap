<script setup lang="ts">

import { storeToRefs } from 'pinia'
import { onMounted, computed, ref, watch } from 'vue'
import { listen, Event } from '@tauri-apps/api/event'

import { useXAPDeviceStore, XAPDevice } from '@/stores/devices'
import DeviceInfo from '@/components/DeviceInfo.vue'
import RGB from '@/components/RGB.vue'
import { FrontendEvent } from '@bindings/FrontendEvent'

const store = useXAPDeviceStore()
const { currentDevice, devices } = storeToRefs(store)
const devicesA: ref<Array<XAPDevice>> = computed(() => Array.from(devices.value.values()))

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
        <q-select filled v-model="currentDevice" :options="devicesA" :option-label="(device: XAPDevice) => (device.info.qmk.manufacturer ?? 'unknown manufacturer') + ': ' + (device.info.qmk.product_name ?? 'unknown product')" label="XAP Device" emit-value/>
      </q-toolbar>

      <q-tabs align="left">
        <q-route-tab v-if="currentDevice != null" to="/page1" label="Device" />
        <q-route-tab v-if="currentDevice?.keymap != null" to="/page2" label="Keymap" />
        <q-route-tab v-if="currentDevice?.info?.lighting?.rgblight != null" to="/page3" label="RGB" />
      </q-tabs>
    </q-header>

    <q-page-container>
      <div class="q-pa-md">
        <div class="row">
          <div class="col">
            <DeviceInfo v-if="currentDevice != null"/>
          </div>
          <div class="col-6">
            <RGB v-if="currentDevice?.info?.lighting?.rgblight != null"/>
          </div>
        </div>
      </div>

      <router-view />
    </q-page-container>

    <q-footer elevated class="bg-grey-8 text-white">
      <q-toolbar>
        <q-toolbar-title>
          <q-avatar>
            <img src="src/assets/qmk.svg">
          </q-avatar>
        </q-toolbar-title>
      </q-toolbar>
    </q-footer>

  </q-layout>
</template>
