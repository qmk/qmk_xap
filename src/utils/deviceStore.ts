import { defineStore } from 'pinia'

import { XapDeviceState } from '@generated/xap'
import { XapSecureStatus } from '@generated/xap'

export const useXapDeviceStore = defineStore('xap-device-store', {
    state: () => {
        return {
            device: null as XapDeviceState | null,
            devices: new Map<string, XapDeviceState>(),
        }
    },
    getters: {},
    actions: {
        addDevice(device: XapDeviceState): boolean {
            if (!this.devices.has(device.id)) {
                this.devices.set(device.id, device)
                if (!this.device) {
                    this.device = device
                }
                return true
            }
            return false
        },
        removeDevice(id: string) {
            if (this.devices.has(id)) {
                this.devices.delete(id)
            }

            if (this.device?.id == id) {
                this.device = this.devices.values().next().value ?? null
            }
        },
        updateSecureStatus(id: string, secure_status: XapSecureStatus) {
            const device = this.devices.get(id)
            if (device) {
                device.secure_status = secure_status
            }
        },
    },
})
