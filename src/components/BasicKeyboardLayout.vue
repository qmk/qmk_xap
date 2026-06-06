<script setup lang="ts">
    import { computed } from 'vue'
    import type { StyleValue } from 'vue'
    import type { KeyCode } from '@generated/xap'
    import KeyLabel from '@/components/KeyLabel.vue'

    const props = defineProps<{ codes: KeyCode[]; keySizeRem: number }>()
    const emit = defineEmits<{ select: [code: number] }>()

    const KEY_GAP_REM = 0.25
    const COMPACT_GROUP_GAP_UNITS = 0.25
    const RIGHT_REGION_START_X = 15.5
    const NUMPAD_START_X = 19
    const ANSI_WIDTH_UNITS = 22.5
    const ANSI_HEIGHT_UNITS = 6.25

    interface KeyPos {
        key: string
        x: number
        y: number
        w: number
        h: number
    }

    const ANSI_LAYOUT: KeyPos[] = [
        // Function row (y=0)
        { key: 'KC_ESCAPE', x: 0, y: 0, w: 1, h: 1 },
        { key: 'KC_F1', x: 2, y: 0, w: 1, h: 1 },
        { key: 'KC_F2', x: 3, y: 0, w: 1, h: 1 },
        { key: 'KC_F3', x: 4, y: 0, w: 1, h: 1 },
        { key: 'KC_F4', x: 5, y: 0, w: 1, h: 1 },
        { key: 'KC_F5', x: 6.5, y: 0, w: 1, h: 1 },
        { key: 'KC_F6', x: 7.5, y: 0, w: 1, h: 1 },
        { key: 'KC_F7', x: 8.5, y: 0, w: 1, h: 1 },
        { key: 'KC_F8', x: 9.5, y: 0, w: 1, h: 1 },
        { key: 'KC_F9', x: 11, y: 0, w: 1, h: 1 },
        { key: 'KC_F10', x: 12, y: 0, w: 1, h: 1 },
        { key: 'KC_F11', x: 13, y: 0, w: 1, h: 1 },
        { key: 'KC_F12', x: 14, y: 0, w: 1, h: 1 },
        { key: 'KC_PRINT_SCREEN', x: 15.5, y: 0, w: 1, h: 1 },
        { key: 'KC_SCROLL_LOCK', x: 16.5, y: 0, w: 1, h: 1 },
        { key: 'KC_PAUSE', x: 17.5, y: 0, w: 1, h: 1 },

        // Number row (y=1.5)
        { key: 'KC_GRAVE', x: 0, y: 1.5, w: 1, h: 1 },
        { key: 'KC_1', x: 1, y: 1.5, w: 1, h: 1 },
        { key: 'KC_2', x: 2, y: 1.5, w: 1, h: 1 },
        { key: 'KC_3', x: 3, y: 1.5, w: 1, h: 1 },
        { key: 'KC_4', x: 4, y: 1.5, w: 1, h: 1 },
        { key: 'KC_5', x: 5, y: 1.5, w: 1, h: 1 },
        { key: 'KC_6', x: 6, y: 1.5, w: 1, h: 1 },
        { key: 'KC_7', x: 7, y: 1.5, w: 1, h: 1 },
        { key: 'KC_8', x: 8, y: 1.5, w: 1, h: 1 },
        { key: 'KC_9', x: 9, y: 1.5, w: 1, h: 1 },
        { key: 'KC_0', x: 10, y: 1.5, w: 1, h: 1 },
        { key: 'KC_MINUS', x: 11, y: 1.5, w: 1, h: 1 },
        { key: 'KC_EQUAL', x: 12, y: 1.5, w: 1, h: 1 },
        { key: 'KC_BACKSPACE', x: 13, y: 1.5, w: 2, h: 1 },

        // QWERTY row (y=2.5)
        { key: 'KC_TAB', x: 0, y: 2.5, w: 1.5, h: 1 },
        { key: 'KC_Q', x: 1.5, y: 2.5, w: 1, h: 1 },
        { key: 'KC_W', x: 2.5, y: 2.5, w: 1, h: 1 },
        { key: 'KC_E', x: 3.5, y: 2.5, w: 1, h: 1 },
        { key: 'KC_R', x: 4.5, y: 2.5, w: 1, h: 1 },
        { key: 'KC_T', x: 5.5, y: 2.5, w: 1, h: 1 },
        { key: 'KC_Y', x: 6.5, y: 2.5, w: 1, h: 1 },
        { key: 'KC_U', x: 7.5, y: 2.5, w: 1, h: 1 },
        { key: 'KC_I', x: 8.5, y: 2.5, w: 1, h: 1 },
        { key: 'KC_O', x: 9.5, y: 2.5, w: 1, h: 1 },
        { key: 'KC_P', x: 10.5, y: 2.5, w: 1, h: 1 },
        { key: 'KC_LEFT_BRACKET', x: 11.5, y: 2.5, w: 1, h: 1 },
        { key: 'KC_RIGHT_BRACKET', x: 12.5, y: 2.5, w: 1, h: 1 },
        { key: 'KC_BACKSLASH', x: 13.5, y: 2.5, w: 1.5, h: 1 },

        // Caps row (y=3.5)
        { key: 'KC_CAPS_LOCK', x: 0, y: 3.5, w: 1.75, h: 1 },
        { key: 'KC_A', x: 1.75, y: 3.5, w: 1, h: 1 },
        { key: 'KC_S', x: 2.75, y: 3.5, w: 1, h: 1 },
        { key: 'KC_D', x: 3.75, y: 3.5, w: 1, h: 1 },
        { key: 'KC_F', x: 4.75, y: 3.5, w: 1, h: 1 },
        { key: 'KC_G', x: 5.75, y: 3.5, w: 1, h: 1 },
        { key: 'KC_H', x: 6.75, y: 3.5, w: 1, h: 1 },
        { key: 'KC_J', x: 7.75, y: 3.5, w: 1, h: 1 },
        { key: 'KC_K', x: 8.75, y: 3.5, w: 1, h: 1 },
        { key: 'KC_L', x: 9.75, y: 3.5, w: 1, h: 1 },
        { key: 'KC_SEMICOLON', x: 10.75, y: 3.5, w: 1, h: 1 },
        { key: 'KC_QUOTE', x: 11.75, y: 3.5, w: 1, h: 1 },
        { key: 'KC_ENTER', x: 12.75, y: 3.5, w: 2.25, h: 1 },

        // Shift row (y=4.5)
        { key: 'KC_LEFT_SHIFT', x: 0, y: 4.5, w: 2.25, h: 1 },
        { key: 'KC_Z', x: 2.25, y: 4.5, w: 1, h: 1 },
        { key: 'KC_X', x: 3.25, y: 4.5, w: 1, h: 1 },
        { key: 'KC_C', x: 4.25, y: 4.5, w: 1, h: 1 },
        { key: 'KC_V', x: 5.25, y: 4.5, w: 1, h: 1 },
        { key: 'KC_B', x: 6.25, y: 4.5, w: 1, h: 1 },
        { key: 'KC_N', x: 7.25, y: 4.5, w: 1, h: 1 },
        { key: 'KC_M', x: 8.25, y: 4.5, w: 1, h: 1 },
        { key: 'KC_COMMA', x: 9.25, y: 4.5, w: 1, h: 1 },
        { key: 'KC_DOT', x: 10.25, y: 4.5, w: 1, h: 1 },
        { key: 'KC_SLASH', x: 11.25, y: 4.5, w: 1, h: 1 },
        { key: 'KC_RIGHT_SHIFT', x: 12.25, y: 4.5, w: 2.75, h: 1 },

        // Bottom row (y=5.5)
        { key: 'KC_LEFT_CTRL', x: 0, y: 5.5, w: 1.25, h: 1 },
        { key: 'KC_LEFT_GUI', x: 1.25, y: 5.5, w: 1.25, h: 1 },
        { key: 'KC_LEFT_ALT', x: 2.5, y: 5.5, w: 1.25, h: 1 },
        { key: 'KC_SPACE', x: 3.75, y: 5.5, w: 6.25, h: 1 },
        { key: 'KC_RIGHT_ALT', x: 10, y: 5.5, w: 1.25, h: 1 },
        { key: 'KC_RIGHT_GUI', x: 11.25, y: 5.5, w: 1.25, h: 1 },
        { key: 'KC_APPLICATION', x: 12.5, y: 5.5, w: 1.25, h: 1 },
        { key: 'KC_RIGHT_CTRL', x: 13.75, y: 5.5, w: 1.25, h: 1 },

        // Edit cluster (x offset = 15.5)
        { key: 'KC_INSERT', x: 15.5, y: 1.5, w: 1, h: 1 },
        { key: 'KC_HOME', x: 16.5, y: 1.5, w: 1, h: 1 },
        { key: 'KC_PAGE_UP', x: 17.5, y: 1.5, w: 1, h: 1 },
        { key: 'KC_DELETE', x: 15.5, y: 2.5, w: 1, h: 1 },
        { key: 'KC_END', x: 16.5, y: 2.5, w: 1, h: 1 },
        { key: 'KC_PAGE_DOWN', x: 17.5, y: 2.5, w: 1, h: 1 },

        // Arrow keys
        { key: 'KC_UP', x: 16.5, y: 4.5, w: 1, h: 1 },
        { key: 'KC_LEFT', x: 15.5, y: 5.5, w: 1, h: 1 },
        { key: 'KC_DOWN', x: 16.5, y: 5.5, w: 1, h: 1 },
        { key: 'KC_RIGHT', x: 17.5, y: 5.5, w: 1, h: 1 },

        // Numpad (x offset = 19)
        { key: 'KC_NUM_LOCK', x: 19, y: 1.5, w: 1, h: 1 },
        { key: 'KC_KP_SLASH', x: 20, y: 1.5, w: 1, h: 1 },
        { key: 'KC_KP_ASTERISK', x: 21, y: 1.5, w: 1, h: 1 },
        { key: 'KC_KP_MINUS', x: 22, y: 1.5, w: 1, h: 1 },
        { key: 'KC_KP_7', x: 19, y: 2.5, w: 1, h: 1 },
        { key: 'KC_KP_8', x: 20, y: 2.5, w: 1, h: 1 },
        { key: 'KC_KP_9', x: 21, y: 2.5, w: 1, h: 1 },
        { key: 'KC_KP_PLUS', x: 22, y: 2.5, w: 1, h: 2 },
        { key: 'KC_KP_4', x: 19, y: 3.5, w: 1, h: 1 },
        { key: 'KC_KP_5', x: 20, y: 3.5, w: 1, h: 1 },
        { key: 'KC_KP_6', x: 21, y: 3.5, w: 1, h: 1 },
        { key: 'KC_KP_1', x: 19, y: 4.5, w: 1, h: 1 },
        { key: 'KC_KP_2', x: 20, y: 4.5, w: 1, h: 1 },
        { key: 'KC_KP_3', x: 21, y: 4.5, w: 1, h: 1 },
        { key: 'KC_KP_ENTER', x: 22, y: 4.5, w: 1, h: 2 },
        { key: 'KC_KP_0', x: 19, y: 5.5, w: 2, h: 1 },
        { key: 'KC_KP_DOT', x: 21, y: 5.5, w: 1, h: 1 },
    ]

    const codeMap = computed(() => {
        const m = new Map<string, KeyCode>()
        for (const c of props.codes) m.set(c.key, c)
        return m
    })

    const layoutKeys = computed(() =>
        ANSI_LAYOUT.map((pos) => ({ pos, code: codeMap.value.get(pos.key) ?? null })),
    )
    function compactX(x: number): number {
        let result = x
        if (x >= RIGHT_REGION_START_X) result -= COMPACT_GROUP_GAP_UNITS
        if (x >= NUMPAD_START_X) result -= COMPACT_GROUP_GAP_UNITS
        return result
    }

    const compressedLayoutKeys = computed(() =>
        layoutKeys.value.map(({ pos, code }) => ({
            code,
            pos: {
                ...pos,
                x: compactX(pos.x),
                y: pos.y > 0 ? pos.y - COMPACT_GROUP_GAP_UNITS : pos.y,
            },
        })),
    )
    const unitRem = computed(() => props.keySizeRem + KEY_GAP_REM)
    const ansiKeyboardStyle = computed<StyleValue>(() => ({
        width: `${ANSI_WIDTH_UNITS * unitRem.value}rem`,
        height: `${ANSI_HEIGHT_UNITS * unitRem.value}rem`,
    }))

    function posToStyle(pos: KeyPos): StyleValue {
        return {
            position: 'absolute',
            top: `${pos.y * unitRem.value}rem`,
            left: `${pos.x * unitRem.value}rem`,
            width: `${pos.w * unitRem.value - KEY_GAP_REM}rem`,
            height: `${pos.h * unitRem.value - KEY_GAP_REM}rem`,
        }
    }
</script>

<template>
    <div class="ansi-keyboard-scroll" data-basic-keyboard-scroll>
        <div
            class="ansi-keyboard-canvas relative"
            data-basic-keyboard-canvas
            :style="ansiKeyboardStyle"
        >
            <button
                v-for="{ pos, code } in compressedLayoutKeys"
                :key="pos.key"
                class="key-name-button absolute rounded-lg p-1 border-2 ring-4 ring-inset shadow-md border-black ring-neutral-300 text-black"
                :class="{ 'opacity-30 cursor-default': code === null }"
                :disabled="code === null || code.code === undefined"
                :style="posToStyle(pos)"
                @click="code?.code !== undefined && emit('select', code.code)"
            >
                <KeyLabel
                    class="text-xs leading-tight"
                    :label="code?.label ?? code?.key ?? pos.key"
                />
                <q-tooltip
                    v-if="code"
                    class="text-xs"
                    style="white-space: normal; max-width: 18rem"
                >
                    <div><b>{{ code.key }}</b></div>
                    <div v-if="code.description">{{ code.description }}</div>
                </q-tooltip>
            </button>
        </div>
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

.ansi-keyboard-scroll {
    box-sizing: border-box;
    display: block;
    min-width: 0;
    width: 100%;
    max-width: 100%;
    overflow-x: scroll;
    overflow-y: hidden;
    padding-bottom: 0.625rem;
    overscroll-behavior-x: contain;
    scrollbar-gutter: stable;
}

.ansi-keyboard-scroll::-webkit-scrollbar {
    height: 0.75rem;
}

.ansi-keyboard-scroll::-webkit-scrollbar-track {
    background: #e5e7eb;
    border-radius: 999px;
}

.ansi-keyboard-scroll::-webkit-scrollbar-thumb {
    background: #9ca3af;
    border: 2px solid #e5e7eb;
    border-radius: 999px;
}

.ansi-keyboard-scroll::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
}

.ansi-keyboard-canvas {
    flex: 0 0 auto;
    max-width: none;
}
</style>
