//#region src/index.ts
var e = (e) => {
	console.log("📦 Received objectManager:", e);
	try {
		let n = new t(e);
		return console.log("✅ [initGame] Instance created:", n), n;
	} catch (e) {
		throw console.error("❌ [initGame] CRASH during construction:", e), e;
	}
}, t = class {
	objectManager;
	webRTC = null;
	constructor(e) {
		this.objectManager = e, this.objectManager && typeof this.objectManager.createGameObject == "function" && console.log("object_manager...is valid:"), this.initializeWebRTC(), this.CameraInit();
	}
	async initializeWebRTC() {
		let e = this.objectManager.createGameObject("network_system");
		if (e) try {
			this.webRTC = e.getComponent("WebRTC") || e.addComponent("WebRTC"), this.webRTC && (console.log("✅ WebRTC component linked via Static Registry"), await this.webRTC.connect(), console.log("📡 WebRTC connect processing started"));
		} catch (e) {
			console.error("❌ Failed to setup WebRTC component:", e);
		}
	}
	async CameraInit() {
		let e = this.objectManager.createGameObject("camera");
		e && e.addComponent("Camera");
	}
	update = (e) => {
		if (this.webRTC && !this.webRTC.isStreaming()) {
			let e = (this.objectManager.findGameObject("camera")?.getComponent("Camera"))?.getStream();
			e && (this.webRTC.addStream(e), console.log("🚀 Stream passed to WebRTC"));
		}
		if (this.webRTC && this.webRTC.isConnected()) {
			let e = this.webRTC.receiveData();
			e && this.handleData(e);
		}
	};
	handleData(e) {
		console.log("📥 Received data:", e);
	}
};
//#endregion
export { t as WebTerminal, e as initGame };
