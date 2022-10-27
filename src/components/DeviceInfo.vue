<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const XAPDevice: ref<string> = ref("");
const XAPSecureStatus: ref<string> = ref("");
const XAPVersion: ref<string> = ref("");

onMounted(async () => {
  XAPDevice.value = await invoke("get_xap_device");
  XAPSecureStatus.value = await invoke("get_secure_status")
  XAPVersion.value = await invoke("get_xap_version")
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
