import { callDevice } from '@/commands/core'

export async function jumpToBootloader(id: string) {
    await callDevice('jump_to_bootloader', id)
}

export async function resetEEPROM(id: string) {
    await callDevice('reset_eeprom', id)
}
