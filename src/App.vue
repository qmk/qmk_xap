<script setup lang="ts">
import DeviceInfo from "./components/DeviceInfo.vue";
import RGB from "./components/RGB.vue";
</script>

<template>
  <v-app>
    <v-navigation-drawer expand-on-hover rail>
      <v-list>
        <v-list-item prepend-avatar="qmk.png" title="QMK XAP"
          subtitle="Rocks!"></v-list-item>
      </v-list>

      <v-divider></v-divider>

      <v-list density="compact" nav>
        <v-list-item prepend-icon="mdi-folder" title="Keyboard" value="keyboard"></v-list-item>
        <v-list-item prepend-icon="mdi-star" title="RGB" value="rgb"></v-list-item>
        <v-list-item prepend-icon="mdi-account-multiple" title="Info" value="info"></v-list-item>
      </v-list>
    </v-navigation-drawer>
    <v-main>
      <!-- <v-container> -->
        <!-- <v-row> -->
          <!-- <v-col cols="2">
            <v-sheet rounded="lg">
              <DeviceInfo />
            </v-sheet>
          </v-col> -->
          <!-- <v-col> -->
              <RGB />
          <!-- </v-col> -->
        <!-- </v-row> -->
      <!-- </v-container> -->
    </v-main>
  </v-app>
</template>

<style scoped>

</style>
