import { Notify } from 'quasar'

export function notifyError(err: string) {
    Notify.create({
        type: 'negative',
        message: 'Error: ' + err,
    })
}
