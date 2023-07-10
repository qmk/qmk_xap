<script setup lang="ts">
    import { storeToRefs } from 'pinia'
    import { ref, watch, onMounted } from 'vue'
    import type { Ref } from 'vue'

    import { useXAPDeviceStore } from '@/stores/devices'
    import { KeyPosition, XAPKeyCodeCategory } from '@generated/xap'
    import { commands } from '@generated/xap'
    import { notifyError } from '@/utils/utils'

    type XAPConstants = { keycodes: XAPKeyCodeCategory[] }
    const store = useXAPDeviceStore()
    const { device } = storeToRefs(store)

    const splitter: Ref<number> = ref(15)
    const keycodeTab: Ref<string> = ref('basic')
    const layerTab: Ref<number> = ref(0)
    const selectedKey: Ref<KeyPosition | null> = ref(null)
    const xapConstants: Ref<XAPConstants | null> = ref(null)

    async function set(code: number) {
        if (selectedKey.value) {
            if (!device.value) {
                return
            }
            // attempt to set keycode
            const ok = await commands.keycodeSet(device.value.id, {
                layer: selectedKey.value.layer,
                row: selectedKey.value.row,
                col: selectedKey.value.column,
                keycode: code,
            })
            switch (ok.status) {
                case 'ok':
                    break
                case 'error':
                    notifyError(ok.error)
                    return
            }
            // read-back updated keymap - state handling is done in the backend
            const keymap = await commands.keymapGet(device.value.id)
            switch (keymap.status) {
                case 'ok':
                    device.value.keymap = keymap.data
                    return
                case 'error':
                    notifyError(keymap.error)
                    return
            }
        }
    }

    function selectKey(layer: number, row: number, col: number) {
        selectedKey.value = { layer: layer, row: row, column: col }
    }

    function colorButton(layer: number, row: number, column: number): string {
        if (
            selectedKey.value?.layer == layer &&
            selectedKey.value?.row == row &&
            selectedKey.value?.column == column
        ) {
            return 'grey'
        }
        return 'white'
    }

    watch(device, async () => {
        selectedKey.value = null
    })

    onMounted(async () => {
        xapConstants.value = await commands.xapConstantsGet()
    })
</script>

<template>
    <q-page>
        <!--   Keymap   -->
        <div class="q-pa-md">
            <q-splitter v-model="splitter">
                <template #before>
                    <q-tabs v-model="layerTab" vertical class="text-primary text-center">
                        <h5>Layer</h5>
                        <!-- eslint-disable-next-line vue/valid-v-for -->
                        <q-tab
                            v-for="(layer, index) in device?.keymap"
                            :name="index"
                            :label="index"
                        />
                    </q-tabs>
                </template>

                <template #after>
                    <q-tab-panels
                        v-model="layerTab"
                        swipeable
                        vertical
                        transition-prev="jump-up"
                        transition-next="jump-up"
                    >
                        <!-- eslint-disable-next-line vue/valid-v-for -->
                        <q-tab-panel v-for="(layer, layer_idx) in device?.keymap" :name="layer_idx">
                            <!-- eslint-disable-next-line vue/require-v-for-key -->
                            <div v-for="row in layer" class="row q-gutter-x-md q-ma-md">
                                <!--  TODO create proper Key and Keycode components -->
                                <!-- eslint-disable-next-line vue/valid-v-for -->
                                <q-responsive
                                    v-for="col in row"
                                    class="col"
                                    style="max-width: 3rem"
                                    :ratio="1"
                                >
                                    <q-btn
                                        :color="
                                            colorButton(
                                                col.position.layer,
                                                col.position.row,
                                                col.position.column,
                                            )
                                        "
                                        text-color="black"
                                        :label="col.code.label ?? col.code.key"
                                        square
                                        @click="
                                            () =>
                                                selectKey(
                                                    col.position.layer,
                                                    col.position.row,
                                                    col.position.column,
                                                )
                                        "
                                    />
                                </q-responsive>
                            </div>
                        </q-tab-panel>
                    </q-tab-panels>
                </template>
            </q-splitter>

            <q-separator />

            <!-- Keycodes -->
            <q-splitter v-model="splitter">
                <template #before>
                    <q-tabs v-model="keycodeTab" vertical class="text-primary text-center">
                        <h5>Keycodes</h5>
                        <!-- eslint-disable vue/no-unused-vars -->
                        <q-tab
                            v-for="category in xapConstants?.keycodes"
                            :key="category.name"
                            :label="category.name"
                            :name="category.name"
                        />
                    </q-tabs>
                </template>

                <template #after>
                    <q-tab-panels
                        v-model="keycodeTab"
                        swipeable
                        vertical
                        transition-prev="jump-up"
                        transition-next="jump-up"
                    >
                        <q-tab-panel
                            v-for="category in xapConstants?.keycodes"
                            :key="category.name"
                            :name="category.name"
                            :label="category.name"
                            class="row q-gutter-md"
                        >
                            <div v-for="code in category.codes" :key="code.code" class="col-1">
                                <q-responsive style="max-width: 4rem" :ratio="1">
                                    <q-btn
                                        color="white"
                                        :disable="device?.secure_status != 'Unlocked'"
                                        square
                                        text-color="black"
                                        :label="code.label ?? code.key"
                                        @click="set(code.code!)"
                                    />
                                    <q-tooltip
                                        v-if="device?.secure_status != 'Unlocked'"
                                        icon="block"
                                        class="bg-red"
                                    >
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
