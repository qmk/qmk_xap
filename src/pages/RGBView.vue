<script setup lang="ts">
    import { watch, computed, onMounted, nextTick, ref } from 'vue'
    import type { Ref } from 'vue'
    import { watchPausable } from '@vueuse/core'
    import { storeToRefs } from 'pinia'
    import ColorPicker from '@radial-color-picker/vue-color-picker'

    import { RGBLightConfig } from '@bindings/RGBLightConfig'
    import { useXAPDeviceStore } from '@/stores/devices'
    import { saveConfig, getConfig, setConfig } from '@/commands/lighting/rgblight'
    import { notifyError } from '@/utils/utils'

    const store = useXAPDeviceStore()
    const { device } = storeToRefs(store)

    const config: Ref<RGBLightConfig> = ref({
        enable: 1,
        mode: 1,
        hue: 255,
        sat: 255,
        val: 255,
        speed: 255,
    })

    const hue = computed({
        get() {
            return Math.ceil((config.value.hue / 255) * 360)
        },
        set(h: number) {
            config.value.hue = Math.ceil((h / 360) * 255)
        },
    })

    async function updateHue(h: number) {
        hue.value = h
    }

    onMounted(async () => {
        pause()
        try {
            if (device.value) {
                config.value = await getConfig(device.value.id)
            }
        } catch (err) {
            notifyError(err)
        }
        await nextTick()
        resume()
    })

    watch(device, async (device) => {
        pause()
        try {
            if (device) {
                config.value = await getConfig(device.id)
            }
        } catch (err) {
            notifyError(err)
        }
        await nextTick()
        resume()
    })

    const { stop, pause, resume } = watchPausable(
        config,
        async (newConfig: RGBLightConfig) => {
            try {
                if (device.value) {
                    await setConfig(device.value.id, newConfig)
                }
            } catch (err) {
                notifyError(err)
            }
        },
        { deep: true }
    )

    async function save() {
        try {
            if (device.value) {
                await saveConfig(device.value.id)
            }
        } catch (err) {
            notifyError(err)
        }
    }
</script>

<template>
    <q-page>
        <div class="">
            <div class="row flex-center">
                <div class="col-3 q-pa-md q-ma-md">
                    <color-picker :hue="hue" @input="updateHue" />
                </div>
                <div class="col q-pa-md q-ma-md q-gutter-md">
                    <q-select
                        v-model.number.lazy="config.mode"
                        :options="device?.info?.lighting?.rgblight?.effects ?? []"
                        label="Mode"
                        emit-value
                    />
                    <q-badge> Hue </q-badge>
                    <q-slider
                        v-model.number.lazy="config.hue"
                        :min="0"
                        :max="255"
                        label
                        marker-labels
                        :markers="32"
                    />
                    <q-badge> Saturation </q-badge>
                    <q-slider
                        v-model.number.lazy="config.sat"
                        :min="0"
                        :max="255"
                        label
                        marker-labels
                        :markers="32"
                    />
                    <q-badge> Value </q-badge>
                    <q-slider
                        v-model.number.lazy="config.val"
                        :min="0"
                        :max="255"
                        label
                        marker-labels
                        :markers="32"
                    />
                    <q-badge> Speed </q-badge>
                    <q-slider
                        v-model.number.lazy="config.speed"
                        :min="0"
                        :max="255"
                        label
                        marker-labels
                        :markers="32"
                    />
                    <q-btn-toggle
                        v-model="config.enable"
                        spread
                        no-caps
                        toggle-color="primary"
                        color="white"
                        text-color="black"
                        :options="[
                            { label: 'Enable', value: 1 },
                            { label: 'Disable', value: 0 },
                        ]"
                    />
                    <q-btn color="white" text-color="black" label="Save" @click="save" />
                </div>
            </div>
        </div>
    </q-page>
</template>

<style>
    @import '@radial-color-picker/vue-color-picker/dist/vue-color-picker.min.css';
</style>
