import { defineStore } from 'pinia'

import { XAPDeviceInfo } from '@generated/XAPDeviceInfo'

export interface XAPDevice {
    id: string,
    info: XAPDeviceInfo
}

export const useXAPDeviceStore = defineStore('xap-device-store', {
    state: () => {
        return { currentDevice: null as XAPDevice | null, devices: new Map<string, XAPDevice>() }
    },
    getters: {},
    actions: {
        addDevice(id: string, device: XAPDevice) {
            if (!this.devices.has(id)) {
                this.devices.set(id, device)
            }

            if (!this.currentDevice) {
                this.currentDevice = device
            }
        },
        removeDevice(id: string) {
            if (this.devices.has(id)) {
                this.devices.delete(id)
            }

            if (this.currentDevice?.id == id) {
                this.currentDevice = this.devices.values().next().value ?? null
            }
        }
    },
})
