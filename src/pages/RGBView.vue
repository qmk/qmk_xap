<script setup lang="ts">
    import { watch, computed, onMounted, nextTick, ref } from 'vue'
    import type { Ref } from 'vue'
    import { watchPausable } from '@vueuse/core'
    import { storeToRefs } from 'pinia'
    import ColorPicker from '@radial-color-picker/vue-color-picker'

    import { RgbLightConfig } from '@generated/xap'
    import { useXAPDeviceStore } from '@/stores/devices'
    import { commands } from '@generated/xap'
    import { notifyError } from '@/utils/utils'

    const store = useXAPDeviceStore()
    const { device } = storeToRefs(store)

    const RgbConfig: Ref<RgbLightConfig> = ref({
        enable: 1,
        mode: 1,
        hue: 255,
        sat: 255,
        val: 255,
        speed: 255,
    })

    const hue = computed({
        get() {
            return Math.ceil((RgbConfig.value.hue / 255) * 360)
        },
        set(h: number) {
            RgbConfig.value.hue = Math.ceil((h / 360) * 255)
        },
    })

    async function updateHue(h: number) {
        hue.value = h
    }

    onMounted(async () => {
        pause()
        if (device.value) {
            const config = await commands.rgblightGetConfig(device.value.id)
            switch (config.status) {
                case 'ok':
                    RgbConfig.value = config.data
                    break
                case 'error':
                    notifyError(config.error)
                    return
            }
        }
        await nextTick()
        resume()
    })

    watch(device, async (device) => {
        pause()
        if (device) {
            const config = await commands.rgblightGetConfig(device.id)
            switch (config.status) {
                case 'ok':
                    RgbConfig.value = config.data
                    break
                case 'error':
                    notifyError(config.error)
                    return
            }
        }

        await nextTick()
        resume()
    })

    const { pause, resume } = watchPausable(
        RgbConfig,
        async (newConfig: RgbLightConfig) => {
            if (device.value) {
                await commands.rgblightSetConfig(device.value.id, newConfig)
            }
        },
        { deep: true },
    )

    async function save() {
        try {
            if (device.value) {
                await commands.rgblightSaveConfig(device.value.id)
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
                        v-model.number.lazy="RgbConfig.mode"
                        :options="device?.info?.lighting?.rgblight?.effects ?? []"
                        label="Mode"
                        emit-value
                    />
                    <q-badge> Hue </q-badge>
                    <q-slider
                        v-model.number.lazy="RgbConfig.hue"
                        :min="0"
                        :max="255"
                        label
                        marker-labels
                        :markers="32"
                    />
                    <q-badge> Saturation </q-badge>
                    <q-slider
                        v-model.number.lazy="RgbConfig.sat"
                        :min="0"
                        :max="255"
                        label
                        marker-labels
                        :markers="32"
                    />
                    <q-badge> Value </q-badge>
                    <q-slider
                        v-model.number.lazy="RgbConfig.val"
                        :min="0"
                        :max="255"
                        label
                        marker-labels
                        :markers="32"
                    />
                    <q-badge> Speed </q-badge>
                    <q-slider
                        v-model.number.lazy="RgbConfig.speed"
                        :min="0"
                        :max="255"
                        label
                        marker-labels
                        :markers="32"
                    />
                    <q-btn-toggle
                        v-model="RgbConfig.enable"
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
