import { callBackend, queryBackend } from '@/commands/core'
import { RGBMatrixConfig } from '@bindings/RGBMatrixConfig'

export async function saveConfig(id: string) {
    await callBackend('rgbmatrix_config_save', id)
}

export async function getConfig(id: string): Promise<RGBMatrixConfig> {
    return await queryBackend('rgbmatrix_config_get', id, null)
}

export async function setConfig(id: string, config: RGBMatrixConfig) {
    await queryBackend('rgbmatrix_config_set', id, config)
}
