<script setup lang="ts">
    import { storeToRefs } from 'pinia'
    import { onMounted, onUnmounted, watchEffect } from 'vue'
    import { listen, emit, Event, UnlistenFn } from '@tauri-apps/api/event'
    import { Notify, Loading } from 'quasar'

    import { useXapDeviceStore } from '@/stores/devices'
    import { XapEvent } from '@generated/xap'
    import { commands } from '@generated/xap'

    import router from '@/router/routes'

    const store = useXapDeviceStore()
    const { device, devices } = storeToRefs(store)

    let unlistenNewDevice: UnlistenFn
    let unlistenRemoveDevice: UnlistenFn
    let unlistenSecureStatusChanged: UnlistenFn

    onMounted(async () => {
        unlistenNewDevice = await listen('new-device', async (event: Event<XapEvent>) => {
            if (event.payload.kind != 'NewDevice') {
                return
            }
            const { id } = event.payload.data

            const result = await commands.deviceGet(id)
            switch (result.status) {
                case 'ok':
                    console.log('new device with id ' + id + Date.now())
                    if (store.addDevice(result.data)) {
                        Notify.create({
                            message: 'New Device ' + result.data.info.qmk.product_name,
                            icon: 'power',
                        })
                    }
                    break
                case 'error':
                    console.error('error getting device info for device ' + id + ': ' + result.error)
                    break
            }
        })

        unlistenRemoveDevice = await listen('removed-device', (event: Event<XapEvent>) => {
            if (event.payload.kind != 'RemovedDevice') {
                return
            }

            const { id } = event.payload.data
            console.log('removed device with id ' + id)

            Notify.create({
                message:
                    'Removed Device ' + (devices.value.get(id)?.info.qmk.product_name ?? 'Unknown'),
                icon: 'power_off',
            })

            store.removeDevice(id)
        })

        unlistenSecureStatusChanged = await listen(
            'secure-status-changed',
            (event: Event<XapEvent>) => {
                if (event.payload.kind != 'SecureStatusChanged') {
                    return
                }

                const { id, secure_status } = event.payload.data
                console.log('secure status ' + secure_status + ' for device ' + id)
                store.updateSecureStatus(id, secure_status)
            },
        )

        await emit('frontend-loaded')
    })

    onUnmounted(async () => {
        if (unlistenNewDevice) {
            unlistenNewDevice()
        }
        if (unlistenRemoveDevice) {
            unlistenRemoveDevice()
        }
        if (unlistenSecureStatusChanged) unlistenSecureStatusChanged()
    })

    watchEffect(async () => {
        if (device.value == null && devices.value.size == 0) {
            Loading.show({
                message: 'Searching for XAP devices',
            })
        } else if (device.value != null && Loading.isActive) {
            router.push('/')
            Loading.hide()
        }
    })
</script>

<template>
    <router-view></router-view>
</template>
