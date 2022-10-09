import 'vuetify/styles'
import { createApp } from "vue";
import { createVuetify } from "vuetify";
import "./style.css";
import App from "./App.vue";

const app = createApp(App)
const vuetify = createVuetify()

app.use(vuetify)
app.mount("#app");
