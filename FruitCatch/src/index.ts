import { 
    IObjectManager, 
    IGameObject,
    Transform,
    Mesh
} from '@mekou/engine-api';

export const initGame = (objectManager: IObjectManager) => {
    console.log("🚀 [initGame] START v0.1.1");
    console.log("📦 Received objectManager:", objectManager);
    
    try {
        const game = new FruitCatchGame(objectManager);
        console.log("✅ [initGame] Instance created:", game);
        return game;
    } catch (e) {
        console.error("❌ [initGame] CRASH during construction:", e);
        throw e;
    }
};

export class FruitCatchGame {
    private fruits: IGameObject[] = [];
    private spawnTimer: number = 0;
    private score: number = 0;
    private objectManager: IObjectManager;

    // 設定値
    private readonly SPAWN_INTERVAL = 1.0; // 1秒ごとに生成
    private readonly GRAVITY = -2.5;       // 落下速度
    private readonly GROUND_Y = 0;         // 消滅する地面の高さ

    constructor(objectManager: IObjectManager) {
        this.objectManager = objectManager;
        console.log("🍎 [Constructor] Check this.objectManager:", this.objectManager);
        // メソッドが存在するか、型が正しいかまでチェック
        console.log("🛠 [Constructor] has createGameObject?:", !!(this.objectManager && this.objectManager.createGameObject));
    }

    /**
     * エンジンのメインループから毎フレーム呼ばれる
     * @param dt 前フレームからの経過時間 (秒)
     */
    public update = (dt: number): void => {

        try {
            // 生成処理
            this.spawnTimer += dt;
            if (this.spawnTimer >= this.SPAWN_INTERVAL) {
                this.spawnFruit();
                this.spawnTimer = 0;
            }

            // 落下処理 ＆ 消す処理
            for (let i = this.fruits.length - 1; i >= 0; i--) {
                const fruit = this.fruits[i];
                
                const transform = fruit.getComponent<Transform>("Transform");

                if (transform) {
                    // 1. 座標の更新 (簡易重力)
                    const pos = transform.position;
                    const nextY = pos.y + (this.GRAVITY * dt);
                    transform.setPosition(pos.x, nextY, pos.z);

                    // 2. 地面判定による削除
                    if (nextY <= this.GROUND_Y) {
                        console.log(`♻️ [GC] Removing fruit at ground: ${fruit}`);
                        this.removeFruit(fruit, i);
                    }
                }
            }
        } catch (e) {
            console.error("🚨 [Update Loop] CRASH:", e);
            console.error("Current this:", this);
        }
    }

    private spawnFruit = async (): Promise<void> => { // async を追加
        console.log("🍉 [spawnFruit] Attempting to create fruit...");
        
        if (!this.objectManager) return;

        try {
            const id = `fruit_${Date.now()}`;
            const fruit = this.objectManager.createGameObject(id);

            // Transform
            let transform = fruit.getComponent<Transform>("Transform") || fruit.addComponent<Transform>("Transform");
            const startX = (Math.random() - 0.5) * 10;
            transform.setPosition(startX, 10, 0);

            // Mesh
            let mesh = fruit.getComponent<Mesh>("Mesh") || fruit.addComponent<Mesh>("Mesh");

            const scriptUrl = import.meta.url;
            const baseUrl = scriptUrl.substring(0, scriptUrl.lastIndexOf('/'));

            let modelPath = "";
            const random = Math.floor(Math.random() * 3);

            if (random === 1) modelPath = `${baseUrl}/grapes.glb`;
            else if (random === 2) modelPath = `${baseUrl}/apple.glb`;

            if (modelPath) {
                // オプションA: CoreのAssetLoaderを信じてURLを丸投げする (推奨)
                mesh.setModel(modelPath); 
                console.log("💎 Assigned Model URL to Core:", modelPath);
                
                /* もしどうしても手動でBlob化したい場合はここだけで止める。
                   下の古い if(random === 1) ... ブロックは絶対に削除すること。
                */
            } else {
                mesh.setBoxGeometry(0.5, 0.5, 0.5);
            }
            
            this.fruits.push(fruit);
        } catch (e) {
            console.error("❌ [spawnFruit] FAILED:", e);
        }
    }

    private removeFruit(fruit: IGameObject, index: number): void {
        this.objectManager.removeObject(fruit);
        this.fruits.splice(index, 1);
    }
}