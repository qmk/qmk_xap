<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const XAPDevice: ref<string> = ref("");
const XAPSecureStatus: ref<string> = ref("");
const XAPVersion: ref<string> = ref("");

async function getDevice() {
  XAPDevice.value = await invoke("get_xap_device");
}

async function getSecureStatus() {
  XAPSecureStatus.value = await invoke("get_secure_status")
}

async function getXAPVersion() {
  XAPVersion.value = await invoke("get_xap_version")
}

</script>

<template>
  <h2>Device Info</h2>
  <button type="button" @click="getDevice()">Show Device Info</button>
  <p>{{ XAPDevice }}</p>

  <button type="button" @click="getSecureStatus()">Get Secure Status</button>
  <p>{{ XAPSecureStatus }}</p>

  <button type="button" @click="getXAPVersion()">Get XAP Version</button>
  <p>{{ XAPVersion }}</p>
</template>
