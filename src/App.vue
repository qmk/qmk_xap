<script setup lang="ts">
    import { storeToRefs } from 'pinia'
    import { onMounted, onUnmounted, watchEffect } from 'vue'
    import { Notify, Loading } from 'quasar'

    import { addListener, clearListener } from '@/utils/events'
    import { useXapDeviceStore } from '@/utils/deviceStore'
    import router from '@/utils/routes'
    import { eventBus } from '@/utils/eventbus'
    import { XapEvent } from '@generated/xap'
    import { commands } from '@generated/xap'

    const store = useXapDeviceStore()
    const { device, devices } = storeToRefs(store)

    onMounted(async () => {
        addListener()
        eventBus.on('xap', async (event: XapEvent) => {
            switch (event.kind) {
                case 'NewDevice':
                    {
                        const { id } = event.data
                        const result = await commands.deviceGet(id)
                        switch (result.status) {
                            case 'ok':
                                console.log('new device with id ' + id + Date.now())
                                if (store.addDevice(result.data)) {
                                    Notify.create({
                                        message: 'New Device ' + result.data.info?.qmk.product_name,
                                        icon: 'power',
                                    })
                                }
                                break
                            case 'error':
                                console.error(
                                    'error getting device info for device ' +
                                        id +
                                        ': ' +
                                        result.error,
                                )
                                break
                        }
                    }
                    break
                case 'RemovedDevice': {
                    const { id } = event.data
                    Notify.create({
                        message:
                            'Removed Device ' +
                            (devices.value.get(id)?.info?.qmk.product_name ?? 'Unknown'),
                        icon: 'power_off',
                    })
                    store.removeDevice(id)
                    console.log('removed device with id ' + id)
                    break
                }
                case 'SecureStatusChanged': {
                    const { id, secure_status } = event.data
                    console.log('secure status ' + secure_status + ' for device ' + id)
                    store.updateSecureStatus(id, secure_status)
                    break
                }
            }
        })
    })

    onUnmounted(async () => {
        clearListener()
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
