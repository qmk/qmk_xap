import { queryDevice } from '@/commands/core'
import { KeyPosition } from '@bindings/KeyPosition'
import { XAPKeyInfo } from '@bindings/XAPKeyInfo'

export async function getKeycode(id: string, position: KeyPosition): Promise<number> {
    return await queryDevice('keycode_get', id, position)
}

export async function getKeyMap(id: string): Promise<Array<Array<Array<XAPKeyInfo|null>>>> {
    return await queryDevice('keymap_get', id, null)
}
