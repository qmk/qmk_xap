import { createApp } from "vue";
import { createPinia } from 'pinia'
import { Quasar } from 'quasar'
import App from "./App.vue";

// Import icon libraries
import '@quasar/extras/roboto-font-latin-ext/roboto-font-latin-ext.css'
import '@quasar/extras/material-icons/material-icons.css'
import '@quasar/extras/fontawesome-v6/fontawesome-v6.css'
import 'quasar/src/css/index.sass'

const pinia = createPinia()
const app = createApp(App)

app.use(Quasar, {
    plugins: {}
})
app.use(pinia)
app.mount("#app")
