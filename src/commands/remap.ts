import { queryBackend } from '@/commands/core'
import { KeyPositionConfig } from '@bindings/KeyPositionConfig'

export async function setKeyCode(id: string, keyConfig: KeyPositionConfig): Promise<number> {
    return await queryBackend('keycode_set', id, keyConfig)
}
