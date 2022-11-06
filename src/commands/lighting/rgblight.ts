import { callBackend, queryBackend } from '@/commands/core'
import { RGBConfig } from "@bindings/RGBConfig"

export async function saveConfig(id: string) {
  await callBackend('rgblight_config_save', id)
}

export async function getConfig(id: string): Promise<RGBConfig> {
  return await queryBackend('rgblight_config_get', id)
}

export async function setConfig(id: string, config: RGBConfig) {
  await queryBackend('rgblight_config_set', id, config)
}
