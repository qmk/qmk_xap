<script setup lang="ts">
    import { storeToRefs } from 'pinia'
    import { ref, watch, onMounted } from 'vue'
    import type { Ref } from 'vue'

    import { useXAPDeviceStore } from '@/stores/devices'
    import { KeyPosition } from '@bindings/KeyPosition'
    import { KeyPositionConfig } from '@bindings/KeyPositionConfig'
    import { XAPKeyInfo } from '@bindings/XAPKeyInfo'
    import { XAPConstants } from '@bindings/XAPConstants'
    import { setKeyCode } from '@/commands/remap'
    import { getKeyMap } from '@/commands/keymap'
    import { notifyError } from '@/utils/utils'
    import { getXapConstants } from '../commands/constants'

    const store = useXAPDeviceStore()
    const { device } = storeToRefs(store)

    const splitter: Ref<number> = ref(15)
    const keycodeTab: Ref<string> = ref('basic')
    const layerTab: Ref<number> = ref(0)

    const selectedKey: Ref<KeyPosition | null> = ref(null)

    const xapConstants: Ref<XAPConstants | null> = ref(null)

    async function set(code: number) {
        if (selectedKey.value) {
            try {
                if (!device.value) {
                    return
                }
                const config: KeyPositionConfig = {
                    layer: selectedKey.value.layer,
                    row: selectedKey.value.row,
                    col: selectedKey.value.col,
                    keycode: code,
                }
                // attempt to set keycode
                await setKeyCode(device.value.id, config)
                // read-back updated keymap - state handling is done in the backend
                device.value.key_info = await getKeyMap(device.value.id)
            } catch (err: unknown) {
                notifyError(err)
            }
        }
    }

    function selectKey(key: XAPKeyInfo) {
        selectedKey.value = {
            layer: key.position.layer,
            row: key.position.row,
            col: key.position.col
        }
    }

    function colorButton(key: XAPKeyInfo): string {
        if (
            selectedKey.value?.layer == key.position.layer &&
            selectedKey.value?.row == key.position.row &&
            selectedKey.value?.col == key.position.col
        ) {
            return 'grey'
        }
        return 'white'
    }

    function keyLabel(key: XAPKeyInfo): string{
        return key.keycode.label ?? key.keycode.key
    }

    watch(device, async () => {
        selectedKey.value = null
    })

    onMounted(async () => {
        try {
            xapConstants.value = await getXapConstants()
            console.log(xapConstants.value)
        } catch (err) {
            notifyError(err)
        }
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
                            v-for="(layer, index) in device?.key_info"
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
                        <q-tab-panel v-for="(layer, layer_idx) in device?.key_info" :name="layer_idx">
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
                                    <!-- TODO proper representation based on x/y/w/h values -->
                                    <q-btn
                                        v-if="col !== null"
                                        :color="colorButton(col)"
                                        text-color="black"
                                        :label="keyLabel(col)"
                                        square
                                        @click="() => selectKey(col)"
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
                                        @click="set(code.code)"
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
