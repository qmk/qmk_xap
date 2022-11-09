<script setup lang="ts">
    import { storeToRefs } from 'pinia'
    import { computed } from 'vue'
    import type { Ref } from 'vue'

    import { secureUnlock, secureLock } from '@/commands/xap'
    import { useXAPDeviceStore } from '@/stores/devices'
    import { XAPDeviceDTO } from '@bindings/XAPDeviceDTO'
    import { XAPSecureStatus } from '@bindings/XAPSecureStatus'

    const store = useXAPDeviceStore()
    const { device, devices } = storeToRefs(store)
    const devicesA: Ref<Array<XAPDeviceDTO>> = computed(() => Array.from(devices.value.values()))

    async function lock() {
        if (device.value) {
            await secureLock(device.value.id)
        }
    }

    async function unlock() {
        if (device.value) {
            await secureUnlock(device.value.id)
        }
    }
</script>

<template>
    <q-layout view="hHh LpR fff">
        <q-header class="bg-primary text-white" height-hint="98">
            <q-toolbar>
                <q-toolbar-title>
                    <q-avatar>
                        <img src="qmk.svg" />
                    </q-avatar>
                    QMK XAP GUI
                </q-toolbar-title>
                <q-tabs align="left">
                    <q-route-tab label="Device" :disable="device == null" to="/device" exact />
                    <q-route-tab
                        label="Keymap"
                        :disable="device?.info.keymap == null"
                        to="/keymap"
                        exact
                    />
                    <q-route-tab
                        v-if="device?.info?.lighting?.rgblight != null"
                        label="RGB"
                        to="/rgb"
                        exact
                    />
                </q-tabs>
            </q-toolbar>
            <div class="bg-white">
                <q-select
                    v-model="device"
                    label="XAP device"
                    :disable="device == null"
                    filled
                    :options="devicesA"
                    :option-label="(device:XAPDeviceDTO) => device?.info.qmk.manufacturer + ' - ' + device?.info.qmk.product_name "
                    emit-value
                />
            </div>
        </q-header>
        <q-page-container>
            <router-view v-if="device != null" />
        </q-page-container>
        <q-page-sticky position="bottom-right" :offset="[24, 24]">
            <q-btn
                v-if="device?.secure_status != 'Unlocked'"
                fab
                icon="lock_open"
                :loading="device?.secure_status as XAPSecureStatus == 'Unlocking'"
                color="secondary"
                text-color="white"
                @click="unlock"
            />
            <q-btn
                v-else
                fab
                :loading="device?.secure_status as XAPSecureStatus == 'Unlocking'"
                color="secondary"
                text-color="white"
                icon="lock"
                @click="lock"
            />
        </q-page-sticky>
    </q-layout>
</template>
