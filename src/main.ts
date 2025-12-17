import { createApp } from "vue";
import App from "./App.vue";

window.ws = new WebSocket("ws://localhost:9001")

window.ws.onopen = () => {
	console.log("已连接")

	createApp(App).mount("#app");
}


