<script setup lang="ts">
    import { storeToRefs } from 'pinia'

    import { useXAPDeviceStore } from '@/stores/devices'
    import { XAPSecureStatus } from '@generated/xap'
    import { commands } from '@generated/xap'

    const store = useXAPDeviceStore()
    const { device } = storeToRefs(store)

    async function lock() {
        if (device.value) {
            await commands.xapSecureLock(device.value.id)
        }
    }

    async function unlock() {
        if (device.value) {
            await commands.xapSecureUnlock(device.value.id)
        }
    }

    async function bootloader() {
        if (device.value) {
            await commands.qmkJumpToBootloader(device.value.id)
        }
    }

    async function reset() {
        if (device.value) {
            await commands.qmkReinitializeEeprom(device.value.id)
        }
    }
</script>

<template>
    <q-page>
        <div class="q-gutter-md q-pa-md">
            <h5>Device Information</h5>
            <q-field
                v-if="device?.info.qmk.manufacturer != null"
                filled
                label="Manufacturer"
                stack-label
            >
                <template #control>
                    <div class="self-center full-width no-outline" tabindex="0">
                        {{ device?.info.qmk.manufacturer }}
                    </div>
                </template>
            </q-field>
            <q-field
                v-if="device?.info.qmk.product_name != null"
                filled
                label="Product"
                stack-label
            >
                <template #control>
                    <div class="self-center full-width no-outline" tabindex="0">
                        {{ device?.info.qmk.product_name }}
                    </div>
                </template>
            </q-field>
            <q-field filled label="XAP Version" stack-label>
                <template #control>
                    <div class="self-center full-width no-outline" tabindex="0">
                        {{ device?.info.xap.version }}
                    </div>
                </template>
            </q-field>
            <q-field filled label="QMK Version" stack-label>
                <template #control>
                    <div class="self-center full-width no-outline" tabindex="0">
                        {{ device?.info.qmk.version }}
                    </div>
                </template>
            </q-field>
            <q-field
                v-if="device?.info.qmk.hardware_id != null"
                filled
                label="Hardware Id"
                stack-label
            >
                <template #control>
                    <div class="self-center full-width no-outline" tabindex="0">
                        {{ device?.info.qmk.hardware_id }}
                    </div>
                </template>
            </q-field>
            <q-field v-if="device?.info.qmk.config != null" filled label="Config JSON" stack-label>
                <template #control>
                    <div class="self-center full-width no-outline" tabindex="0">
                        {{ device?.info.qmk.config }}
                    </div>
                </template>
            </q-field>
            <h5>Secure Actions</h5>
            <q-field filled label="Secure Status" stack-label>
                <template #control>
                    <div class="self-center full-width no-outline" tabindex="0">
                        {{ device?.secure_status }}
                    </div>
                </template>
            </q-field>
            <div>
                <q-btn
                    v-if="device?.secure_status != 'Unlocked'"
                    class="full-width"
                    :loading="device?.secure_status == 'Unlocking'"
                    color="primary"
                    text-color="white"
                    label="Unlock"
                    @click="unlock"
                />
                <q-btn
                    v-else
                    class="full-width"
                    :loading="(device?.secure_status as XAPSecureStatus) == 'Unlocking'"
                    color="primary"
                    text-color="white"
                    label="Lock"
                    @click="lock"
                />
            </div>
            <div v-if="device?.info.qmk.jump_to_bootloader_enabled">
                <q-btn
                    :disable="(device?.secure_status as XAPSecureStatus) != 'Unlocked'"
                    class="full-width"
                    color="primary"
                    text-color="white"
                    label="Jump to Bootloader"
                    @click="bootloader"
                />
                <q-tooltip v-if="device.secure_status != 'Unlocked'" class="bg-red">
                    Device is locked
                </q-tooltip>
            </div>
            <div v-if="device?.info.qmk.eeprom_reset_enabled">
                <q-btn
                    :disable="(device?.secure_status as XAPSecureStatus) != 'Unlocked'"
                    class="full-width"
                    color="primary"
                    text-color="white"
                    label="Reset EEPROM"
                    @click="reset"
                />
                <q-tooltip v-if="device.secure_status != 'Unlocked'" class="bg-red">
                    Device is locked
                </q-tooltip>
            </div>
        </div>
    </q-page>
</template>
