import { XapEvent } from '@/generated/xap'
import mitt, { Emitter } from 'mitt'

type XapEvents = { xap: XapEvent }

export const eventBus: Emitter<XapEvents> = mitt<XapEvents>()
