import { Notify } from 'quasar'

export function notifyError(err: unknown) {
    Notify.create({
        type: 'negative',
        message: 'Error: ' + err,
    })
}
