import { callBackend } from '@/commands/core'

export async function jumpToBootloader(id: string) {
    await callBackend('jump_to_bootloader', id)
}

export async function resetEEPROM(id: string) {
    await callBackend('reset_eeprom', id)
}
