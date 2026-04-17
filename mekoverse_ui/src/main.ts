import { 
        type IObjectManager,
        type IGameObject,
        type WebRTC
 } from '@mekou/engine-api';

export const initGame = (objectManager: IObjectManager) => {
    console.log("start Slam app");
    return new SlamApp(objectManager);
};

export class SlamApp {
    private webrtc: WebRTC;

    constructor(objectManager: IObjectManager) {
        const systemObj: IGameObject = objectManager.createGameObject("SlamAppSystem");

        const rtc = systemObj.getComponent<WebRTC>("WebRTC") || systemObj.addComponent<WebRTC>("WebRTC");
        this.webrtc = rtc;
    }

    public update(_deltaTime: number) {

      let data = this.webrtc.receiveData();
      if (data) {
        console.log("Received data:", data);
      }
    
    }
}