<script setup lang="ts">

import { storeToRefs } from 'pinia'
import { computed, ref } from 'vue'

import { secureUnlock, secureLock } from '@/commands/xap'
import { useXAPDeviceStore, XAPDevice } from '@/stores/devices'

const store = useXAPDeviceStore()
const { device, devices } = storeToRefs(store)
const devicesA: ref<Array<XAPDevice>> = computed(() => Array.from(devices.value.values()))

async function lock() {
  await secureLock(device.value.id)
}

async function unlock() {
  await secureUnlock(device.value.id)
}

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
          <q-route-tab label="Device" :disable="device == null" to="/device" exact />
          <q-route-tab label="Keymap" :disable="device?.info.keymap == null" to="/keymap" exact />
          <q-route-tab label="RGB" v-if="device?.info?.lighting?.rgblight != null" to="/rgb" exact />
        </q-tabs>
      </q-toolbar>
      <div class="bg-white">
        <q-select label="XAP device" :disable="device == null" filled v-model="device" :options="devicesA"
          :option-label="(device: XAPDevice) => (device.info.qmk.manufacturer ?? 'unknown manufacturer') + ' - ' + (device.info.qmk.product_name ?? 'unknown product')"
          emit-value />
      </div>
    </q-header>
    <q-page-container>
      <router-view v-if="device != null" />
    </q-page-container>
    <q-page-sticky position="bottom-right" :offset="[24, 24]">
      <q-btn fab icon="lock_open" v-if="device?.secure_status != 'Unlocked'"
        :loading="device?.secure_status == 'Unlocking'" color="secondary" text-color="white" @click="unlock" />
      <q-btn fab v-else :loading="device?.secure_status == 'Unlocking'" color="secondary" text-color="white" icon="lock"
        @click="lock" />
    </q-page-sticky>
  </q-layout>
</template>
