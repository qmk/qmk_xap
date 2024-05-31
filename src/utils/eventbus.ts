import { XapEvent } from '@/generated/xap'
import mitt, { Emitter } from 'mitt'

export const eventBus: Emitter<XapEvent> = mitt<XapEvent>()
