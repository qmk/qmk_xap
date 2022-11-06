<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { ref, watch, onMounted } from 'vue'

import { useXAPDeviceStore, XAPDevice } from '@/stores/devices'
import { KeyPositionConfig } from '@bindings/KeyPositionConfig'
import { KeymapInfo } from '@bindings/KeymapInfo'
import { getKeycode } from '@/commands/keymap'
import { notifyError } from '@/utils/utils'

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
  let id: string = currentDevice.value.id

  for (let layer = 0; layer < (keymap_info?.layer_count ?? 0); layer++) {
    result.push([])
    for (let row = 0; row < keymap_info.matrix.rows; row++) {
      result[layer].push([])
      for (let col = 0; col < keymap_info.matrix.cols; col++) {
        try {
          let keycode: number = await getKeycode(id, layer, row, col)
          result[layer][row].push({ layer: layer, row: row, column: col, keycode: keycode })
        } catch (err) {
          notifyError(err)
        }
      }
    }
  }

  keymap.value = result
}

</script>

<template>
  <q-page>
    <div class="q-gutter-md q-pa-md">
      <h5>Keymap</h5>
      <div v-for="(layer, index) in keymap">
        <h5> Layer {{ index }} </h5>
        <div class="row" v-for="row in layer">

          <q-chip v-for="key in row" square size="xl"> {{ key.keycode }} </q-chip>

        </div>
      </div>
    </div>
  </q-page>
</template>
