<script setup lang="ts">
import { storeToRefs } from 'pinia'

import { ref } from 'vue'
import { useXAPDeviceStore } from '@/stores/devices'
import { secureUnlock, secureLock } from '@/commands/xap'
import { resetEEPROM, jumpToBootloader } from '@/commands/qmk'
import { XAPDeviceDTO } from '@bindings/XAPDeviceDTO';

const store = useXAPDeviceStore()
const { device }: ref<XAPDeviceDTO> = storeToRefs(store)

async function lock() {
  await secureLock(device.value.id)
}

async function unlock() {
  await secureUnlock(device.value.id)
}

async function bootloader() {
  await jumpToBootloader(device.value.id)
}

async function reset() {
  await resetEEPROM(device.value.id)
}

</script>

<template>
  <q-page>
    <div class="q-gutter-md q-pa-md">
      <h5>Device Information</h5>
      <q-field v-if="device?.info.qmk.manufacturer != null" filled label="Manufacturer" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ device?.info.qmk.manufacturer }}</div>
        </template>
      </q-field>
      <q-field v-if="device?.info.qmk.product_name != null" filled label="Product" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ device?.info.qmk.product_name }}</div>
        </template>
      </q-field>
      <q-field filled label="XAP Version" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ device?.info.xap.version }}</div>
        </template>
      </q-field>
      <q-field filled label="QMK Version" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ device?.info.qmk.version }}</div>
        </template>
      </q-field>
      <q-field v-if="device?.info.qmk.hardware_id != null" filled label="Hardware Id" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ device?.info.qmk.hardware_id }}</div>
        </template>
      </q-field>
      <q-field v-if="device?.info.qmk.config != null" filled label="Config JSON" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ device?.info.qmk.config }}</div>
        </template>
      </q-field>
      <h5>Secure Actions</h5>
      <q-field filled label="Secure Status" stack-label>
        <template v-slot:control>
          <div class="self-center full-width no-outline" tabindex="0">{{ device?.secure_status }}</div>
        </template>
      </q-field>
      <div>
        <q-btn v-if="device?.secure_status != 'Unlocked'" class="full-width"
          :loading="device?.secure_status == 'Unlocking'" color="primary" text-color="white" label="Unlock"
          @click="unlock" />
        <q-btn v-else class="full-width" :loading="device?.secure_status == 'Unlocking'" color="primary"
          text-color="white" label="Lock" @click="lock" />
      </div>
      <div>
        <q-btn v-if="device?.info.qmk.jump_to_bootloader_enabled"
          :disable="device?.secure_status != 'Unlocked'" class="full-width" color="primary" text-color="white"
          label="Jump to Bootloader" @click="bootloader" />
      </div>
      <div>
        <q-btn v-if="device?.info.qmk.eeprom_reset_enabled" :disable="device?.secure_status != 'Unlocked'"
          class="full-width" color="primary" text-color="white" label="Reset EEPROM" @click="reset" />
      </div>
    </div>
  </q-page>
</template>
