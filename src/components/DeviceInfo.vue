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
  <q-btn @click="getDevice()" label="Show Device Info"/>
  <p>{{ XAPDevice }}</p>

  <q-btn @click="getSecureStatus()" label="Get Secure Status"/>
  <p>{{ XAPSecureStatus }}</p>

  <q-btn @click="getXAPVersion()" label="Get XAP Version"/>
  <p>{{ XAPVersion }}</p>
</template>
