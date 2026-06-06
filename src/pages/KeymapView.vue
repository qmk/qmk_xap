<script setup lang="ts">
    import { storeToRefs } from 'pinia'
    import { ref, watch, onMounted, onUnmounted, computed, nextTick } from 'vue'
    import type { Ref, StyleValue } from 'vue'

    import { useXapDeviceStore } from '@/utils/deviceStore'
    import {
        KeyCode,
        KeycodeTemplate,
        LayoutEntry,
        MappedKeymap,
        Point3D,
        XapConstants,
        XapDeviceState,
    } from '@generated/xap'
    import { commands } from '@/utils/commands'
    import { notifyDeviceLocked, notifyError, notifyInfo } from '@/utils/utils'
    import { fitCurrentWindowToContent } from '@/utils/windowFit'
    import { keymapKeyFamilyClass, MOD_MASK } from '@/utils/keycodeFamily'
    import KeycodePicker from '@/components/KeycodePicker.vue'
    import KeyLabel from '@/components/KeyLabel.vue'

    const store = useXapDeviceStore()
    const { device } = storeToRefs(store) as { device: Ref<XapDeviceState | null> }

    const layerTab: Ref<number> = ref(0)
    const selectedKey: Ref<Point3D | null> = ref(null)
    const selectedLayout: Ref<string | null> = ref(null)
    const xapConstants: Ref<XapConstants | null> = ref(null)
    const keymap: Ref<MappedKeymap | null> = ref(null)
    const keymapPageRef: Ref<HTMLElement | null> = ref(null)

    type PendingOrigin = 'picker' | 'tap-slot'
    interface PendingAssignment {
        position: Point3D
        template: KeycodeTemplate
        origin: PendingOrigin
    }
    const pendingAssignment: Ref<PendingAssignment | null> = ref(null)

    const layerCount = computed(() => keymap.value?.keys.length ?? 0)
    const pendingTemplate = computed(() => pendingAssignment.value?.template ?? null)
    const pickerDisabled = computed(
        () =>
            device.value == null ||
            device.value.secure_status !== 'Unlocked' ||
            device.value.info?.remap?.set_keycode_enabled === false,
    )
    const KEY_UNIT_REM = 4.5
    const KEY_GAP_REM = 0.5
    const KEY_MARGIN_REM = KEY_GAP_REM / 2
    const AUTO_FIT_SIDE_GUTTER_PX = 32
    const AUTO_FIT_BOTTOM_GUTTER_PX = 40
    const AUTO_FIT_MIN_WIDTH_PX = 800
    const AUTO_FIT_MIN_HEIGHT_PX = 600
    const AUTO_FIT_DEBOUNCE_MS = 50
    // When the window is capped to the screen and the ANSI layout in the picker
    // would show less than this fraction, shorten the keymap display (scrollable)
    // to bring the ANSI layout back into view.
    const ANSI_MIN_VISIBLE_RATIO = 0.5
    const KEYMAP_DISPLAY_MIN_PX = 120
    let autoFitTimer: number | undefined
    let displayFitTimer: number | undefined
    let pickerResizeObserver: ResizeObserver | undefined
    const keymapDisplayMaxHeight: Ref<number | null> = ref(null)

    interface KeymapLayoutBounds {
        minX: number
        minY: number
        width: number
        height: number
    }

    function layoutWidth(layout: LayoutEntry): number {
        return layout.w ?? 1
    }

    function layoutHeight(layout: LayoutEntry): number {
        return layout.h ?? 1
    }

    const keymapBounds = computed<KeymapLayoutBounds>(() => {
        const keys = keymap.value?.keys ?? []
        let minX = Number.POSITIVE_INFINITY
        let minY = Number.POSITIVE_INFINITY
        let maxX = Number.NEGATIVE_INFINITY
        let maxY = Number.NEGATIVE_INFINITY

        for (const layer of keys) {
            for (const row of layer) {
                for (const col of row) {
                    if (!col) continue
                    const { layout } = col
                    minX = Math.min(minX, layout.x)
                    minY = Math.min(minY, layout.y)
                    maxX = Math.max(maxX, layout.x + layoutWidth(layout))
                    maxY = Math.max(maxY, layout.y + layoutHeight(layout))
                }
            }
        }

        if (!Number.isFinite(minX) || !Number.isFinite(minY)) {
            return { minX: 0, minY: 0, width: 1, height: 1 }
        }

        return {
            minX,
            minY,
            width: Math.max(maxX - minX, 1),
            height: Math.max(maxY - minY, 1),
        }
    })
    const keymapCanvasStyle = computed<StyleValue>(() => ({
        width: `${keymapBounds.value.width * KEY_UNIT_REM}rem`,
        height: `${keymapBounds.value.height * KEY_UNIT_REM}rem`,
    }))
    const keymapPanelsStyle = computed<StyleValue>(() =>
        keymapDisplayMaxHeight.value != null
            ? { maxHeight: `${keymapDisplayMaxHeight.value}px`, overflowY: 'auto' }
            : {},
    )

    function rootRemPx(): number {
        const fontSize = Number.parseFloat(getComputedStyle(document.documentElement).fontSize)
        return Number.isFinite(fontSize) ? fontSize : 16
    }

    function scheduleWindowFit() {
        if (autoFitTimer !== undefined) {
            window.clearTimeout(autoFitTimer)
        }

        autoFitTimer = window.setTimeout(async () => {
            autoFitTimer = undefined
            await nextTick()
            fitKeymapWindow()
        }, AUTO_FIT_DEBOUNCE_MS)
    }

    async function fitKeymapWindow() {
        if (!device.value || !keymap.value || !keymapPageRef.value) return

        // Size the window against the keymap's natural (uncapped) height; the
        // display cap below is applied afterwards and must not feed back here.
        keymapDisplayMaxHeight.value = null
        await nextTick()

        const keymapWidthPx = keymapBounds.value.width * KEY_UNIT_REM * rootRemPx()
        const contentWidth = keymapWidthPx + AUTO_FIT_SIDE_GUTTER_PX * 2
        const basicKeyboard = keymapPageRef.value.querySelector<HTMLElement>(
            '[data-basic-keyboard-scroll]',
        )
        const bottomElement = basicKeyboard ?? keymapPageRef.value
        const contentBottom = bottomElement.getBoundingClientRect().bottom + window.scrollY
        const contentHeight = contentBottom + AUTO_FIT_BOTTOM_GUTTER_PX

        await fitCurrentWindowToContent({
            contentWidth,
            contentHeight,
            minWidth: AUTO_FIT_MIN_WIDTH_PX,
            minHeight: AUTO_FIT_MIN_HEIGHT_PX,
        })
        ensurePickerObserved()
        scheduleKeymapDisplayFit()
    }

    function scheduleKeymapDisplayFit() {
        if (displayFitTimer !== undefined) window.clearTimeout(displayFitTimer)
        displayFitTimer = window.setTimeout(() => {
            displayFitTimer = undefined
            updateKeymapDisplayFit()
        }, AUTO_FIT_DEBOUNCE_MS)
    }

    // Keep at least ANSI_MIN_VISIBLE_RATIO of the picker's ANSI layout on screen
    // by capping the keymap display height (with a vertical scrollbar) when the
    // window can't show everything. Reacts to viewport and picker-tab changes.
    async function updateKeymapDisplayFit() {
        const page = keymapPageRef.value
        if (!page) return

        keymapDisplayMaxHeight.value = null
        await nextTick()

        const panels = page.querySelector<HTMLElement>('.keymap-panels')
        const ansi = page.querySelector<HTMLElement>('[data-basic-keyboard-scroll]')
        if (!panels || !ansi) return

        const ansiHeight = ansi.scrollHeight
        if (ansiHeight <= 0) return

        const ansiTop = ansi.getBoundingClientRect().top
        const ansiVisible = Math.max(0, Math.min(ansiHeight, window.innerHeight - ansiTop))
        if (ansiVisible >= ansiHeight * ANSI_MIN_VISIBLE_RATIO) return

        const panelsHeight = panels.getBoundingClientRect().height
        const target = Math.max(KEYMAP_DISPLAY_MIN_PX, panelsHeight - (ansiHeight - ansiVisible))
        keymapDisplayMaxHeight.value = Math.round(target)
    }

    function ensurePickerObserved() {
        if (pickerResizeObserver || typeof ResizeObserver === 'undefined') return
        const picker = keymapPageRef.value?.querySelector<HTMLElement>('.keycode-area')
        if (!picker) return
        pickerResizeObserver = new ResizeObserver(() => scheduleKeymapDisplayFit())
        pickerResizeObserver.observe(picker)
    }

    function pointsEqual(a: Point3D | null, b: Point3D | null): boolean {
        if (!a || !b) return false
        return a.x === b.x && a.y === b.y && a.z === b.z
    }

    async function applyTemplate(template: KeycodeTemplate, position: Point3D) {
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
        const remap = await commands.remapKey(device.value.id, {
            layer: Number(position.z),
            row: Number(position.y),
            column: Number(position.x),
            keycode: enc.data,
        })
        if (remap.status === 'error') {
            notifyError(remap.error)
            return
        }
        pendingAssignment.value = null
        updateKeymap()
    }

    function startTemplate(template: KeycodeTemplate, complete: boolean) {
        if (!device.value) {
            notifyInfo('Connect a device first')
            return
        }
        if (!selectedKey.value) {
            notifyInfo('Select a key on the layer first')
            return
        }
        if (device.value.secure_status !== 'Unlocked') {
            notifyDeviceLocked()
            return
        }
        if (complete) {
            applyTemplate(template, selectedKey.value)
            return
        }
        pendingAssignment.value = {
            position: selectedKey.value,
            template,
            origin: 'picker',
        }
    }

    async function remapKey(code: number) {
        if (!device.value || !selectedLayout.value || !selectedKey.value) {
            return
        }
        if (device.value.secure_status !== 'Unlocked') {
            notifyDeviceLocked()
            return
        }
        const pending = pendingAssignment.value
        if (pending) {
            // The picker click is the second step of a parameterised template.
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
                    // LM (mod_mask slot) doesn't take a basic kc.
                    notifyInfo('Pick a modifier from the panel to complete LM')
                    return
            }
            applyTemplate(filled, pending.position)
            return
        }
        const ok = await commands.remapKey(device.value.id, {
            layer: Number(selectedKey.value.z),
            row: Number(selectedKey.value.y),
            column: Number(selectedKey.value.x),
            keycode: code,
        })
        switch (ok.status) {
            case 'ok':
                break
            case 'error':
                notifyError(ok.error)
                return
        }
        updateKeymap()
    }

    function fillLayerMod(name: string) {
        const mask = MOD_MASK[name]
        const pending = pendingAssignment.value
        if (mask === undefined || !pending || pending.template.kind !== 'LayerMod') return
        const filled: KeycodeTemplate = { ...pending.template, mod_mask: mask }
        applyTemplate(filled, pending.position)
    }

    function cancelPending(reason?: string) {
        if (!pendingAssignment.value) return
        pendingAssignment.value = null
        notifyInfo(`Setup cancelled: ${reason ?? 'user cancelled'}`)
    }

    function selectBottomHalf(col: { key: { position: Point3D; code: KeyCode } }) {
        if (!device.value) return
        if (device.value.secure_status !== 'Unlocked') {
            notifyDeviceLocked()
            return
        }
        const t = col.key.code.template
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
        selectedKey.value = col.key.position
        pendingAssignment.value = {
            position: col.key.position,
            template: incomplete,
            origin: 'tap-slot',
        }
    }

    function selectPosition(position: Point3D) {
        if (pendingAssignment.value && !pointsEqual(pendingAssignment.value.position, position)) {
            cancelPending('selected a different key')
        }
        selectedKey.value = position
    }

    function isPendingPosition(position: Point3D): boolean {
        return pointsEqual(pendingAssignment.value?.position ?? null, position)
    }

    function applyLayout(layout: LayoutEntry): StyleValue {
        const bounds = keymapBounds.value
        const top = `${(layout.y - bounds.minY) * KEY_UNIT_REM}rem`
        const left = `${(layout.x - bounds.minX) * KEY_UNIT_REM}rem`
        const width = `${layoutWidth(layout) * KEY_UNIT_REM - KEY_GAP_REM}rem`
        const height = `${layoutHeight(layout) * KEY_UNIT_REM - KEY_GAP_REM}rem`

        return {
            top: top,
            left: left,
            width: width,
            height: height,
            margin: `${KEY_MARGIN_REM}rem`,
        }
    }

    function applyKeySelection(position: Point3D): string {
        return selectedKey.value?.x == position.x && selectedKey.value?.y == position.y && selectedKey.value?.z == position.z
            ? 'border-amber-500 ring-amber-300'
            : 'border-black ring-neutral-300'
    }

    function getLayouts(): string[] {
        if (device.value == null) {
            return []
        }
        return Object.keys(device!.value.config!.layouts)
    }

    function updateKeymap() {
        if (!device.value || !selectedLayout.value) {
            return
        }

        commands.keymapGet(device.value.id, selectedLayout.value).then((result) => {
            switch (result.status) {
                case 'ok':
                    keymap.value = result.data
                    break
                case 'error':
                    notifyError(result.error)
                    break
            }
        })
    }

    function onKeyDown(e: KeyboardEvent) {
        if (e.key === 'Escape' && pendingAssignment.value) {
            cancelPending('Esc pressed')
        }
    }

    watch(device, async () => {
        selectedKey.value = null
        pendingAssignment.value = null
        updateKeymap()
        scheduleWindowFit()
    })

    watch(selectedLayout, async () => {
        if (!device.value || !selectedLayout.value) {
            return
        }

        updateKeymap()
        scheduleWindowFit()
    })

    watch([keymap, pendingAssignment], () => {
        scheduleWindowFit()
    }, { flush: 'post' })

    onMounted(async () => {
        xapConstants.value = await commands.xapConstantsGet()
        window.addEventListener('keydown', onKeyDown)
        window.addEventListener('resize', scheduleKeymapDisplayFit)

        if (!device.value) {
            return
        }

        let layouts = getLayouts()

        if (layouts.length != 0) {
            selectedLayout.value = layouts[0]
        }
    })

    onUnmounted(() => {
        window.removeEventListener('keydown', onKeyDown)
        window.removeEventListener('resize', scheduleKeymapDisplayFit)
        pickerResizeObserver?.disconnect()
        if (autoFitTimer !== undefined) {
            window.clearTimeout(autoFitTimer)
        }
        if (displayFitTimer !== undefined) {
            window.clearTimeout(displayFitTimer)
        }
    })
</script>

<template>
    <q-page class="keymap-page-root">
        <div ref="keymapPageRef" class="keymap-page">
            <q-toolbar class="keymap-toolbar">
                <!-- Layouts -->
                <q-select
                    v-model="selectedLayout"
                    label="Layout"
                    class="layout-select"
                    :disable="device == null"
                    borderless
                    :options="getLayouts()"
                />
                <q-tabs
                    v-model="layerTab"
                    inline-label
                    outside-arrows
                    align="left"
                    class="text-primary layer-tabs"
                >
                    <q-btn flat label="Layer" />
                    <q-tab
                        v-for="(_, index) in keymap?.keys"
                        :key="index"
                        :name="index"
                        :label="index"
                    />
                </q-tabs>
            </q-toolbar>
            <!--   Keymap   -->
            <q-tab-panels v-model="layerTab" class="keymap-panels" :style="keymapPanelsStyle">
                <q-tab-panel
                    v-for="(layer, layer_idx) in keymap?.keys"
                    :key="layer_idx"
                    :name="layer_idx"
                    class="keymap-tab-panel"
                >
                    <div class="keymap-stage">
                        <div
                            class="keymap-canvas relative"
                            data-keymap-canvas
                            :style="keymapCanvasStyle"
                        >
                            <template v-for="row in layer">
                                <button
                                    v-for="col in row.filter((col) => col != null)"
                                    class="key-button key-name-button rounded-lg p-2 absolute align-middle text-black border-2 ring-4 ring-inset shadow-md"
                                    :class="[
                                        applyKeySelection(col!.key.position),
                                        col!.key.code.top && col!.key.code.bottom ? 'split-key' : '',
                                        keymapKeyFamilyClass(col!.key.code),
                                        isPendingPosition(col!.key.position) ? 'pending-key' : '',
                                    ]"
                                    :style="applyLayout(col!.layout)"
                                    @click="() => selectPosition(col!.key.position)"
                                >
                                    <template v-if="col!.key.code.top && col!.key.code.bottom">
                                        <KeyLabel class="split-top" :label="col!.key.code.top" />
                                        <span class="split-divider"></span>
                                        <KeyLabel
                                            class="split-bottom split-bottom-clickable"
                                            :label="col!.key.code.bottom"
                                            @click.stop="selectBottomHalf(col!)"
                                        />
                                    </template>
                                    <KeyLabel
                                        v-else
                                        :label="col!.key.code.label ?? col!.key.code.key ?? 'unknown'"
                                    />
                                    <q-tooltip
                                        v-if="col!.key.code.key"
                                        class="text-xs"
                                        style="white-space: normal; max-width: 18rem"
                                    >
                                        <div><b>{{ col!.key.code.key }}</b></div>
                                        <div v-if="col!.key.code.description">{{ col!.key.code.description }}</div>
                                    </q-tooltip>
                                </button>
                            </template>
                        </div>
                    </div>
                </q-tab-panel>
            </q-tab-panels>

            <KeycodePicker
                :xap-constants="xapConstants"
                :layer-count="layerCount"
                :pending-template="pendingTemplate"
                :disabled="pickerDisabled"
                @select="remapKey"
                @start-template="startTemplate"
                @fill-layer-mod="fillLayerMod"
                @cancel="cancelPending"
            />
        </div>
    </q-page>
</template>

<style scoped>
.key-name-button {
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    text-align: center;
    white-space: normal;
    line-height: 1.1;
}

.keymap-page-root {
    display: flex;
    height: 0;
    min-width: 0;
    overflow: hidden;
}

.keymap-page {
    display: flex;
    flex: 1 1 auto;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    padding: 0 1rem 0.5rem;
}

.keymap-toolbar {
    flex: 0 0 auto;
    min-height: 3rem;
    padding: 0;
    gap: 1rem;
}

.layout-select {
    flex: 0 0 12rem;
    max-width: 16rem;
}

.layer-tabs {
    min-width: 0;
    flex: 1 1 auto;
}

.keymap-panels {
    flex: 0 0 auto;
    overflow: visible;
    background: transparent;
}

.keymap-tab-panel {
    overflow: visible;
    padding: 0.75rem 0 1rem;
}

.keymap-stage {
    display: flex;
    justify-content: center;
    overflow-x: auto;
    overflow-y: visible;
    padding: 0.25rem 0 0.5rem;
}

.keymap-canvas {
    flex: 0 0 auto;
}

.key-button {
    transition:
        box-shadow 120ms ease,
        transform 120ms ease;
    transform-origin: center;
}

.key-button:hover {
    transform: translateY(-0.0625rem);
    box-shadow:
        0 8px 14px -8px rgba(0, 0, 0, 0.45),
        0 4px 8px -6px rgba(0, 0, 0, 0.35);
}

.split-key {
    flex-direction: column;
}

.key-name-button .split-top,
.key-name-button .split-bottom {
    flex: 1 1 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    font-size: 0.7rem;
    line-height: 1;
}

.key-name-button .split-bottom {
    font-size: 0.85rem;
    font-weight: 600;
}

.split-bottom-clickable {
    cursor: pointer;
}

.split-bottom-clickable:hover {
    background-color: rgba(0, 0, 0, 0.06);
}

.split-divider {
    flex: 0 0 auto;
    width: 100%;
    border-top: 1px solid rgba(0, 0, 0, 0.35);
}

/* Family colour tints applied via keymapKeyFamilyClass(). The picker has its
   own copies of these rules (scoped to KeycodePicker.vue) since scoped CSS
   doesn't cross component boundaries. */
.family-layer {
    background-color: #eff6ff;       /* blue-50 */
    --tw-ring-color: #93c5fd;         /* blue-300 */
}
.family-modtap {
    background-color: #faf5ff;       /* purple-50 */
    --tw-ring-color: #d8b4fe;         /* purple-300 */
}
.family-layermod {
    background-color: #ecfeff;       /* cyan-50 */
    --tw-ring-color: #67e8f9;         /* cyan-300 */
}
.family-modified {
    background-color: #fff7ed;       /* orange-50 */
    --tw-ring-color: #fdba74;         /* orange-300 */
}

.pending-key {
    animation: pending-pulse 1.4s ease-in-out infinite;
    border-style: dashed !important;
}

.pending-key .split-bottom {
    font-style: italic;
    color: #6b7280; /* gray-500 */
}

@keyframes pending-pulse {
    0%, 100% { box-shadow: 0 0 0 0 rgba(217, 119, 6, 0.4); }
    50%      { box-shadow: 0 0 0 4px rgba(217, 119, 6, 0.1); }
}

</style>
