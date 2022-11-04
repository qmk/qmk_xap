<script setup lang="ts">

import { storeToRefs } from 'pinia'
import { computed, ref } from 'vue'

import { useXAPDeviceStore, XAPDevice } from '@/stores/devices'

const store = useXAPDeviceStore()
const { currentDevice, devices } = storeToRefs(store)
const devicesA: ref<Array<XAPDevice>> = computed(() => Array.from(devices.value.values()))

</script>

<template>
  <q-layout view="hHh LpR fff">
    <q-header class="bg-primary text-white" height-hint="98">
      <q-toolbar>
        <q-toolbar-title>
          <q-avatar>
            <img src="src/assets/qmk.svg">
          </q-avatar>
          QMK XAP GUI
        </q-toolbar-title>
        <q-tabs align="left">
          <q-route-tab label="Device" :disable="currentDevice == null" to="/device" exact />
          <q-route-tab label="Keymap" :disable="currentDevice?.keymap == null" to="/keymap" exact />
          <q-route-tab label="RGB" v-if="currentDevice?.info?.lighting?.rgblight != null" to="/rgb" exact />
        </q-tabs>
      </q-toolbar>
      <div class="bg-white">
      <q-select label="XAP device" :disable="currentDevice == null" filled v-model="currentDevice" :options="devicesA"
        :option-label="(device: XAPDevice) => (device.info.qmk.manufacturer ?? 'unknown manufacturer') + ' - ' + (device.info.qmk.product_name ?? 'unknown product')"
        emit-value />
      </div>
    </q-header>
    <q-page-container>
      <router-view v-if="currentDevice != null" />
    </q-page-container>
  </q-layout>
</template>
