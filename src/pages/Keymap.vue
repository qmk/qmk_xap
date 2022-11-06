<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { ref, watch } from 'vue'

import { useXAPDeviceStore } from '@/stores/devices'
import { XAPDeviceDTO } from '@bindings/XAPDeviceDTO'
import { KeyPosition } from '@bindings/KeyPosition'
import { KeyPositionConfig } from "@bindings/KeyPositionConfig"
import { setKeyCode } from '@/commands/remap'
import { getKeyMap } from '@/commands/keymap'
import { notifyError } from '@/utils/utils'

const store = useXAPDeviceStore()
const { device }: ref<XAPDeviceDTO> = storeToRefs(store)

const splitter: ref<number> = ref(15)
const keycodeTab: ref<string> = ref('basic')
const layerTab: ref<number> = ref(0)

const selectedKey: ref<KeyPosition> = ref(null)

// TODO - move to backend and generate from Rust struct
interface KeyCodeCategory {
  name: string,
  codes: Array<KeyCode>
}

// TODO - move to backend and generate from Rust struct
interface KeyCode {
  code: number,
  name: string
}

// TODO totally made-up - construction has to happen in the backend
const keyCodes: ref<Array<KeyCodeCategory>> = ref([
  {
    name: 'basic',
    codes: [
      {
        code: 4,
        name: 'A'
      },
      {
        code: 5,
        name: 'B'
      },
    ]
  },
  {
    name: 'quantum',
    codes: [
      {
        code: 4,
        name: 'A'
      },
      {
        code: 5,
        name: 'B'
      },
    ]
  },
]
)

async function set(code: number) {
  if (selectedKey) {
    try {
      const config: KeyPositionConfig = { layer: selectedKey.value.layer, row: selectedKey.value.row, col: selectedKey.value.col, keycode: code, }
      // attempt to set keycode
      await setKeyCode(device.value.id, config)
      // read-back updated keymap - state handling is done in the backend
      device.value.keymap = await getKeyMap(device.value.id)
    } catch (err: any) {
      notifyError(err)
    }
  }
}

function selectKey(layer: number, row: number, col: number) {
  selectedKey.value = { layer: layer, row: row, col: col }
}

function colorButton(layer: number, row: number, col: number): string {
  if (selectedKey.value?.layer == layer && selectedKey.value?.row == row && selectedKey.value?.col == col) {
    return 'grey'
  }
  return 'white'
}


watch(device, async (device: XAPDeviceDTO) => {
  selectedKey.value = null
})

</script>

<template>
  <q-page>
    <!--   Keymap   -->
    <div class="q-pa-md">
      <q-splitter v-model="splitter">
        <template v-slot:before>
          <q-tabs v-model="layerTab" vertical class="text-primary text-center">
            <h5> Layer </h5>
            <q-tab v-for="(layer, index) in device.keymap" :name="index" :label="index" />
          </q-tabs>
        </template>

        <template v-slot:after>
          <q-tab-panels v-model="layerTab" swipeable vertical transition-prev="jump-up" transition-next="jump-up">
            <q-tab-panel v-for="(layer, layerid) in device.keymap" :name="layerid">
              <div class="row q-gutter-x-md q-ma-md" v-for="(row, rowid) in layer">
                <!--  TODO create proper Key and Keycode components -->
                <q-responsive v-for="(key, colid) in row" class="col" style="max-width:3rem" :ratio="1">
                  <q-btn :color="colorButton(layerid, rowid, colid)" text-color="black" :label="key.keycode" square
                    @click="() => selectKey(layerid, rowid, colid)" />
                </q-responsive>
              </div>
            </q-tab-panel>
          </q-tab-panels>
        </template>
      </q-splitter>

      <q-separator />

      <!-- Keycodes -->
      <q-splitter v-model="splitter">
        <template v-slot:before>
          <q-tabs v-model="keycodeTab" vertical class="text-primary text-center">
            <h5> Keycodes </h5>
            <q-tab v-for="category in keyCodes" :name="category.name" :label="category.name" />
          </q-tabs>
        </template>

        <template v-slot:after>
          <q-tab-panels v-model="keycodeTab" swipeable vertical transition-prev="jump-up" transition-next="jump-up">
            <q-tab-panel v-for="category in keyCodes" :name="category.name" :label="category.name">
              <!--  TODO create proper Key and Keycode components -->
              <div class="row q-gutter-x-md q-ma-md">
                <q-responsive v-for="code in category.codes" class="col" style="max-width:3rem" :ratio="1">
                  <q-btn color="white" :disable="device.secure_status != 'Unlocked'" square text-color="black"
                    @click="set(code.code)" :label="code.name" />
                  <q-tooltip v-if="device.secure_status != 'Unlocked'" class="bg-red">
                    Device is locked
                  </q-tooltip>
                </q-responsive>
              </div>
            </q-tab-panel>
          </q-tab-panels>
        </template>

      </q-splitter>
    </div>
  </q-page>
</template>
