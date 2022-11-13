import { queryDevice } from '@/commands/core'
import { KeyPositionConfig } from '@bindings/KeyPositionConfig'

export async function setKeyCode(id: string, keyConfig: KeyPositionConfig): Promise<number> {
    return await queryDevice('keycode_set', id, keyConfig)
}
