import { describe, expect, it } from 'vitest'

import {
    appendCappedBroadcastMessage,
    BroadcastMessage,
    filterBroadcastMessages,
    formatBroadcastDateTime,
    formatBroadcastTime,
    formatHexRows,
} from './broadcast'

function logMessage(sequence: number, deviceId: string, text: string): BroadcastMessage {
    return {
        sequence,
        receivedAt: sequence,
        deviceId,
        kind: 'Log',
        text,
    }
}

describe('broadcast formatting', () => {
    it('formats local receive timestamps with millisecond precision', () => {
        const receivedAt = new Date(2026, 4, 26, 14, 22, 8, 143).getTime()

        expect(formatBroadcastTime(receivedAt)).toBe('14:22:08.143')
        expect(formatBroadcastDateTime(receivedAt)).toBe('2026-05-26 14:22:08.143')
    })

    it('renders raw data as offset hex rows with an ASCII gutter', () => {
        const [row] = formatHexRows([0x01, 0x2a, 0x20, 0x7e, 0xff])

        expect(row).toMatchObject({
            offset: '0000',
            ascii: '.* ~.',
        })
        expect(row.bytes.trimEnd()).toBe('01 2A 20 7E FF')
        expect(row.bytes).toHaveLength(48)
    })
})

describe('broadcast session behavior', () => {
    it('keeps only the latest messages when the history cap is reached', () => {
        let messages: BroadcastMessage[] = []

        messages = appendCappedBroadcastMessage(messages, logMessage(1, 'one', 'first'), 2)
        messages = appendCappedBroadcastMessage(messages, logMessage(2, 'two', 'second'), 2)
        messages = appendCappedBroadcastMessage(messages, logMessage(3, 'three', 'third'), 2)

        expect(messages.map((message) => message.sequence)).toEqual([2, 3])
        expect(appendCappedBroadcastMessage(messages, logMessage(4, 'four', 'fourth'), 1)).toEqual([
            logMessage(4, 'four', 'fourth'),
        ])
    })

    it('filters by source, type, and string or hexadecimal text', () => {
        const messages: BroadcastMessage[] = [
            logMessage(1, 'alpha', 'Layer changed to 2'),
            {
                sequence: 2,
                receivedAt: 2,
                deviceId: 'beta',
                kind: 'User',
                payload: [0x01, 0x2a, 0xff],
            },
        ]
        const sourceName = (deviceId: string) =>
            deviceId === 'alpha' ? 'Mode Designs - SixtyFive' : 'Keebio - Iris'

        expect(
            filterBroadcastMessages(
                messages,
                { query: 'layer', sourceId: null, type: 'Log' },
                sourceName,
            ),
        ).toHaveLength(1)
        expect(
            filterBroadcastMessages(
                messages,
                { query: '01 2a', sourceId: 'beta', type: 'Raw' },
                sourceName,
            ),
        ).toHaveLength(1)
        expect(
            filterBroadcastMessages(
                messages,
                { query: 'iris', sourceId: null, type: 'All' },
                sourceName,
            ),
        ).toEqual([messages[1]])
    })
})
