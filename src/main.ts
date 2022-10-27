import { createApp } from "vue";
import { Quasar } from 'quasar'

// Import icon libraries
import '@quasar/extras/roboto-font-latin-ext/roboto-font-latin-ext.css'
import '@quasar/extras/material-icons/material-icons.css'
import '@quasar/extras/fontawesome-v6/fontawesome-v6.css'

// Import Quasar css
import 'quasar/src/css/index.sass'

// import "./style.css";
import App from "./App.vue";

const app = createApp(App)


app.use(Quasar, {
    plugins: {}
})
app.mount("#app");
