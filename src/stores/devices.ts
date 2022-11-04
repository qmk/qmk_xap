import { defineStore } from 'pinia'

import { XAPDeviceInfo, XAPSecureStatus } from '@generated/XAPDeviceInfo'
import { invoke } from "@tauri-apps/api/tauri"

export interface XAPDevice {
    id: string,
    secure_status: XAPSecureStatus,
    info: XAPDeviceInfo
}

export const useXAPDeviceStore = defineStore('xap-device-store', {
    state: () => {
        return { currentDevice: null as XAPDevice | null, devices: new Map<string, XAPDevice>() }
    },
    getters: {},
    actions: {
        addDevice(id: string, device: XAPDevice): Boolean {
            if (!this.devices.has(id)) {
                getSecureStatus(id).then((secure_status) => {
                    device.secure_status = secure_status
                    this.devices.set(id, device)
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
            if (this.currentDevice?.id == id) {
                this.currentDevice.secure_status = secure_status
            }
        }
    },
})


async function getSecureStatus(id: string): Promise<XAPSecureStatus> {
    return await invoke('secure_status_get', { id: id })
        .then((secure_status: XAPSecureStatus) => { return secure_status })
        .catch((error) => console.error(error))
}
