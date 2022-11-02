<script setup lang="ts">
import { ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { invoke } from '@tauri-apps/api/tauri'

import { useXAPDeviceStore, XAPDevice } from '@/stores/devices'

const store = useXAPDeviceStore()
const { currentDevice } = storeToRefs(store)

</script>

<template>
  <q-page>
    <div class="q-gutter-md q-pa-md">
      <h2>Device Info</h2>
      <q-field v-if="currentDevice.info.qmk.manufacturer != null" filled label="Manufacturer" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ currentDevice.info.qmk.manufacturer }}</div>
        </template>
      </q-field>
      <q-field v-if="currentDevice.info.qmk.product_name != null" filled label="Product" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ currentDevice.info.qmk.product_name }}</div>
        </template>
      </q-field>
      <q-field filled label="XAP Version" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ currentDevice.info.xap.version }}</div>
        </template>
      </q-field>
      <q-field filled label="QMK Version" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ currentDevice.info.qmk.version }}</div>
        </template>
      </q-field>
      <q-field v-if="currentDevice.info.qmk.hardware_id != null" filled label="Hardware Id" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ currentDevice.info.qmk.hardware_id }}</div>
        </template>
      </q-field>
      <q-field v-if="currentDevice.info.qmk.config != null" filled label="Config JSON" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ currentDevice.info.qmk.config }}</div>
        </template>
      </q-field>
    </div>
  </q-page>
</template>
