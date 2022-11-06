import { defineStore } from 'pinia'

import { getSecureStatus } from '@/commands/xap'
import { notifyError } from '@/utils/utils'
import { XAPDeviceDTO } from '@bindings/XAPDeviceDTO'

export const useXAPDeviceStore = defineStore('xap-device-store', {
    state: () => {
        return { currentDevice: null as XAPDeviceDTO | null, devices: new Map<string, XAPDeviceDTO>() }
    },
    getters: {},
    actions: {
        addDevice(device: XAPDeviceDTO): Boolean {
            if (!this.devices.has(device.id)) {
                getSecureStatus(device.id).then(
                    (status) => {
                        device.secure_status = status
                    },
                    (err: any) => {
                        notifyError(err)
                        device.secure_status = 'Disabled'
                    }).then(() => {
                        this.devices.set(device.id, device)
                        if (!this.currentDevice) {
                            this.currentDevice = device
                        }
                    })
                return true
            }
            return false
        },
        removeDevice(id: string) {
            if (this.devices.has(id)) {
                this.devices.delete(id)
            }

            if (this.currentDevice?.id == id) {
                this.currentDevice = this.devices.values().next().value ?? null
            }
        },
        updateSecureStatus(id: string, secure_status: XAPSecureStatus) {
            if (this.devices.has(id)) {
                this.devices.get(id).secure_status = secure_status
            }
        }
    },
})
