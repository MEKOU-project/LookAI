const pc = new RTCPeerConnection({
  iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
});

// トラックを受信した時の処理
pc.ontrack = (event) => {
  const el = document.createElement(event.track.kind) as HTMLVideoElement;
  el.srcObject = event.streams[0];
  el.autoplay = true;
  el.controls = true;
  document.getElementById('app')!.appendChild(el);
};

// ここにRust側とSDPを交換（Signaling）する処理を書く