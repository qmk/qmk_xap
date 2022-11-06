import { invoke } from "@tauri-apps/api/tauri"

export async function queryBackend<T, R>(handler: string, id: string, arg: T): Promise<R> {
    let prettyArg = (arg != null || arg != undefined) ? JSON.stringify(arg) : '()'
    let truncatedId = "(" + id.substring(0, 4) + ")"
    return await invoke(handler, { id: id, arg: arg }).then(
        (ok: R) => {
            let prettyOk = (arg != null || arg != undefined) ? JSON.stringify(ok) : '()'
            console.log(truncatedId + " ok: " + handler + " arg: " + prettyArg + " response: " + prettyOk)
            return ok
        },
        (err: string) => {
            console.error(truncatedId + "error: " + handler + " arg: " + prettyArg + " error: " + err)
            throw err
        })
}

export async function callBackend<T, R>(handler: string, id: string): Promise<void> {
    let truncatedId = "(" + id.substring(0, 4) + ")"
    return await invoke(handler, { id: id }).then(
        () => {
            console.log(truncatedId + " ok: " + handler)
        },
        (err: string) => {
            console.error(truncatedId + " error: " + handler + " error: " + err)
            throw err
        })
}
