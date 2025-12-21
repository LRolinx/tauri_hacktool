export class SocketClient {
	uri: string
	ws?: WebSocket
	onOpen?: () => void
	onClose?: () => void
	onMessage?: () => void

	constructor(url: string = 'ws://localhost:9001') {
		this.uri = url
	}

	open() {
		this.ws = new WebSocket(this.uri)
		if (this.onOpen != void 0) {
			this.ws.onopen = this.onOpen
		}
		if (this.onClose != void 0) {
			this.ws.onclose = this.onClose
		} else {
			// 内部重连机制
			this.ws.onclose = () => {
				setTimeout(() => {
					// 一秒后重连
					console.log('正在重连')
					this.open()
				}, 1000)
			}
		}
		if (this.onMessage != void 0) {
			this.ws.onmessage = this.onMessage
		}
	}

	send(data: string | ArrayBufferLike | Blob | ArrayBufferView) {
		if (this.ws?.readyState == this.ws?.OPEN) {
			// 连接上才允许发送
			this.ws?.send(data)
		}
	}
}
