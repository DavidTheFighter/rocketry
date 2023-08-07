<template>
  <video :style="videoStyle" ref="videoStream"></video>
</template>

<script>
export default {
  name: 'VideoStream',
  mounted() {
    this.setupVideoElement();
  },
  methods: {
    async setupStream(browser_description) {
      const response = await fetch('http://localhost:8000/browser-stream', {
        method: 'POST',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json' ,
          'Access-Control-Allow-Origin': '*',
          'Access-Control-Allow-Methods': 'POST',
        },
        body: JSON.stringify({
          camera_index: 0,
          browser_session: browser_description,
        }),
      });

      if (!response.ok) {
        this.videoStyle.backgroundColor = '#F00';
        return;
      }

      const data = await response.json();
      const session_description = JSON.parse(atob(data.text_response));
      this.pc.setRemoteDescription(new RTCSessionDescription(session_description));
    },
    setupVideoElement() {
      this.pc = new RTCPeerConnection({
        iceServers: [
          {
            urls: 'stun:stun.l.google.com:19302'
          }
        ]
      });

      this.pc.ontrack = (event) => {
        this.$refs.videoStream.srcObject = event.streams[0];
        this.$refs.videoStream.autoplay = true;
        this.$refs.videoStream.controls = false;
      };

      // this.pc.oniceconnectionstatechange = () => log(this.pc.iceConnectionState);
      this.pc.onicecandidate = (event) => {
        if (event.candidate === null) {
          this.setupStream(btoa(JSON.stringify(this.pc.localDescription)));
        }
      };

      this.pc.addTransceiver('video', {'direction': 'recvonly'})
      this.pc.createOffer().then(d => this.pc.setLocalDescription(d)).catch(() => {});
    }
  },
  data() {
    return {
      videoStyle: {
        backgroundColor: '#000',
      },
    };
  },
}
</script>

<style scoped>

</style>