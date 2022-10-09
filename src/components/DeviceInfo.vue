<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const hid_devices: string[] = ref();
const hid_device = ref("");

async function getDevice() {
  hid_device.value = await invoke("get_hid_device");
}

</script>

<template>
  <h2>Device Info</h2>
  <v-table>

  </v-table>
  <div class="card">
    <button type="button" @click="getDevices()">Get all HID devices</button>
  </div>

  <li v-for="device in hid_devices">
    {{device}}
  </li>
</template>
