import { createApp } from 'vue'
import { createPinia } from 'pinia'

import { Quasar } from 'quasar'
import '@quasar/extras/roboto-font-latin-ext/roboto-font-latin-ext.css'
import '@quasar/extras/material-icons/material-icons.css'
import '@quasar/extras/fontawesome-v6/fontawesome-v6.css'
import 'quasar/src/css/index.sass'

import App from '@/App.vue'
import router from '@/router/routes'

createApp(App)
.use(Quasar, {
    plugins: {}
})
.use(createPinia())
.use(router)
.mount('#app')
