import { defineStore } from 'pinia'

import { XAPDevice } from '@bindings/XAPDevice'
import { XAPSecureStatus } from '@bindings/XAPSecureStatus'

export const useXAPDeviceStore = defineStore('xap-device-store', {
    state: () => {
        return {
            device: null as XAPDevice | null,
            devices: new Map<string, XAPDevice>(),
        }
    },
    getters: {},
    actions: {
        addDevice(device: XAPDevice): boolean {
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
        updateSecureStatus(id: string, secure_status: XAPSecureStatus) {
            const device = this.devices.get(id)
            if (device) {
                device.secure_status = secure_status
            }
        },
    },
})
