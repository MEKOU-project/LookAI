import { 
    IObjectManager, 
    IGameObject, 
    Transform, 
    Mesh 
} from '@mekou/engine-api';

export const initGame = (objectManager: IObjectManager) => {
    console.log("🚀 [SlamApp-Test] initGame started");
    return new SlamApp(objectManager);
};

export class SlamApp {
    private testCube: IGameObject;

    constructor(objectManager: IObjectManager) {
        console.log("🛠 [SlamApp-Test] Constructor");
        this.testCube = objectManager.createGameObject("SlamTestCube");
        
        const transform = this.testCube.addComponent<Transform>("Transform");
        transform.setPosition(0, 2, -5);

        const mesh = this.testCube.addComponent<Mesh>("Mesh");
        mesh.setBoxGeometry(1, 1, 1);
        console.log("✅ [SlamApp-Test] Setup complete");
    }

    // FruitCatchと同じ「アロー関数形式」にする
    public update = (dt: number): void => {
        if (!this.testCube) return;
        const transform = this.testCube.getComponent<Transform>("Transform");
        if (transform) {
            const rot = transform.rotation;
            transform.setRotation(0, rot.y + dt, 0);
        }
    }
}