<script setup lang="ts">
    import { storeToRefs } from 'pinia'
    import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
    import type { Ref } from 'vue'

    import { useXapDeviceStore } from '@/utils/deviceStore'
    import {
        KeyCode,
        KeycodeTemplate,
        XapConstants,
        XapDeviceState,
    } from '@generated/xap'
    import { commands } from '@/utils/commands'
    import { notifyDeviceLocked, notifyError, notifyInfo } from '@/utils/utils'
    import { keymapKeyFamilyClass, MOD_MASK } from '@/utils/keycodeFamily'
    import KeycodePicker from '@/components/KeycodePicker.vue'

    const store = useXapDeviceStore()
    const { device } = storeToRefs(store) as { device: Ref<XapDeviceState | null> }

    type Clockwise = 0 | 1
    interface EncoderSlot {
        encoder: number
        clockwise: Clockwise
    }
    interface PendingAssignment {
        layer: number
        slot: EncoderSlot
        template: KeycodeTemplate
        origin: 'picker' | 'tap-slot'
    }

    const layerTab: Ref<number> = ref(0)
    const xapConstants: Ref<XapConstants | null> = ref(null)
    // [layer][encoder][clockwise 0=CCW, 1=CW] -> KeyCode
    const encoderKeymap: Ref<KeyCode[][][] | null> = ref(null)
    const selectedSlot: Ref<EncoderSlot | null> = ref(null)
    const pendingAssignment: Ref<PendingAssignment | null> = ref(null)
    const pageRef: Ref<HTMLElement | null> = ref(null)

    const encoderCount = computed(
        () => device.value?.config?.encoder?.rotary?.length ?? 0,
    )
    const layerCount = computed(() => {
        // KeymapInfo and RemapInfo both report layer_count; take whichever exists.
        const km = device.value?.info?.keymap?.layer_count ?? null
        const rm = device.value?.info?.remap?.layer_count ?? null
        return km ?? rm ?? 0
    })
    const pendingTemplate = computed(() => pendingAssignment.value?.template ?? null)
    const pickerDisabled = computed(
        () =>
            device.value == null ||
            device.value.secure_status !== 'Unlocked' ||
            device.value.info?.remap?.set_encoder_keycode_enabled === false,
    )

    function slotsEqual(a: EncoderSlot | null, b: EncoderSlot | null): boolean {
        if (!a || !b) return false
        return a.encoder === b.encoder && a.clockwise === b.clockwise
    }

    function isPendingSlot(layer: number, slot: EncoderSlot): boolean {
        const p = pendingAssignment.value
        return p !== null && p.layer === layer && slotsEqual(p.slot, slot)
    }

    function isSelected(layer: number, slot: EncoderSlot): boolean {
        return layer === layerTab.value && slotsEqual(selectedSlot.value, slot)
    }

    function keycodeForSlot(layer: number, slot: EncoderSlot): KeyCode | null {
        return encoderKeymap.value?.[layer]?.[slot.encoder]?.[slot.clockwise] ?? null
    }

    function slotLabel(layer: number, slot: EncoderSlot): string {
        const code = keycodeForSlot(layer, slot)
        if (!code) return '—'
        return code.label ?? code.key ?? 'unknown'
    }

    async function readSlot(layer: number, encoder: number, clockwise: Clockwise) {
        if (!device.value) return null
        const result = await commands.keymapGetEncoderKeycode(device.value.id, {
            layer,
            encoder,
            clockwise,
        })
        if (result.status === 'error') {
            notifyError(result.error)
            return null
        }
        return await commands.decodeKeycode(result.data)
    }

    async function fetchEncoderKeymap() {
        if (!device.value || layerCount.value === 0 || encoderCount.value === 0) {
            encoderKeymap.value = null
            return
        }

        // Single round-trip: the backend sweeps every (layer, encoder, cw) slot
        // and returns the decoded tensor. Logs total + per-call timing so the
        // encoder fetch shows up alongside the keymap fetch in init profiles.
        const result = await commands.encoderKeymapGet(device.value.id)
        if (result.status === 'error') {
            notifyError(result.error)
            encoderKeymap.value = null
            return
        }
        encoderKeymap.value = result.data
    }

    async function refreshSlot(layer: number, slot: EncoderSlot) {
        const code = await readSlot(layer, slot.encoder, slot.clockwise)
        if (!code || !encoderKeymap.value) return
        encoderKeymap.value[layer][slot.encoder][slot.clockwise] = code
    }

    async function writeSlot(layer: number, slot: EncoderSlot, keycode: number) {
        if (!device.value) return false
        const result = await commands.remappingSetEncoderKeycode(device.value.id, {
            layer,
            encoder: slot.encoder,
            clockwise: slot.clockwise,
            keycode,
        })
        if (result.status === 'error') {
            notifyError(result.error)
            return false
        }
        await refreshSlot(layer, slot)
        return true
    }

    function selectSlot(slot: EncoderSlot) {
        const pending = pendingAssignment.value
        if (
            pending &&
            (pending.layer !== layerTab.value || !slotsEqual(pending.slot, slot))
        ) {
            cancelPending('selected a different slot')
        }
        selectedSlot.value = slot
    }

    function cancelPending(reason?: string) {
        if (!pendingAssignment.value) return
        pendingAssignment.value = null
        notifyInfo(`Setup cancelled: ${reason ?? 'user cancelled'}`)
    }

    async function applyTemplate(template: KeycodeTemplate, layer: number, slot: EncoderSlot) {
        if (!device.value) return
        if (device.value.secure_status !== 'Unlocked') {
            notifyDeviceLocked()
            return
        }
        const enc = await commands.keycodeTemplateEncode(template)
        if (enc.status === 'error') {
            notifyError(enc.error)
            return
        }
        const ok = await writeSlot(layer, slot, enc.data)
        if (!ok) return
        pendingAssignment.value = null
    }

    async function onPickerSelect(code: number) {
        if (!device.value || !selectedSlot.value) {
            notifyInfo('Select an encoder direction first')
            return
        }
        if (device.value.secure_status !== 'Unlocked') {
            notifyDeviceLocked()
            return
        }
        const pending = pendingAssignment.value
        const layer = pending?.layer ?? layerTab.value
        const slot = pending?.slot ?? selectedSlot.value
        if (pending) {
            const tap = code & 0xff
            if (code > 0xff) {
                notifyInfo('Pick a basic key (code <= 0xFF) to complete the tap slot')
                return
            }
            let filled: KeycodeTemplate
            switch (pending.template.kind) {
                case 'ModTap':
                    filled = { ...pending.template, tap_kc: tap }
                    break
                case 'LayerTap':
                    filled = { ...pending.template, tap_kc: tap }
                    break
                case 'Modified':
                    filled = { ...pending.template, base_kc: tap }
                    break
                default:
                    notifyInfo('Pick a modifier from the panel to complete LM')
                    return
            }
            await applyTemplate(filled, layer, slot)
            return
        }
        await writeSlot(layer, slot, code)
    }

    function onStartTemplate(template: KeycodeTemplate, complete: boolean) {
        if (!device.value) {
            notifyInfo('Connect a device first')
            return
        }
        if (!selectedSlot.value) {
            notifyInfo('Select an encoder direction first')
            return
        }
        if (device.value.secure_status !== 'Unlocked') {
            notifyDeviceLocked()
            return
        }
        if (complete) {
            applyTemplate(template, layerTab.value, selectedSlot.value)
            return
        }
        pendingAssignment.value = {
            layer: layerTab.value,
            slot: selectedSlot.value,
            template,
            origin: 'picker',
        }
    }

    function onFillLayerMod(name: string) {
        const mask = MOD_MASK[name]
        const pending = pendingAssignment.value
        if (mask === undefined || !pending || pending.template.kind !== 'LayerMod') return
        const filled: KeycodeTemplate = { ...pending.template, mod_mask: mask }
        applyTemplate(filled, pending.layer, pending.slot)
    }

    function startTapSlotEdit(layer: number, slot: EncoderSlot) {
        if (!device.value) return
        if (device.value.secure_status !== 'Unlocked') {
            notifyDeviceLocked()
            return
        }
        const code = keycodeForSlot(layer, slot)
        const t = code?.template
        if (!t) return
        let incomplete: KeycodeTemplate | null = null
        switch (t.kind) {
            case 'ModTap':
                incomplete = { ...t, tap_kc: null }
                break
            case 'LayerTap':
                incomplete = { ...t, tap_kc: null }
                break
            case 'LayerMod':
                incomplete = { ...t, mod_mask: null }
                break
            default:
                return
        }
        selectedSlot.value = slot
        pendingAssignment.value = {
            layer,
            slot,
            template: incomplete,
            origin: 'tap-slot',
        }
    }

    function onKeyDown(e: KeyboardEvent) {
        if (e.key === 'Escape' && pendingAssignment.value) {
            cancelPending('Esc pressed')
        }
    }

    watch(device, async () => {
        selectedSlot.value = null
        pendingAssignment.value = null
        await fetchEncoderKeymap()
    })

    onMounted(async () => {
        xapConstants.value = await commands.xapConstantsGet()
        window.addEventListener('keydown', onKeyDown)
        await fetchEncoderKeymap()
    })

    onUnmounted(() => {
        window.removeEventListener('keydown', onKeyDown)
    })
</script>

<template>
    <q-page class="encoder-page-root">
        <div ref="pageRef" class="encoder-page">
            <q-toolbar class="encoder-toolbar">
                <q-tabs
                    v-model="layerTab"
                    inline-label
                    outside-arrows
                    align="left"
                    class="text-primary layer-tabs"
                >
                    <q-btn flat label="Layer" />
                    <q-tab
                        v-for="(_, index) in encoderKeymap"
                        :key="index"
                        :name="index"
                        :label="index"
                    />
                </q-tabs>
            </q-toolbar>

            <q-tab-panels v-model="layerTab" class="encoder-panels">
                <q-tab-panel
                    v-for="(layer, layerIdx) in encoderKeymap"
                    :key="layerIdx"
                    :name="layerIdx"
                    class="encoder-tab-panel"
                >
                    <div class="encoder-card-grid">
                        <div
                            v-for="(pair, encIdx) in layer"
                            :key="encIdx"
                            class="encoder-card"
                        >
                            <div class="encoder-card-header">Encoder {{ encIdx }}</div>
                            <button
                                v-for="(code, cwIdx) in pair"
                                :key="cwIdx"
                                class="encoder-slot key-name-button rounded-lg p-2 text-black border-2 ring-4 ring-inset shadow-md"
                                :class="[
                                    keymapKeyFamilyClass(code),
                                    isSelected(layerIdx, { encoder: encIdx, clockwise: (cwIdx as 0 | 1) }) ? 'border-amber-500 ring-amber-300' : 'border-black ring-neutral-300',
                                    isPendingSlot(layerIdx, { encoder: encIdx, clockwise: (cwIdx as 0 | 1) }) ? 'pending-key' : '',
                                ]"
                                @click="selectSlot({ encoder: encIdx, clockwise: (cwIdx as 0 | 1) })"
                                @dblclick="startTapSlotEdit(layerIdx, { encoder: encIdx, clockwise: (cwIdx as 0 | 1) })"
                            >
                                <span class="slot-direction">{{ cwIdx === 0 ? '↺' : '↻' }}</span>
                                <span class="slot-label">{{ slotLabel(layerIdx, { encoder: encIdx, clockwise: (cwIdx as 0 | 1) }) }}</span>
                                <q-tooltip
                                    v-if="code.key"
                                    class="text-xs"
                                    style="white-space: normal; max-width: 18rem"
                                >
                                    <div><b>{{ code.key }}</b></div>
                                    <div v-if="code.description">{{ code.description }}</div>
                                    <div v-if="code.template" class="text-gray-300">
                                        Double-click to re-edit the tap slot.
                                    </div>
                                </q-tooltip>
                            </button>
                        </div>
                    </div>

                    <div v-if="encoderCount === 0" class="empty-hint">
                        This keyboard reports no encoders in its config blob.
                    </div>
                </q-tab-panel>
            </q-tab-panels>

            <KeycodePicker
                :xap-constants="xapConstants"
                :layer-count="layerCount"
                :pending-template="pendingTemplate"
                :disabled="pickerDisabled"
                @select="onPickerSelect"
                @start-template="onStartTemplate"
                @fill-layer-mod="onFillLayerMod"
                @cancel="cancelPending"
            />
        </div>
    </q-page>
</template>

<style scoped>
.encoder-page-root {
    display: flex;
    height: 0;
    min-width: 0;
    overflow: hidden;
}

.encoder-page {
    display: flex;
    flex: 1 1 auto;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    padding: 0 1rem 0.5rem;
}

.encoder-toolbar {
    flex: 0 0 auto;
    min-height: 3rem;
    padding: 0;
    gap: 1rem;
}

.layer-tabs {
    min-width: 0;
    flex: 1 1 auto;
}

.encoder-panels {
    flex: 0 0 auto;
    overflow: visible;
    background: transparent;
}

.encoder-tab-panel {
    overflow: visible;
    padding: 0.75rem 0 1rem;
}

.encoder-card-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
}

.encoder-card {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem;
    border: 1px solid #e5e7eb;
    border-radius: 0.75rem;
    background: #f9fafb;
    min-width: 9rem;
}

.encoder-card-header {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: #6b7280;
    text-align: center;
}

.encoder-slot {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: 0.5rem;
    width: 8rem;
    height: 3.25rem;
    cursor: pointer;
    transition:
        box-shadow 120ms ease,
        transform 120ms ease;
}

.encoder-slot:hover {
    transform: translateY(-0.0625rem);
    box-shadow:
        0 8px 14px -8px rgba(0, 0, 0, 0.45),
        0 4px 8px -6px rgba(0, 0, 0, 0.35);
}

.slot-direction {
    font-size: 1.25rem;
    flex: 0 0 1.5rem;
    text-align: center;
}

.slot-label {
    flex: 1 1 0;
    font-size: 0.8rem;
    line-height: 1.1;
    overflow-wrap: anywhere;
    word-break: break-word;
    text-align: left;
}

.key-name-button {
    overflow: hidden;
    white-space: normal;
}

.empty-hint {
    margin-top: 0.5rem;
    font-size: 0.875rem;
    color: #6b7280;
}

/* Family colour tints (mirrors KeymapView; scoped CSS doesn't cross). */
.family-layer {
    background-color: #eff6ff;
    --tw-ring-color: #93c5fd;
}
.family-modtap {
    background-color: #faf5ff;
    --tw-ring-color: #d8b4fe;
}
.family-layermod {
    background-color: #ecfeff;
    --tw-ring-color: #67e8f9;
}
.family-modified {
    background-color: #fff7ed;
    --tw-ring-color: #fdba74;
}

.pending-key {
    animation: pending-pulse 1.4s ease-in-out infinite;
    border-style: dashed !important;
}

@keyframes pending-pulse {
    0%, 100% { box-shadow: 0 0 0 0 rgba(217, 119, 6, 0.4); }
    50%      { box-shadow: 0 0 0 4px rgba(217, 119, 6, 0.1); }
}
</style>
