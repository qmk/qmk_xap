import { defineStore } from 'pinia'

import type { RawBroadcastType, XapDeviceState } from '@generated/xap'
import {
    appendCappedBroadcastMessage,
    BroadcastMessage,
    deviceDisplayName,
} from '@/utils/broadcast'

export const useBroadcastStore = defineStore('broadcast-store', {
    state: () => ({
        messages: [] as BroadcastMessage[],
        sourceNames: new Map<string, string>(),
        nextSequence: 0,
    }),
    getters: {
        sourceName: (state) => (deviceId: string) =>
            state.sourceNames.get(deviceId) ?? 'Unknown device',
    },
    actions: {
        rememberDevice(device: XapDeviceState) {
            this.sourceNames.set(device.id, deviceDisplayName(device))
        },
        appendLog(deviceId: string, text: string, receivedAt = Date.now()) {
            this.nextSequence += 1
            this.messages = appendCappedBroadcastMessage(this.messages, {
                sequence: this.nextSequence,
                receivedAt,
                deviceId,
                kind: 'Log',
                text,
            })
        },
        appendRaw(
            deviceId: string,
            kind: RawBroadcastType,
            payload: number[],
            receivedAt = Date.now(),
        ) {
            this.nextSequence += 1
            this.messages = appendCappedBroadcastMessage(this.messages, {
                sequence: this.nextSequence,
                receivedAt,
                deviceId,
                kind,
                payload: [...payload],
            })
        },
        clearMessages() {
            this.messages = []
        },
    },
})
