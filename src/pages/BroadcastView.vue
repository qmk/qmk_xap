<script setup lang="ts">
    import { storeToRefs } from 'pinia'
    import { computed, nextTick, ref, watch } from 'vue'
    import type { Ref } from 'vue'

    import { useBroadcastStore } from '@/utils/broadcastStore'
    import {
        BROADCAST_HISTORY_LIMIT,
        BroadcastTypeFilter,
        filterBroadcastMessages,
        formatBroadcastDateTime,
        formatBroadcastTime,
        formatConnectionId,
        formatHexRows,
    } from '@/utils/broadcast'

    const store = useBroadcastStore()
    const { messages, sourceNames } = storeToRefs(store)

    const query = ref('')
    const sourceId: Ref<string | null> = ref(null)
    const typeFilter: Ref<BroadcastTypeFilter> = ref('All')
    const autoScroll = ref(true)
    const feedRef: Ref<HTMLElement | null> = ref(null)

    const sourceOptions = computed(() => [
        { label: 'Source: All devices', value: null },
        ...Array.from(sourceNames.value.entries())
            .sort(([, left], [, right]) => left.localeCompare(right))
            .map(([value, label]) => ({
                label: `Source: ${label} (${formatConnectionId(value)})`,
                value,
            })),
    ])

    const visibleMessages = computed(() =>
        filterBroadcastMessages(
            messages.value,
            {
                query: query.value,
                sourceId: sourceId.value,
                type: typeFilter.value,
            },
            store.sourceName,
        ),
    )

    const statusMessage = computed(() => {
        if (visibleMessages.value.length === messages.value.length) {
            return `Showing ${visibleMessages.value.length} of up to ${BROADCAST_HISTORY_LIMIT.toLocaleString()} session messages`
        }

        return `Showing ${visibleMessages.value.length} of ${messages.value.length} captured messages (up to ${BROADCAST_HISTORY_LIMIT.toLocaleString()} retained)`
    })

    const lastVisibleSequence = computed(
        () => visibleMessages.value[visibleMessages.value.length - 1]?.sequence,
    )

    watch(lastVisibleSequence, async () => {
        if (!autoScroll.value) {
            return
        }

        await nextTick()
        if (feedRef.value) {
            feedRef.value.scrollTop = feedRef.value.scrollHeight
        }
    })

    function badgeLabel(kind: 'Log' | 'Keyboard' | 'User'): string {
        return kind === 'Log' ? 'LOG' : `${kind.toUpperCase()} / RAW`
    }
</script>

<template>
    <q-page class="broadcast-page-root">
        <section class="broadcast-page">
            <header class="broadcast-header">
                <div class="page-heading">
                    <h5>Broadcast Messages</h5>
                    <span class="listening-status">
                        <span class="listening-dot"></span>
                        Listening
                    </span>
                </div>

                <div class="broadcast-controls">
                    <q-input
                        v-model="query"
                        class="message-filter"
                        dense
                        outlined
                        clearable
                        placeholder="Filter messages"
                    >
                        <template #prepend>
                            <q-icon name="search" />
                        </template>
                    </q-input>
                    <q-select
                        v-model="sourceId"
                        class="source-filter"
                        dense
                        outlined
                        emit-value
                        map-options
                        :options="sourceOptions"
                    />
                    <q-btn-toggle
                        v-model="typeFilter"
                        class="type-filter"
                        dense
                        no-caps
                        unelevated
                        toggle-color="primary"
                        text-color="grey-8"
                        :options="[
                            { label: 'All', value: 'All' },
                            { label: 'Log', value: 'Log' },
                            { label: 'Raw', value: 'Raw' },
                        ]"
                    />
                    <q-btn
                        class="clear-button"
                        outline
                        no-caps
                        color="grey-7"
                        label="Clear"
                        @click="store.clearMessages()"
                    />
                </div>
            </header>

            <q-card flat bordered class="console-panel">
                <div ref="feedRef" class="message-feed">
                    <div v-if="visibleMessages.length === 0" class="empty-feed">
                        <q-icon name="sensors" size="2rem" color="grey-5" />
                        <div>No broadcast messages captured.</div>
                        <div class="empty-detail">
                            Log, keyboard, and user broadcasts will appear here.
                        </div>
                    </div>

                    <article
                        v-for="message in visibleMessages"
                        :key="message.sequence"
                        class="message-block"
                        :class="`message-${message.kind.toLowerCase()}`"
                    >
                        <div class="message-meta">
                            <time
                                class="message-time"
                                :title="formatBroadcastDateTime(message.receivedAt)"
                            >
                                {{ formatBroadcastTime(message.receivedAt) }}
                            </time>
                            <span class="message-badge">{{ badgeLabel(message.kind) }}</span>
                            <span class="source-name">{{
                                store.sourceName(message.deviceId)
                            }}</span>
                            <code class="source-id">{{
                                formatConnectionId(message.deviceId)
                            }}</code>
                        </div>

                        <p v-if="message.kind === 'Log'" class="log-payload">{{ message.text }}</p>
                        <div v-else class="raw-payload">
                            <div v-if="message.payload.length === 0" class="empty-payload">
                                Empty payload
                            </div>
                            <div
                                v-for="row in formatHexRows(message.payload)"
                                :key="row.offset"
                                class="hex-row"
                            >
                                <span class="hex-offset">{{ row.offset }}</span>
                                <span class="hex-bytes">{{ row.bytes }}</span>
                                <span class="hex-ascii">{{ row.ascii }}</span>
                            </div>
                        </div>
                    </article>
                </div>

                <q-separator />
                <footer class="console-footer">
                    <span>{{ statusMessage }}</span>
                    <q-icon class="history-info" name="info_outline" size="1.1rem">
                        <q-tooltip>
                            Session history is currently limited to 1,000 messages.
                        </q-tooltip>
                    </q-icon>
                    <q-space />
                    <q-toggle v-model="autoScroll" dense color="primary" label="Auto-scroll" />
                </footer>
            </q-card>
        </section>
    </q-page>
</template>

<style scoped>
    .broadcast-page-root {
        display: flex;
        height: 0;
        min-width: 0;
        overflow: hidden;
        background: #f7f8fb;
    }

    .broadcast-page {
        display: flex;
        flex: 1 1 auto;
        flex-direction: column;
        gap: 1rem;
        min-width: 0;
        min-height: 0;
        padding: 1.25rem 1.5rem 1rem;
    }

    .broadcast-header {
        display: flex;
        flex: 0 0 auto;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: 1rem;
    }

    .page-heading {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .page-heading h5 {
        margin: 0;
        font-weight: 500;
    }

    .listening-status {
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
        padding: 0.2rem 0.65rem;
        border: 1px solid #bbf7d0;
        border-radius: 999px;
        color: #15803d;
        background: #f0fdf4;
        font-size: 0.8rem;
        font-weight: 500;
    }

    .listening-dot {
        width: 0.5rem;
        height: 0.5rem;
        border-radius: 50%;
        background: #22c55e;
    }

    .broadcast-controls {
        display: flex;
        align-items: center;
        flex-wrap: wrap;
        justify-content: flex-end;
        gap: 0.6rem;
    }

    .message-filter {
        width: 15rem;
    }

    .source-filter {
        width: min(18rem, 100vw);
    }

    .type-filter {
        border: 1px solid #d1d5db;
        border-radius: 0.3rem;
    }

    .clear-button {
        min-height: 2.5rem;
    }

    .console-panel {
        display: flex;
        flex: 1 1 auto;
        flex-direction: column;
        min-height: 0;
        min-width: 0;
        overflow: hidden;
        border-radius: 0.6rem;
        background: #fff;
    }

    .message-feed {
        display: flex;
        flex: 1 1 auto;
        flex-direction: column;
        gap: 0.75rem;
        min-height: 18rem;
        overflow-y: auto;
        padding: 1rem;
    }

    .empty-feed {
        display: flex;
        flex: 1 1 auto;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 0.4rem;
        color: #6b7280;
    }

    .empty-detail {
        color: #9ca3af;
        font-size: 0.85rem;
    }

    .message-block {
        flex: 0 0 auto;
        border: 1px solid #e5e7eb;
        border-left: 3px solid #1976d2;
        border-radius: 0.45rem;
        padding: 0.65rem 0.85rem 0.75rem;
        background: #fff;
    }

    .message-user {
        border-left-color: #26a69a;
    }

    .message-keyboard {
        border-left-color: #9c27b0;
    }

    .message-meta {
        display: flex;
        align-items: center;
        flex-wrap: wrap;
        gap: 0.55rem;
        color: #6b7280;
        font-size: 0.8rem;
    }

    .message-time,
    .source-id,
    .raw-payload {
        font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    }

    .message-time {
        color: #6b7280;
    }

    .message-badge {
        padding: 0.12rem 0.45rem;
        border-radius: 0.3rem;
        color: #075985;
        background: #e0f2fe;
        font-weight: 600;
        font-size: 0.68rem;
        letter-spacing: 0.03em;
    }

    .message-user .message-badge {
        color: #0f766e;
        background: #ccfbf1;
    }

    .message-keyboard .message-badge {
        color: #7e22ce;
        background: #f3e8ff;
    }

    .source-name {
        padding: 0.15rem 0.5rem;
        border-radius: 999px;
        background: #f3f4f6;
        color: #374151;
    }

    .source-id {
        color: #6b7280;
        font-size: 0.75rem;
    }

    .log-payload {
        margin: 0.6rem 0 0;
        color: #1f2937;
        font-size: 0.95rem;
    }

    .raw-payload {
        overflow-x: auto;
        margin-top: 0.6rem;
        padding: 0.55rem 0.7rem;
        border-radius: 0.35rem;
        color: #374151;
        background: #f8fafc;
        font-size: 0.78rem;
        line-height: 1.5;
    }

    .hex-row {
        display: flex;
        width: max-content;
        white-space: pre;
    }

    .empty-payload {
        color: #94a3b8;
        font-style: italic;
    }

    .hex-offset {
        flex: 0 0 3.25rem;
        color: #9ca3af;
    }

    .hex-bytes {
        flex: 0 0 auto;
        color: #334155;
    }

    .hex-ascii {
        margin-left: 1.1rem;
        padding-left: 1rem;
        border-left: 1px solid #e5e7eb;
        color: #64748b;
    }

    .console-footer {
        display: flex;
        align-items: center;
        gap: 0.45rem;
        padding: 0.55rem 1rem;
        color: #6b7280;
        font-size: 0.78rem;
    }

    .history-info {
        color: #9ca3af;
        cursor: help;
    }

    @media (max-width: 900px) {
        .broadcast-page {
            padding: 1rem;
        }

        .broadcast-controls,
        .message-filter,
        .source-filter {
            width: 100%;
        }
    }
</style>
