import { createApp } from "vue";
import { createPinia } from 'pinia'
import { Quasar } from 'quasar'
import { listen } from '@tauri-apps/api/event'

// Import icon libraries
import '@quasar/extras/roboto-font-latin-ext/roboto-font-latin-ext.css'
import '@quasar/extras/material-icons/material-icons.css'
import '@quasar/extras/fontawesome-v6/fontawesome-v6.css'

// Import Quasar css
import 'quasar/src/css/index.sass'

// import "./style.css";
import App from "./App.vue";

const pinia = createPinia()
const app = createApp(App)

app.use(Quasar, {
    plugins: {}
})
app.use(pinia)
app.mount("#app")

await listen('new-device', event => {
    console.log("new device with id")
    console.log(event.payload.id)
    window.location.reload();
    // TODO - doesn't work as I want it to be
    // const instance = getCurrentInstance();
    // instance?.proxy?.$forceUpdate();
})

await listen('removed-device', event => {
    console.log("removed device with id")
    console.log(event.payload.id)
    window.location.reload();
    // TODO - doesn't work as I want it to be
    // const instance = getCurrentInstance();
    // instance?.proxy?.$forceUpdate();
})
