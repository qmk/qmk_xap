<script setup lang="ts">
import { ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { invoke } from '@tauri-apps/api/tauri'

import { useXAPDeviceStore } from '@/stores/devices'

const XAPDevice: ref<string> = ref("");
const XAPSecureStatus: ref<string> = ref("");
const XAPVersion: ref<string> = ref("");

const store = useXAPDeviceStore()
const { current_id } = storeToRefs(store)

watch(current_id, async (newId: String | null) => {
  if (newId != null) {
    XAPDevice.value = await invoke("get_xap_device", { id: newId });
    XAPSecureStatus.value = await invoke("get_secure_status", { id: newId })
    XAPVersion.value = await invoke("get_xap_version", { id: newId })
    return;
  }

  XAPDevice.value = ''
  XAPSecureStatus.value = ''
  XAPVersion.value = ''
})


</script>

<template>
  <div class="q-gutter-md q-pa-md">
    <h2>Device Info</h2>
    <q-field filled label="Device" stack-label>
      <template v-slot:control>
        <div class="self-center full-width no-outline" tabindex="0">{{ XAPDevice }}</div>
      </template>
    </q-field>

    <q-field filled label="Secure Status" stack-label>
      <template v-slot:control>
        <div class="self-center full-width no-outline" tabindex="0">{{ XAPSecureStatus }}</div>
      </template>
    </q-field>

    <q-field filled label="XAP Version" stack-label>
      <template v-slot:control>
        <div class="self-center full-width no-outline" tabindex="0">{{ XAPVersion }}</div>
      </template>
    </q-field>
  </div>
</template>
