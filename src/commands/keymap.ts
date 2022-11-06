import { queryBackend } from "@/commands/core";
import { KeyPosition } from "@bindings/KeyPosition";
import { KeyPositionConfig } from "../../src-tauri/bindings/KeyPositionConfig";

export async function getKeycode(id: string, position: KeyPosition): Promise<number> {
    return await queryBackend('keycode_get', id, position)
}

export async function getKeyMap(id: string): Promise<Array<Array<KeyPositionConfig>>> {
    return await queryBackend('keymap_get', id, null)
}
