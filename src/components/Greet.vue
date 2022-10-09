<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

interface HID {
  name: string
}

const greetMsg = ref("");
const name = ref("");
const hid_devices: string[] = ref();
const hid_device = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsg.value = await invoke("greet", { name: name.value });
}

async function getDevices() {
  hid_devices.values = await invoke("get_hid_devices");
}

async function getDevice() {
  hid_device.value = await invoke("get_hid_device");
}

</script>

<template>
  <div class="card">
    <input id="greet-input" v-model="name" placeholder="Enter a name..." />
    <button type="button" @click="greet()">Greet</button>
  </div>

  <div class="card">
    <button type="button" @click="getDevices()">Get all HID devices</button>
  </div>
  <div class="card">
    <button type="button" @click="getDevice()">Get HID device</button>
  </div>

  <p>{{ greetMsg }}</p>

  <p>{{ hid_device }}</p>

  <li v-for="device in hid_devices">
    {{device}}
  </li>
</template>
