import type { RawBroadcastType, XapDeviceState } from '@generated/xap'

export const BROADCAST_HISTORY_LIMIT = 1000

export type BroadcastTypeFilter = 'All' | 'Log' | 'Raw'

export type BroadcastMessage =
    | {
          sequence: number
          receivedAt: number
          deviceId: string
          kind: 'Log'
          text: string
      }
    | {
          sequence: number
          receivedAt: number
          deviceId: string
          kind: RawBroadcastType
          payload: number[]
      }

export interface BroadcastFilters {
    query: string
    sourceId: string | null
    type: BroadcastTypeFilter
}

export interface HexRow {
    offset: string
    bytes: string
    ascii: string
}

function pad2(value: number): string {
    return String(value).padStart(2, '0')
}

function pad3(value: number): string {
    return String(value).padStart(3, '0')
}

function formatByte(byte: number): string {
    return (byte & 0xff).toString(16).padStart(2, '0').toUpperCase()
}

function printableByte(byte: number): string {
    const value = byte & 0xff
    return value >= 0x20 && value <= 0x7e ? String.fromCharCode(value) : '.'
}

export function formatBroadcastTime(receivedAt: number): string {
    const date = new Date(receivedAt)
    return `${pad2(date.getHours())}:${pad2(date.getMinutes())}:${pad2(date.getSeconds())}.${pad3(date.getMilliseconds())}`
}

export function formatBroadcastDateTime(receivedAt: number): string {
    const date = new Date(receivedAt)
    return `${date.getFullYear()}-${pad2(date.getMonth() + 1)}-${pad2(date.getDate())} ${formatBroadcastTime(receivedAt)}`
}

export function formatConnectionId(id: string): string {
    return id.length > 12 ? `${id.slice(0, 4)}...${id.slice(-4)}` : id
}

export function deviceDisplayName(device: XapDeviceState): string {
    const parts = [device.info?.qmk.manufacturer, device.info?.qmk.product_name].filter(
        (part): part is string => part != null && part.trim().length > 0,
    )
    return parts.join(' - ') || 'Unknown device'
}

export function formatHexRows(payload: number[]): HexRow[] {
    const rows: HexRow[] = []

    for (let offset = 0; offset < payload.length; offset += 16) {
        const chunk = payload.slice(offset, offset + 16)
        const firstGroup = chunk.slice(0, 8).map(formatByte).join(' ')
        const secondGroup = chunk.slice(8, 16).map(formatByte).join(' ')
        rows.push({
            offset: offset.toString(16).padStart(4, '0').toUpperCase(),
            bytes: `${firstGroup.padEnd(23, ' ')}  ${secondGroup}`.padEnd(48, ' '),
            ascii: chunk.map(printableByte).join(''),
        })
    }

    return rows
}

export function appendCappedBroadcastMessage(
    messages: BroadcastMessage[],
    message: BroadcastMessage,
    limit = BROADCAST_HISTORY_LIMIT,
): BroadcastMessage[] {
    if (limit <= 0) {
        return []
    }

    const retained = limit === 1 ? [] : messages.slice(-(limit - 1))
    return [...retained, message]
}

export function filterBroadcastMessages(
    messages: BroadcastMessage[],
    filters: BroadcastFilters,
    sourceName: (deviceId: string) => string,
): BroadcastMessage[] {
    const query = filters.query.trim().toLowerCase()

    return messages.filter((message) => {
        if (filters.sourceId != null && message.deviceId !== filters.sourceId) {
            return false
        }

        if (filters.type === 'Log' && message.kind !== 'Log') {
            return false
        }

        if (filters.type === 'Raw' && message.kind === 'Log') {
            return false
        }

        if (query.length === 0) {
            return true
        }

        const payload =
            message.kind === 'Log'
                ? message.text
                : `${message.payload.map(formatByte).join(' ')} ${message.payload.map(printableByte).join('')}`
        const searchable = `${sourceName(message.deviceId)} ${message.deviceId} ${message.kind} ${payload}`
        return searchable.toLowerCase().includes(query)
    })
}
