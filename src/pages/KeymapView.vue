<script setup lang="ts">
    import { storeToRefs } from 'pinia'
    import { ref, watch, onMounted } from 'vue'
    import type { Ref, StyleValue } from 'vue'

    import { useXapDeviceStore as useXapDeviceStore } from '@/stores/devices'
    import {
        LayoutEntry,
        MappedKeymap,
        Point3D,
        XapConstants,
        XapDeviceState,
    } from '@generated/xap'
    import { commands } from '@generated/xap'
    import { notifyError } from '@/utils/utils'

    const store = useXapDeviceStore()
    const { device } = storeToRefs(store) as { device: Ref<XapDeviceState | null> }

    const keycodeTab: Ref<string> = ref('basic')
    const layerTab: Ref<number> = ref(0)
    const selectedKey: Ref<Point3D | null> = ref(null)
    const selectedLayout: Ref<string | null> = ref(null)
    const xapConstants: Ref<XapConstants | null> = ref(null)
    const keymap: Ref<MappedKeymap | null> = ref(null)

    async function remapKey(code: number) {
        if (!device.value || !selectedLayout.value || !selectedKey.value) {
            return
        }
        // attempt to set keycode
        const ok = await commands.remapKey(device.value.id, {
            layer: selectedKey.value.z,
            row: selectedKey.value.y,
            column: selectedKey.value.x,
            keycode: code,
        })
        switch (ok.status) {
            case 'ok':
                break
            case 'error':
                notifyError(ok.error)
                return
        }

        updateKeymap()
    }

    function applyLayout(layout: LayoutEntry): StyleValue {
        return {
            top: `${layout.y * 5}rem`,
            left: `${layout.x * 5}rem`,
            width: `${layout.w! * 4.5}rem`,
            height: `${layout.h! * 4.5}rem`,
        }
    }

    function applyKeySelection(position: Point3D): string {
        return selectedKey.value?.x == position.x && selectedKey.value?.y == position.y && selectedKey.value?.z == position.z
            ? 'border-amber-500 ring-amber-300 scale-110'
            : 'border-black ring-neutral-300'
    }

    function getLayouts(): string[] {
        if (device.value == null) {
            return []
        }
        return Object.keys(device!.value.config!.layouts)
    }

    function updateKeymap() {
        if (!device.value || !selectedLayout.value) {
            return
        }

        commands.keymapGet(device.value.id, selectedLayout.value).then((result) => {
            switch (result.status) {
                case 'ok':
                    keymap.value = result.data
                    break
                case 'error':
                    notifyError(result.error)
                    break
            }
        })
    }

    watch(device, async () => {
        selectedKey.value = null
        updateKeymap()
    })

    watch(selectedLayout, async () => {
        if (!device.value || !selectedLayout.value) {
            return
        }

        updateKeymap()
    })

    onMounted(async () => {
        xapConstants.value = await commands.xapConstantsGet()

        if (!device.value) {
            return
        }

        let layouts = getLayouts()

        if (layouts.length != 0) {
            selectedLayout.value = layouts[0]
        }
    })
</script>

<template>
    <q-page>
        <div>
            <q-toolbar>
                <!-- Layouts -->
                <q-select
                    v-model="selectedLayout"
                    label="Layout"
                    :disable="device == null"
                    borderless
                    :options="getLayouts()"
                />
                <q-tabs
                    v-model="layerTab"
                    inline-label
                    outside-arrows
                    align="left"
                    class="text-primary"
                >
                    <q-btn flat label="Layer" />
                    <q-tab v-for="(_, index) in keymap?.keys" :name="index" :label="index" />
                </q-tabs>
            </q-toolbar>
            <!--   Keymap   -->
            <q-tab-panels v-model="layerTab">
                <q-tab-panel
                    :style="{ height: `${Math.max(keymap?.size.y ?? 2, 2) * 3}rem` }"
                    v-for="(layer, layer_idx) in keymap?.keys"
                    :name="layer_idx"
                >
                    <div class="relative">
                        <template v-for="row in layer">
                            <button
                                v-for="col in row.filter((col) => col != null)"
                                class="hover:scale-110 truncate rounded-lg p-2 absolute align-middle text-black border-2 ring-4 ring-inset shadow-md"
                                :class="applyKeySelection(col!.key.position)"
                                :style="applyLayout(col!.layout)"
                                @click="() => (selectedKey = col!.key.position)"
                            >
                                <span>{{
                                    col!.key.code.label ?? col!.key.code.key ?? 'unknown'
                                }}</span>
                            </button>
                        </template>
                    </div>
                </q-tab-panel>
            </q-tab-panels>
            <!-- Keycodes -->
            <q-tabs
                v-model="keycodeTab"
                class="text-primary"
                align="left"
                inline-label
                outside-arrows
            >
                <q-tab
                    v-for="category in xapConstants?.keycodes"
                    :key="category.name"
                    :label="category.name"
                    :name="category.name"
                />
            </q-tabs>
            <q-tab-panels v-model="keycodeTab">
                <q-tab-panel
                    v-for="category in xapConstants?.keycodes.sort((a, b) =>
                        a.name.localeCompare(b.name),
                    )"
                    :key="category.name"
                    :name="category.name"
                    :label="category.name"
                    class="row"
                >
                    <button
                        v-for="code in category.codes"
                        :key="code.code"
                        :disabled="device?.secure_status != 'Unlocked'"
                        style="width: 4.5rem; height: 4.5rem"
                        class="mr-2 mb-2 hover:scale-110 truncate rounded-lg p-2 text-black border-2 ring-4 ring-inset shadow-md border-black ring-neutral-300"
                        @click="remapKey(code.code!)"
                    >
                        <span>{{ code.label ?? code.key }}</span>
                        <q-tooltip
                            v-if="device?.secure_status != 'Unlocked'"
                            icon="block"
                            class="bg-red"
                        >
                            Device is locked
                        </q-tooltip>
                    </button>
                </q-tab-panel>
            </q-tab-panels>
        </div>
    </q-page>
</template>
