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
		this.objectManager = e, this.objectManager && typeof this.objectManager.createGameObject == "function" && console.log("object_manager...is valid:"), this.initializeWebRTC();
	}
	async initializeWebRTC() {
		let e = this.objectManager.createGameObject("network_system");
		if (e) try {
			this.webRTC = e.getComponent("WebRTC") || e.addComponent("WebRTC"), this.webRTC && (console.log("✅ WebRTC component linked via Static Registry"), await this.webRTC.connect(), console.log("📡 WebRTC connect processing started"));
		} catch (e) {
			console.error("❌ Failed to setup WebRTC component:", e);
		}
	}
	update = (e) => {
		if (!this.webRTC || !this.webRTC.isConnected()) return;
		let t = this.webRTC?.receiveData();
		t && this.handleData(t);
	};
	handleData(e) {
		console.log("📥 Received data:", e);
	}
};
//#endregion
export { t as WebTerminal, e as initGame };
