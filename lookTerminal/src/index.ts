import { 
    IObjectManager,
    IGameObject,
    WebRTC
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

    /**
     * エンジンのメインループから毎フレーム呼ばれる
     */
    public update = (dt: number): void => {
        if (!this.webRTC || !this.webRTC.isConnected()) {
            return;
        }
        const data = this.webRTC?.receiveData();
        if (data) {
            this.handleData(data);
        }
    }

    private handleData(data: any) {
        console.log("📥 Received data:", data);
    }
}