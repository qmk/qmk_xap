<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const color: string = ref("hello world!")
const items: Array<string> = ref(["a", "b", "c"])

onMounted(() => {
  console.log(`The initial color is ${color.value}.`)
})

const colorChanged = () => {
  console.log("color changed!")
}


</script>

<template>
  <!-- <h2>RGB</h2> -->
  <v-select :items="items" label="Outlined variant" variant="outlined" />
  <v-color-picker @input="colorChanged" mode="rgb" v-model="color" />
</template>
