import { createApp } from 'vue'
import { createPinia } from 'pinia'

import { Quasar, Notify, Loading } from 'quasar'
import '@quasar/extras/roboto-font-latin-ext/roboto-font-latin-ext.css'
import '@quasar/extras/material-icons/material-icons.css'
import '@quasar/extras/fontawesome-v6/fontawesome-v6.css'
import 'quasar/src/css/index.sass'

import Main from '@/Main.vue'
import router from '@/router/routes'

createApp(Main)
    .use(Quasar, {
        plugins: [
            Notify,
            Loading
        ],
        config: {
            notify: {
                position: 'top-right'
            }
        }
    })
    .use(createPinia())
    .use(router)
    .mount('#qmk-xap-app')
