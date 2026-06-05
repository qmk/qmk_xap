import type { KeyCode, KeycodeTemplate } from '@generated/xap'

// QMK modifier mask table (qmk_firmware_ref/quantum/modifiers.h).
// Right-side flag is bit 4; bits 0-3 are CTRL/SHIFT/ALT/GUI.
export const MOD_MASK: Record<string, number> = {
    LCTL: 0x01,
    LSFT: 0x02,
    LALT: 0x04,
    LGUI: 0x08,
    RCTL: 0x11,
    RSFT: 0x12,
    RALT: 0x14,
    RGUI: 0x18,
    LCS: 0x03,
    LCA: 0x05,
    LCG: 0x09,
    LSA: 0x06,
    LSG: 0x0a,
    LAG: 0x0c,
    LCAG: 0x0d,
    LCSG: 0x0b,
    LSAG: 0x0e,
    LCSA: 0x07,
    LCSAG: 0x0f,
    MEH: 0x07,
    HYPR: 0x0f,
    RCS: 0x13,
    RCA: 0x15,
    RCG: 0x19,
    RSA: 0x16,
    RSG: 0x1a,
    RAG: 0x1c,
    RCAG: 0x1d,
    RCSG: 0x1b,
    RSAG: 0x1e,
    RCSA: 0x17,
    RCSAG: 0x1f,
}

export function modName(mask: number): string {
    for (const [name, value] of Object.entries(MOD_MASK)) {
        if (value === (mask & 0x1f)) return name
    }
    return `0x${mask.toString(16).toUpperCase()}`
}

export function templateFamilyClass(kind: KeycodeTemplate['kind']): string {
    switch (kind) {
        case 'LayerOp':
        case 'LayerTap':
            return 'family-layer'
        case 'ModTap':
        case 'OneShotMod':
            return 'family-modtap'
        case 'LayerMod':
            return 'family-layermod'
        case 'Modified':
            return 'family-modified'
        default:
            return ''
    }
}

export function keymapKeyFamilyClass(code: KeyCode): string {
    return code.template ? templateFamilyClass(code.template.kind) : ''
}

export function tabFamilyClass(color: string | null | undefined): string {
    switch (color) {
        case 'blue':
            return 'family-layer'
        case 'purple':
            return 'family-modtap'
        case 'cyan':
            return 'family-layermod'
        case 'orange':
            return 'family-modified'
        default:
            return ''
    }
}
