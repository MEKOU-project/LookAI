import { 
    IObjectManager,
    IGameObject,
    WebRTC,
    Camera,
} from '@mekou/engine-api';


export const initGame = (objectManager: IObjectManager) => {
    console.log("📦 Received objectManager:", objectManager);
    
    try {
        const game = new WebTerminal(objectManager);
        console.log("✅ [initGame] Instance created:", game);
        return game;
    } catch (e) {
        console.error("❌ [initGame] CRASH during construction:", e);
        throw e;
    }
};

export class WebTerminal {
    private objectManager: IObjectManager;
    private webRTC: WebRTC | null = null;

    constructor(objectManager: IObjectManager) {
        this.objectManager = objectManager;
        if(this.objectManager && typeof this.objectManager.createGameObject === 'function') {
            console.log("object_manager...is valid:");
        }
        this.initializeWebRTC();
        this.CameraInit();
    }

    private async initializeWebRTC() {
        const network_system = this.objectManager.createGameObject("network_system");

        if (network_system) {
            try {
                // 固定ロードにしたので、直接 addComponent を呼ぶだけでOK
                // 内部で ComponentRegistry.getRegisteredClass("WebRTC") が走り、即座に実体が返る
                this.webRTC = network_system.getComponent<WebRTC>("WebRTC") ||
                              network_system.addComponent<WebRTC>("WebRTC");

                if (this.webRTC) {
                    console.log("✅ WebRTC component linked via Static Registry");
                    await this.webRTC.connect();
                    console.log("📡 WebRTC connect processing started");
                }
            } catch (e) {
                console.error("❌ Failed to setup WebRTC component:", e);
            }
        }
    }

    private async CameraInit(){
        const cameraObject = this.objectManager.createGameObject("camera");
        if (cameraObject) {
            const cameraComponent = cameraObject.addComponent<Camera>("Camera");
        }
    }

    /**
     * エンジンのメインループから毎フレーム呼ばれる
     */
    public update = (dt: number): void => {
        // 1. 送信準備
        if (this.webRTC && !this.webRTC.isStreaming()) {
            const camObj = this.objectManager.findGameObject("camera");
            const camera = camObj?.getComponent<Camera>("Camera"); // CameraComponent ではなくインターフェースの Camera
            const stream = camera?.getStream();
            
            if (stream) {
                this.webRTC.addStream(stream); 
                // ここで addStream を呼べば、次フレームの isStreaming() は true を返すはず
                console.log("🚀 Stream passed to WebRTC");
            }
        }

        // 2. データ受信
        if (this.webRTC && this.webRTC.isConnected()) {
            const data = this.webRTC.receiveData();
            if (data) this.handleData(data);
        }
    }

    private handleData(data: any) {
        console.log("📥 Received data:", data);
    }
}