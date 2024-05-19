<script setup lang="ts">
    import { storeToRefs } from 'pinia'
    import { commands } from '@generated/xap'
    import { useXapDeviceStore } from '@/stores/devices'
    import { XapDevice } from '@generated/xap'

    const store = useXapDeviceStore()
    const { device, devices } = storeToRefs(store)
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
                    :options="() => Array.from(devices.values())"
                    :option-value="(device: XapDevice) => device.id"
                    :option-label="
                        (device: XapDevice) =>
                            device?.info.qmk.manufacturer + ' - ' + device?.info.qmk.product_name
                    "
                    emit-value
                />
            </div>
        </q-header>
        <q-page-container>
            <router-view v-if="device != null" />
        </q-page-container>
        <q-page-sticky position="bottom-right" :offset="[24, 24]">
            <q-btn
                fab
                :loading="device?.secure_status == 'Unlocking'"
                color="secondary"
                text-color="white"
                :icon="device?.secure_status == 'Unlocked' ? 'lock' : 'lock_open'"
                @click="
                    async () =>
                        device?.secure_status == 'Unlocked'
                            ? commands.xapSecureLock(device!.id)
                            : commands.xapSecureUnlock(device!.id)
                "
            />
        </q-page-sticky>
    </q-layout>
</template>
