import { Notify } from 'quasar'

export function notifyError(err: any) {
    Notify.create({
        type: 'negative',
        message: 'Error: ' + err,
    })
}
