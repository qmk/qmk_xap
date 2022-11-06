import { queryBackend } from "@/commands/core";

export async function getKeycode(id: string, layer: number, row: number, col: number): Promise<number> {
    return await queryBackend('keycode_get', id, { layer: layer, row: row, col })
}
