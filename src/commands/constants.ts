import { queryBackend } from '@/commands/core'
import { XAPConstants } from '@bindings/XAPConstants'

export async function getXapConstants(): Promise<XAPConstants> {
    return await queryBackend('xap_constants_get', null)
}
