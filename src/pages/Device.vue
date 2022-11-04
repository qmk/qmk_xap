<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { invoke } from '@tauri-apps/api/tauri'

import { useXAPDeviceStore } from '@/stores/devices'

const store = useXAPDeviceStore()
const { currentDevice } = storeToRefs(store)

async function lock() {
  await invoke('secure_lock', { id: currentDevice.value.id })
    .catch((error) => console.error(error))
}

async function unlock() {
  await invoke('secure_unlock', { id: currentDevice.value.id })
    .catch((error) => console.error(error))
}

async function jumpToBootloader() {
  await invoke('jump_to_bootloader', { id: currentDevice.value.id })
    .catch((error) => console.error(error))
}

async function resetEEPROM() {
  await invoke('reset_eeprom', { id: currentDevice.value.id })
    .catch((error) => console.error(error))
}

</script>

<template>
  <q-page>
    <div class="q-gutter-md q-pa-md">
      <h5>Device Information</h5>
      <q-field v-if="currentDevice?.info.qmk.manufacturer != null" filled label="Manufacturer" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ currentDevice?.info.qmk.manufacturer }}</div>
        </template>
      </q-field>
      <q-field v-if="currentDevice?.info.qmk.product_name != null" filled label="Product" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ currentDevice?.info.qmk.product_name }}</div>
        </template>
      </q-field>
      <q-field filled label="XAP Version" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ currentDevice?.info.xap.version }}</div>
        </template>
      </q-field>
      <q-field filled label="QMK Version" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ currentDevice?.info.qmk.version }}</div>
        </template>
      </q-field>
      <q-field v-if="currentDevice?.info.qmk.hardware_id != null" filled label="Hardware Id" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ currentDevice?.info.qmk.hardware_id }}</div>
        </template>
      </q-field>
      <q-field v-if="currentDevice?.info.qmk.config != null" filled label="Config JSON" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ currentDevice?.info.qmk.config }}</div>
        </template>
      </q-field>
      <h5>Secure Actions</h5>
      <q-field filled label="Secure Status" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ currentDevice?.secure_status }}</div>
        </template>
      </q-field>
      <div>
        <q-btn v-if="currentDevice?.secure_status != 'Unlocked'" class="full-width"
          :loading="currentDevice?.secure_status == 'Unlocking'" color="primary" text-color="white" label="Unlock"
          @click="unlock" />
        <q-btn v-else class="full-width" :loading="currentDevice?.secure_status == 'Unlocking'" color="primary"
          text-color="white" label="Lock" @click="lock" />
      </div>
      <div>
        <q-btn v-if="currentDevice?.info.qmk.jump_to_bootloader_enabled" :disable="currentDevice?.secure_status != 'Unlocked'" class="full-width" color="primary"
          text-color="white" label="Jump to Bootloader" @click="jumpToBootloader" />
      </div>
      <div>
        <q-btn v-if="currentDevice?.info.qmk.eeprom_reset_enabled" :disable="currentDevice?.secure_status != 'Unlocked'" class="full-width" color="primary" text-color="white"
          label="Reset EEPROM" @click="resetEEPROM" />
      </div>
    </div>
  </q-page>
</template>
