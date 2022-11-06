import { callBackend } from '@/commands/core'

export async function secureLock(id: string) {
    await callBackend('secure_lock', id)
}

export async function secureUnlock(id: string) {
    await callBackend('secure_unlock', id)
}
