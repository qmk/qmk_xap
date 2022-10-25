<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const XAPDevice: ref<string> = ref("");

async function getDevice() {
  XAPDevice.value = await invoke("get_xap_device");
  console.log(XAPDevice)
}

async function getSecureStatus() {
  await invoke("get_secure_status")
}

</script>

<template>
    <h2>Device Info</h2>
    <button type="button" @click="getDevice()">Show Device Info</button>
    <p>{{XAPDevice}}</p>
    <button type="button" @click="getSecureStatus()">Get Secure Status</button>
</template>
