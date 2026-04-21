//#region src/index.ts
var e = (e) => {
	console.log("🚀 [initGame] START v0.1.1"), console.log("📦 Received objectManager:", e);
	try {
		let n = new t(e);
		return console.log("✅ [initGame] Instance created:", n), n;
	} catch (e) {
		throw console.error("❌ [initGame] CRASH during construction:", e), e;
	}
}, t = class {
	fruits = [];
	spawnTimer = 0;
	score = 0;
	objectManager;
	SPAWN_INTERVAL = 1;
	GRAVITY = -2.5;
	GROUND_Y = 0;
	constructor(e) {
		this.objectManager = e, console.log("🍎 [Constructor] Check this.objectManager:", this.objectManager), console.log("🛠 [Constructor] has createGameObject?:", !!(this.objectManager && this.objectManager.createGameObject));
	}
	update = (e) => {
		try {
			this.spawnTimer += e, this.spawnTimer >= this.SPAWN_INTERVAL && (this.spawnFruit(), this.spawnTimer = 0);
			for (let t = this.fruits.length - 1; t >= 0; t--) {
				let n = this.fruits[t], r = n.getComponent("Transform");
				if (r) {
					let i = r.position, a = i.y + this.GRAVITY * e;
					r.setPosition(i.x, a, i.z), a <= this.GROUND_Y && (console.log(`♻️ [GC] Removing fruit at ground: ${n}`), this.removeFruit(n, t));
				}
			}
		} catch (e) {
			console.error("🚨 [Update Loop] CRASH:", e), console.error("Current this:", this);
		}
	};
	spawnFruit = async () => {
		if (console.log("🍉 [spawnFruit] Attempting to create fruit..."), this.objectManager) try {
			let e = `fruit_${Date.now()}`, t = this.objectManager.createGameObject(e), n = t.getComponent("Transform") || t.addComponent("Transform"), r = (Math.random() - .5) * 10;
			n.setPosition(r, 10, 0);
			let i = t.getComponent("Mesh") || t.addComponent("Mesh"), a = import.meta.url, o = a.substring(0, a.lastIndexOf("/")), s = "", c = Math.floor(Math.random() * 3);
			c === 1 ? s = `${o}/grapes.glb` : c === 2 && (s = `${o}/apple.glb`), s ? (i.setModel(s), console.log("💎 Assigned Model URL to Core:", s)) : i.setBoxGeometry(.5, .5, .5), this.fruits.push(t);
		} catch (e) {
			console.error("❌ [spawnFruit] FAILED:", e);
		}
	};
	removeFruit(e, t) {
		this.objectManager.removeObject(e), this.fruits.splice(t, 1);
	}
};
//#endregion
export { t as FruitCatchGame, e as initGame };
