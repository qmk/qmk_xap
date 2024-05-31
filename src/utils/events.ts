import { listen, Event, UnlistenFn } from '@tauri-apps/api/event'
import { XapEvent } from '@generated/xap'
import { eventBus } from '@/utils/eventbus'

let tauriEventListener: UnlistenFn

export function clearListener() {
    if (tauriEventListener) {
        tauriEventListener()
    }
}

export async function addListener() {
    tauriEventListener = await listen('xap', async (event: Event<XapEvent>) => {
        eventBus.emit('xap', event.payload)
    })
}
