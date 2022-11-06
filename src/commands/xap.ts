import { callBackend, queryBackend } from '@/commands/core'
import { XAPSecureStatus } from '@generated/XAPDeviceInfo'

export async function secureLock(id: string) {
    await callBackend('secure_lock', id)
}

export async function secureUnlock(id: string) {
    await callBackend('secure_unlock', id)
}


export async function getSecureStatus(id: string): Promise<XAPSecureStatus> {
    return await queryBackend('secure_status_get', id, null)
}
