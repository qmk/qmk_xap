import { callDevice, queryDevice } from '@/commands/core'
import { RGBMatrixConfig } from '@bindings/RGBMatrixConfig'

export async function saveConfig(id: string) {
    await callDevice('rgbmatrix_config_save', id)
}

export async function getConfig(id: string): Promise<RGBMatrixConfig> {
    return await queryDevice('rgbmatrix_config_get', id, null)
}

export async function setConfig(id: string, config: RGBMatrixConfig) {
    await queryDevice('rgbmatrix_config_set', id, config)
}
