import { callDevice, queryDevice } from '@/commands/core'
import { RGBLightConfig } from '@bindings/RGBLightConfig'

export async function saveConfig(id: string) {
    await callDevice('rgblight_config_save', id)
}

export async function getConfig(id: string): Promise<RGBLightConfig> {
    return await queryDevice('rgblight_config_get', id, null)
}

export async function setConfig(id: string, config: RGBLightConfig) {
    await queryDevice('rgblight_config_set', id, config)
}
