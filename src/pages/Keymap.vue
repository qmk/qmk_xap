<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { invoke } from '@tauri-apps/api/tauri'
import { ref, watch, onMounted } from 'vue'

import { useXAPDeviceStore, XAPDevice } from '@/stores/devices'
import { KeyPositionConfig } from '@bindings/KeyPositionConfig';
import { KeymapInfo } from '../../src-tauri/bindings/KeymapInfo';

const store = useXAPDeviceStore()
const { currentDevice } = storeToRefs(store)

// [
//   [ layer 0
//     [ // row 0
//        { col }, { col }, { col }
//     ],
//     [ // row 1
//        { col }, { col }, { col }
//     ]
//   ],
//   [ layer 1
//     [ // row 0
//        { col }, { col }, { col } 
//     ],
//     [ // row 1
//        { col }, { col }, { col }
//     ]
//   ]
// ]
const keymap: ref<Array<Array<Array<KeyPositionConfig>>>> = ref()

onMounted(async () => {
  await queryKeymap()
})

watch(currentDevice, async (device: XAPDevice) => {
  if (device) {
    await queryKeymap()
  } else {
    keymap.value = []
  }
})

const queryKeymap = async () => {
  var result: Array<Array<Array<KeyPositionConfig>>> = [];
  let keymap_info: KeymapInfo = currentDevice.value.info.keymap

  for (let layer = 0; layer < (keymap_info?.layer_count ?? 0); layer++) {
    result.push([])
    for (let row = 0; row < keymap_info.matrix.rows; row++) {
      result[layer].push([])
      for (let col = 0; col < keymap_info.matrix.cols; col++) {
        let keycode: number = await getKeycode(layer, row, col)
        result[layer][row].push({ layer: layer, row: row, column: col, keycode: keycode })
      }
    }
  }

  keymap.value = result
}

async function getKeycode(layer: number, row: number, col: number): Promise<number> {
  return await invoke('keycode_get', { id: currentDevice.value.id, arg: { layer: layer, row: row, col } })
    .then((keycode: number) => {
      return keycode;
    })
    .catch((error) => console.error(error));
}

</script>

<template>
  <q-page>
    <div class="q-gutter-md q-pa-md">
      <h5>Keymap</h5>
      <div v-for="(layer, index) in keymap">
        <h5> Layer {{ index }} </h5>
        <div class="row" v-for="row in layer">

            <q-chip  v-for="key in row" square size="xl"> {{ key.keycode }} </q-chip>
 
        </div>
      </div>
    </div>
  </q-page>
</template>
