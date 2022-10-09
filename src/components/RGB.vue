<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
</script>

<template>
  <h2>RGB</h2>
  <v-color-picker mode="rgb"/>
</template>
