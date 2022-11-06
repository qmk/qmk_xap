<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { ref, watch, onMounted } from 'vue'

import { useXAPDeviceStore } from '@/stores/devices'
import { XAPDeviceDTO } from '@bindings/XAPDeviceDTO'

import { notifyError } from '@/utils/utils'

const store = useXAPDeviceStore()
const { device }: ref<XAPDeviceDTO> = storeToRefs(store)
</script>

<template>
  <q-page>
    <div class="q-gutter-md q-pa-md">
      <h5>Keymap</h5>
      <div v-for="(layer, index) in device.keymap">
        <h5> Layer {{ index }} </h5>
        <div class="row" v-for="row in layer">
          <q-chip v-for="key in row" square size="xl"> {{ key.keycode }} </q-chip>
        </div>
      </div>

    </div>

  </q-page>
</template>
