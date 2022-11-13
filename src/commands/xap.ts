import { callDevice, queryDevice } from '@/commands/core'
import { XAPSecureStatus } from '@bindings/XAPSecureStatus'

export async function secureLock(id: string) {
    await callDevice('secure_lock', id)
}

export async function secureUnlock(id: string) {
    await callDevice('secure_unlock', id)
}

export async function getSecureStatus(id: string): Promise<XAPSecureStatus> {
    return await queryDevice('secure_status_get', id, null)
}
