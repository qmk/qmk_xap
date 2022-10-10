<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const HidDevices: Array<string> = ref();

async function getDevice() {
  HidDevices.values = await invoke("get_xap_devices");
  console.log(HidDevices)
}

</script>

<template>
  <h2>Device Info</h2>
  <v-table>

  </v-table>
  <div class="card">
    <button type="button" @click="getDevice()">Get all HID devices</button>
  </div>

  <li v-for="device in HidDevices">
    {{device}}
  </li>
</template>
