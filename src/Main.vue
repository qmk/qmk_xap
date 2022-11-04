<script setup lang="ts">

import { storeToRefs } from 'pinia'
import { onMounted, onUnmounted, watchEffect } from 'vue'
import { listen, emit, Event } from '@tauri-apps/api/event'
import { Notify, Loading } from 'quasar'

import { useXAPDeviceStore, XAPDevice } from '@/stores/devices'
import { FrontendEvent } from '@bindings/FrontendEvent'

const store = useXAPDeviceStore()
const { currentDevice, devices } = storeToRefs(store)

let unlistenNewDevice
let unlistenRemoveDevice
let unlistenSecureStatusChanged

onMounted(async () => {
  unlistenNewDevice = await listen('new-device', (event: Event<FrontendEvent>) => {
    let id = event.payload.id
    let device = event.payload.device
    console.log("new device with id " + id + Date.now())

    let xapDevice: XAPDevice = { id, info: device, secure_status: "Disabled" };
    if (store.addDevice(id, xapDevice)) {
      Notify.create({
        message: 'New Device ' + device.qmk.product_name,
        icon: 'announcement'
      })
    }
  })

  unlistenRemoveDevice = await listen('removed-device', (event: Event<FrontendEvent>) => {
    let id = event.payload.id
    console.log("removed device with id " + id)

    Notify.create({
      message: 'Removed Device ' + (devices.value.get(id)?.info.qmk.product_name ?? 'Unknown'),
      icon: 'announcement'
    })

    store.removeDevice(id)
  })

  unlistenSecureStatusChanged = await listen('secure-status-changed', (event: Event<FrontendEvent>) => {
    let id = event.payload.id
    let secure_status = event.payload.secure_status
    console.log("secure status " + secure_status + " for device " + id)
    store.updateSecureStatus(id, secure_status)
  })

  await emit('frontend-loaded');
})

onUnmounted(async () => {
  if (unlistenNewDevice) {
    unlistenNewDevice()
  }
  if (unlistenRemoveDevice) {
    unlistenRemoveDevice()
  }
  if (unlistenSecureStatusChanged) (
    unlistenSecureStatusChanged()
  )
})

watchEffect(async () => {
  if (currentDevice.value == null && devices.value.size == 0) {
    Loading.show({
      message: 'Searching for XAP devices'
    })
  } else if (currentDevice.value != null && Loading.isActive) {
    Loading.hide()
  }
})
</script>

<template>
  <router-view></router-view>
</template>
