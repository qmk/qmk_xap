<script setup lang="ts">

import DeviceInfo from "./components/DeviceInfo.vue"
import RGB from "./components/RGB.vue"

import { ref } from 'vue'


const leftDrawerOpen = ref(false)

async function toggleLeftDrawer() {
  leftDrawerOpen.value = !leftDrawerOpen.value
}
</script>

<template>
  <q-layout view="hHh LpR fff">

    <q-header elevated class="bg-primary text-white" height-hint="98">
      <q-toolbar>
        <q-btn dense flat round icon="menu" @click="toggleLeftDrawer" />

        <q-toolbar-title>
          <q-avatar>
            <img src="src/assets/qmk.svg">
          </q-avatar>
          QMK XAP GUI
        </q-toolbar-title>
      </q-toolbar>

      <q-tabs align="left">
        <q-route-tab to="/page1" label="Device" />
        <q-route-tab to="/page2" label="Keymap" />
        <q-route-tab to="/page3" label="RGB" />
      </q-tabs>
    </q-header>

    <q-drawer show-if-above v-model="leftDrawerOpen" side="left" bordered>
      <!-- drawer content -->
    </q-drawer>

    <q-page-container>
      <div class="q-pa-md">
        <div class="row">
          <div class="col">
            <DeviceInfo />
          </div>
          <div class="col-6">
            <RGB />
          </div>
        </div>
      </div>

      <router-view />
    </q-page-container>

    <q-footer elevated class="bg-grey-8 text-white">
      <q-toolbar>
        <q-toolbar-title>
          <q-avatar>
            <img src="src/assets/qmk.svg">
          </q-avatar>
        </q-toolbar-title>
      </q-toolbar>
    </q-footer>

  </q-layout>
</template>
