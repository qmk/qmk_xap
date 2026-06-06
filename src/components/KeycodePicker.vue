<script setup lang="ts">
    import { computed, ref } from 'vue'
    import type { Ref, StyleValue } from 'vue'

    import {
        KeycodeTemplate,
        KeycodeViewSubgroup,
        LayerOp,
        SubgroupTemplate,
        XapConstants,
    } from '@generated/xap'
    import {
        MOD_MASK,
        modName,
        tabFamilyClass,
        templateFamilyClass,
    } from '@/utils/keycodeFamily'
    import BasicKeyboardLayout from '@/components/BasicKeyboardLayout.vue'
    import KeyLabel from '@/components/KeyLabel.vue'

    const props = defineProps<{
        xapConstants: XapConstants | null
        layerCount: number
        pendingTemplate: KeycodeTemplate | null
        // Owner can suppress writes while the device is locked / lacks remap;
        // the picker stays visible so users can still browse.
        disabled?: boolean
    }>()

    const emit = defineEmits<{
        select: [code: number]
        startTemplate: [template: KeycodeTemplate, complete: boolean]
        fillLayerMod: [modName: string]
        cancel: [reason?: string]
    }>()

    const keycodeTab: Ref<string> = ref('basic')
    const PICKER_KEY_SIZE_REM = 3.5
    const PICKER_KEY_GAP_REM = 0.5
    const pickerStyle: StyleValue = {
        '--picker-key-size': `${PICKER_KEY_SIZE_REM}rem`,
        '--picker-key-gap': `${PICKER_KEY_GAP_REM}rem`,
    }

    const keycodeTabs = computed(() => props.xapConstants?.keycode_view?.tabs ?? [])

    // Curated mod options for the LM step-2 mini picker.
    const LM_MOD_OPTIONS = [
        'LCTL', 'LSFT', 'LALT', 'LGUI',
        'RCTL', 'RSFT', 'RALT', 'RGUI',
        'LCA', 'LCAG', 'MEH', 'HYPR',
    ]

    interface TemplateButton {
        label: string
        title: string
        template: KeycodeTemplate
        complete: boolean
    }

    function templateLabel(t: KeycodeTemplate): string {
        switch (t.kind) {
            case 'LayerOp':
                return `${t.op}(${t.layer})`
            case 'OneShotMod':
                return `OSM(${modName(t.mod_mask)})`
            case 'LayerTap':
                return t.tap_kc === null ? `LT(${t.layer})` : `LT(${t.layer}, ?)`
            case 'ModTap':
                return `${modName(t.mod_mask)}_T`
            case 'LayerMod':
                return t.mod_mask === null ? `LM(${t.layer})` : `LM(${t.layer}, ?)`
            case 'Modified':
                return modName(t.mod_mask)
        }
    }

    function slotPrompt(t: KeycodeTemplate): string {
        switch (t.kind) {
            case 'ModTap':
            case 'LayerTap':
            case 'Modified':
                return 'pick a basic key to complete'
            case 'LayerMod':
                return 'pick a modifier to complete'
            default:
                return ''
        }
    }

    function expandTemplate(sub: SubgroupTemplate): TemplateButton[] {
        const layerOpButtons = (op: LayerOp, desc: string): TemplateButton[] => {
            const out: TemplateButton[] = []
            for (let i = 0; i < props.layerCount; i++) {
                out.push({
                    label: `${op}(${i})`,
                    title: `${op}(${i}) - ${desc}`,
                    template: { kind: 'LayerOp', op, layer: i },
                    complete: true,
                })
            }
            return out
        }
        switch (sub.kind) {
            case 'MO':
                return layerOpButtons('MO', 'momentarily activate layer while held')
            case 'TG':
                return layerOpButtons('TG', 'toggle layer on press')
            case 'TO':
                return layerOpButtons('TO', 'switch to layer (turn off others above default)')
            case 'DF':
                return layerOpButtons('DF', 'set the default base layer')
            case 'OSL':
                return layerOpButtons('OSL', 'activate layer until the next key is tapped')
            case 'TT':
                return layerOpButtons('TT', 'tap-toggle layer (hold = momentary, tap repeatedly = toggle)')
            case 'PDF':
                return layerOpButtons('PDF', 'persist new default layer to EEPROM')
            case 'LT': {
                const out: TemplateButton[] = []
                for (let i = 0; i < props.layerCount; i++) {
                    out.push({
                        label: `LT(${i})`,
                        title: `LT(${i}, kc): hold for layer ${i}, tap to send kc - pick a basic key next`,
                        template: { kind: 'LayerTap', layer: i, tap_kc: null },
                        complete: false,
                    })
                }
                return out
            }
            case 'LM': {
                const out: TemplateButton[] = []
                for (let i = 0; i < props.layerCount; i++) {
                    out.push({
                        label: `LM(${i})`,
                        title: `LM(${i}, mod): activate layer ${i} while holding mod - pick a modifier next`,
                        template: { kind: 'LayerMod', layer: i, mod_mask: null },
                        complete: false,
                    })
                }
                return out
            }
            case 'MT':
                return sub.mods
                    .map((name): TemplateButton | null => {
                        const mask = MOD_MASK[name]
                        if (mask === undefined) return null
                        return {
                            label: `${name}_T`,
                            title: `${name}_T(kc): hold for ${name}, tap to send kc - pick a basic key next`,
                            template: { kind: 'ModTap', mod_mask: mask, tap_kc: null },
                            complete: false,
                        }
                    })
                    .filter((b): b is TemplateButton => b !== null)
            case 'QK_MODS':
                return sub.mods
                    .map((name): TemplateButton | null => {
                        const mask = MOD_MASK[name]
                        if (mask === undefined) return null
                        return {
                            label: name,
                            title: `${name}(kc): press kc while holding ${name} - pick a basic key next`,
                            template: { kind: 'Modified', mod_mask: mask, base_kc: null },
                            complete: false,
                        }
                    })
                    .filter((b): b is TemplateButton => b !== null)
        }
    }

    function subgroupVisible(sub: KeycodeViewSubgroup): boolean {
        // While pending LM, hide all regular subgroups; only the mod mini-picker is shown.
        if (props.pendingTemplate?.kind === 'LayerMod') return false
        return sub.codes.length > 0 || sub.template !== null
    }

    function onSelectBasic(code: number) {
        if (props.disabled) return
        emit('select', code)
    }

    function onStartTemplate(btn: TemplateButton) {
        if (props.disabled) return
        emit('startTemplate', btn.template, btn.complete)
    }

    function onFillLayerMod(name: string) {
        if (props.disabled) return
        emit('fillLayerMod', name)
    }
</script>

<template>
    <!-- Pending banner -->
    <div
        v-if="pendingTemplate"
        class="pending-banner"
        :class="templateFamilyClass(pendingTemplate.kind)"
    >
        <q-icon name="adjust" size="sm" class="q-mr-sm" />
        <span>
            Setting up <b>{{ templateLabel(pendingTemplate) }}</b>
            - {{ slotPrompt(pendingTemplate) }}
            (Esc or click another key to cancel)
        </span>
        <q-space />
        <q-btn flat dense label="Cancel" @click="emit('cancel', 'user cancelled')" />
    </div>

    <div class="keycode-area" :style="pickerStyle">
        <!-- Layer-Mod mini picker (replaces the regular picker while pending LM) -->
        <div
            v-if="pendingTemplate?.kind === 'LayerMod'"
            class="mod-mini-picker"
        >
            <div class="text-xs font-semibold text-gray-500 mb-2 uppercase tracking-wide">
                Pick a modifier
            </div>
            <div class="keycode-grid">
                <button
                    v-for="mod in LM_MOD_OPTIONS"
                    :key="mod"
                    class="keycode-button key-name-button rounded-lg p-2 text-black border-2 ring-4 ring-inset shadow-md border-black ring-neutral-300 family-layermod"
                    @click="onFillLayerMod(mod)"
                >
                    <KeyLabel class="picker-key-label" :label="mod" />
                </button>
            </div>
        </div>

        <!-- Keycodes (hidden when LM mini-picker is active) -->
        <template v-if="pendingTemplate?.kind !== 'LayerMod'">
            <q-tabs
                v-model="keycodeTab"
                class="text-primary keycode-tabs"
                align="left"
                inline-label
                outside-arrows
                dense
            >
                <q-tab
                    v-for="tab in keycodeTabs"
                    :key="tab.id"
                    :name="tab.id"
                    :class="[
                        { 'fallback-tab': tab.is_fallback },
                        tabFamilyClass(tab.color),
                    ]"
                >
                    <span>{{ tab.label }}</span>
                    <span v-if="tab.is_fallback" class="fallback-marker">(unmapped)</span>
                </q-tab>
            </q-tabs>
            <q-tab-panels v-model="keycodeTab" class="keycode-panels">
                <q-tab-panel
                    v-for="tab in keycodeTabs"
                    :key="tab.id"
                    :name="tab.id"
                    class="keycode-tab-panel"
                >
                    <div
                        v-for="subgroup in tab.subgroups.filter(subgroupVisible)"
                        :key="subgroup.id"
                        class="subgroup w-full"
                        :class="{ 'fallback-subgroup': subgroup.is_fallback }"
                    >
                        <div
                            v-if="subgroup.label || tab.subgroups.length > 1 || subgroup.is_fallback"
                            class="text-xs font-semibold text-gray-500 mb-1 uppercase tracking-wide"
                        >
                            {{ subgroup.label ?? subgroup.id
                            }}<span v-if="subgroup.is_fallback" class="fallback-marker"> (unmapped)</span>
                        </div>

                        <!-- ANSI keyboard subgroup -->
                        <BasicKeyboardLayout
                            v-if="subgroup.render_mode === 'ansi'"
                            :codes="subgroup.codes"
                            :key-size-rem="PICKER_KEY_SIZE_REM"
                            @select="onSelectBasic"
                        />

                        <!-- Parameterised template subgroup (expanded per layer / mod) -->
                        <div
                            v-else-if="subgroup.template"
                            class="keycode-grid"
                        >
                            <button
                                v-for="(btn, idx) in expandTemplate(subgroup.template)"
                                :key="`${subgroup.id}-${idx}`"
                                class="keycode-button key-name-button rounded-lg p-2 text-black border-2 ring-4 ring-inset shadow-md border-black ring-neutral-300"
                                :class="tabFamilyClass(tab.color)"
                                @click="onStartTemplate(btn)"
                            >
                                <KeyLabel class="picker-key-label" :label="btn.label" />
                                <q-tooltip
                                    class="text-xs"
                                    style="white-space: normal; max-width: 18rem"
                                >
                                    <div><b>{{ btn.label }}</b></div>
                                    <div>{{ btn.title }}</div>
                                </q-tooltip>
                            </button>
                            <div
                                v-if="expandTemplate(subgroup.template).length === 0"
                                class="text-xs text-gray-500"
                            >
                                Connect a device with at least one layer to see {{ subgroup.label ?? subgroup.id }} options.
                            </div>
                        </div>

                        <!-- Regular keycode grid -->
                        <div v-else class="keycode-grid">
                            <button
                                v-for="code in subgroup.codes"
                                :key="code.code"
                                class="keycode-button key-name-button rounded-lg p-2 text-black border-2 ring-4 ring-inset shadow-md border-black ring-neutral-300"
                                @click="onSelectBasic(code.code!)"
                            >
                                <KeyLabel class="picker-key-label" :label="code.label ?? code.key" />
                                <q-tooltip
                                    class="text-xs"
                                    style="white-space: normal; max-width: 18rem"
                                >
                                    <div><b>{{ code.key }}</b></div>
                                    <div v-if="code.description">{{ code.description }}</div>
                                </q-tooltip>
                            </button>
                        </div>
                    </div>
                </q-tab-panel>
            </q-tab-panels>
        </template>
    </div>
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

.picker-key-label {
    font-size: 0.75rem;
}

.keycode-button:hover {
    background-color: rgba(0, 0, 0, 0.05);
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
}

.keycode-button {
    width: var(--picker-key-size);
    height: var(--picker-key-size);
    padding: 0.25rem;
}

.keycode-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, var(--picker-key-size));
    gap: var(--picker-key-gap);
    max-width: calc(16 * var(--picker-key-size) + 15 * var(--picker-key-gap));
}

.keycode-tabs {
    flex: 0 0 auto;
    border-top: 1px solid #e5e7eb;
    margin-top: 0.25rem;
}

.keycode-area {
    display: flex;
    flex: 1 1 auto;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
}

.keycode-panels {
    flex: 1 1 auto;
    min-height: 0;
    overflow: hidden;
    background: transparent;
}

.keycode-tab-panel {
    height: 100%;
    box-sizing: border-box;
    min-width: 0;
    max-width: 100%;
    overflow-x: hidden;
    overflow-y: auto;
    padding: 0.75rem 0.25rem 0.25rem 0;
}

.subgroup {
    min-width: 0;
    max-width: 100%;
    margin-top: 0.75rem;
}

.subgroup:first-child {
    margin-top: 0;
}

.fallback-tab {
    border: 1px dashed #d97706;
    border-radius: 0.25rem;
}

.fallback-marker {
    margin-left: 0.25rem;
    color: #d97706;
    font-size: 0.7rem;
    font-style: italic;
}

.fallback-subgroup .fallback-marker {
    color: #d97706;
}

/* Family colour tints. Same classes apply to picker buttons AND keymap keys. */
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

.pending-banner {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    margin: 0.5rem 0 0.75rem;
    padding: 0.5rem 0.75rem;
    border-radius: 0.5rem;
    border: 1px dashed #6b7280;
    font-size: 0.875rem;
}

.mod-mini-picker {
    flex: 1 1 auto;
    min-height: 0;
    overflow-y: auto;
    margin: 0.75rem 0 1rem;
}
</style>
