<script setup lang="ts">
import DeviceInfo from "./components/DeviceInfo.vue";
import RGB from "./components/RGB.vue";
</script>

<template>
  <v-sheet class="d-flex">
    <v-sheet>
      <DeviceInfo />
    </v-sheet>
    <v-sheet>
      <RGB />
    </v-sheet>
  </v-sheet>
</template>

<style scoped></style>
