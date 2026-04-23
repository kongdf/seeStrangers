<template>
  <div class="container">
    <el-container direction="vertical">
      <el-header>
        <div class="header-content">
          <div class="logo">
            <el-icon :size="28" color="#fff">
              <VideoCamera />
            </el-icon>
          </div>
          <div class="title-area">
            <h1>WebRTC 视频通话</h1>
            <p>{{ state.currentRoom ? '房间: ' + state.currentRoom : '安全 · 加密 · P2P直连' }}</p>
          </div>
        </div>
      </el-header>

      <div class="main-content">
        <div class="video-grid" :class="{ 'has-screen': state.isScreenSharing || state.remoteScreenShare }">
          <div class="videoBox screen-share" v-if="state.isScreenSharing || state.remoteScreenShare">
            <video ref="screenShareEl" autoplay playsinline></video>
            <div class="box-badge active">共享中</div>
          </div>
          <div class="videoBox remote">
            <video ref="remoteVideoEl" autoplay playsinline></video>
            <audio ref="remoteAudioEl" autoplay></audio>
            <div class="box-content" v-if="!state.remoteStream">
              <el-icon :size="40" color="#3b82f6">
                <User />
              </el-icon>
              <span>{{ state.statusText }}</span>
            </div>
            <div class="box-info" v-if="state.remoteStream">
              <span>远端画面</span>
            </div>
          </div>
          <div class="videoBox local">
            <video ref="localVideoEl" autoplay playsinline muted></video>
            <div class="box-info">
              <span>本地</span>
            </div>
          </div>
        </div>
      </div>

      <el-footer v-if="state.isInCall">
        <div class="controls">
          <el-button class="control-btn" :class="{ 'off': !state.isAudioEnabled }" circle @click="toggleAudio">
            <el-icon :size="20">
              <Microphone v-if="state.isAudioEnabled" />
              <Mute v-else />
            </el-icon>
          </el-button>
          <el-button class="control-btn" :class="{ 'off': !state.isVideoEnabled }" circle @click="toggleVideo">
            <el-icon :size="20">
              <VideoCamera v-if="state.isVideoEnabled" />
              <VideoCameraFilled v-else />
            </el-icon>
          </el-button>
          <el-button class="control-btn" :class="{ 'active': state.isScreenSharing }" circle @click="toggleScreenShare">
            <el-icon :size="20">
              <Monitor />
            </el-icon>
          </el-button>
          <el-button class="control-btn hangup" circle @click="hangup">
            <el-icon :size="20">
              <PhoneFilled />
            </el-icon>
          </el-button>
        </div>
      </el-footer>

      <el-dialog v-model="state.showMainDialog" class="main-dialog" width="460px" :close-on-click-modal="false"
        :close-on-press-escape="false" :show-close="false">
        <template #header>
          <div class="dialog-header">
            <div class="header-icon">
              <el-icon :size="32" color="#fff">
                <VideoCamera />
              </el-icon>
            </div>
            <h2>开始通话</h2>
            <p>选择通话方式，开启私密对话</p>
          </div>
        </template>
        <div class="dialog-body">
          <div class="mode-section">
            <div class="mode-btn" :class="{ active: state.callMode === 'room' }" @click="state.callMode = 'room'">
              <div class="mode-icon">
                <el-icon :size="24">
                  <OfficeBuilding />
                </el-icon>
              </div>
              <div class="mode-info">
                <span class="mode-title">加入房间</span>
                <span class="mode-desc">输入房间号加入</span>
              </div>
              <div class="mode-radio">
                <div class="radio-dot" v-if="state.callMode === 'room'"></div>
              </div>
            </div>
            <div class="mode-btn" :class="{ active: state.callMode === 'match' }" @click="state.callMode = 'match'">
              <div class="mode-icon pink">
                <el-icon :size="24">
                  <User />
                </el-icon>
              </div>
              <div class="mode-info">
                <span class="mode-title">随机匹配</span>
                <span class="mode-desc">快速匹配在线用户</span>
              </div>
              <div class="mode-radio">
                <div class="radio-dot" v-if="state.callMode === 'match'"></div>
              </div>
            </div>
          </div>

          <div class="device-section">
            <div class="device-item">
              <div class="device-header">
                <el-icon :size="18" color="#a855f7">
                  <VideoCamera />
                </el-icon>
                <span>摄像头</span>
              </div>
              <el-select v-model="state.selectedVideo" placeholder="选择摄像头" size="large" filterable>
                <el-option v-for="device in state.videoDevices" :key="device.deviceId"
                  :label="device.label || '摄像头 ' + (state.videoDevices.indexOf(device) + 1)" :value="device.deviceId" />
              </el-select>
            </div>
            <div class="device-item">
              <div class="device-header">
                <el-icon :size="18" color="#667eea">
                  <Microphone />
                </el-icon>
                <span>麦克风</span>
              </div>
              <el-select v-model="state.selectedAudio" placeholder="选择麦克风" size="large" filterable>
                <el-option v-for="device in state.audioDevices" :key="device.deviceId"
                  :label="device.label || '麦克风 ' + (state.audioDevices.indexOf(device) + 1)" :value="device.deviceId" />
              </el-select>
            </div>
          </div>

          <div class="room-input" v-if="state.callMode === 'room'">
            <el-input v-model="state.roomId" placeholder="请输入房间号" size="large">
              <template #prefix>
                <el-icon>
                  <OfficeBuilding />
                </el-icon>
              </template>
            </el-input>
          </div>

          <div class="action-section">
            <el-button class="action-btn primary" size="large" @click="startMatch" :loading="state.isMatching"
              v-if="state.callMode === 'match'" :disabled="state.isMatching">
              <el-icon v-if="!state.isMatching">
                <Refresh />
              </el-icon>
              {{ state.isMatching ? '匹配中...' : '随机匹配' }}
            </el-button>
            <el-button class="action-btn primary" size="large" @click="joinRoom" v-if="state.callMode === 'room'"
              :disabled="!state.roomId.trim()">
              <el-icon>
                <VideoCamera />
              </el-icon>
              加入通话
            </el-button>
          </div>
        </div>
      </el-dialog>
    </el-container>
  </div>
</template>

<script setup>
import { reactive, ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { VideoCamera, VideoCameraFilled, Microphone, Mute, Monitor, User, UserFilled, PhoneFilled, Refresh, OfficeBuilding } from '@element-plus/icons-vue'

const ICE_SERVERS = {
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' },
    {
      urls: "stun:stun.relay.metered.ca:80",
    },
    {
      urls: "turn:global.relay.metered.ca:80",
      username: "52714d68b0d839e7317764a8",
      credential: "+j3wBoOxuzPXE7Eh",
    },
    {
      urls: "turn:global.relay.metered.ca:80?transport=tcp",
      username: "52714d68b0d839e7317764a8",
      credential: "+j3wBoOxuzPXE7Eh",
    },
    {
      urls: "turn:global.relay.metered.ca:443",
      username: "52714d68b0d839e7317764a8",
      credential: "+j3wBoOxuzPXE7Eh",
    },
    {
      urls: "turns:global.relay.metered.ca:443?transport=tcp",
      username: "52714d68b0d839e7317764a8",
      credential: "+j3wBoOxuzPXE7Eh",
    },]
}

const WS_URL = `wss://mojianws.kongdf.com`

const localVideoEl = ref(null)
const remoteVideoEl = ref(null)
const remoteAudioEl = ref(null)
const screenShareEl = ref(null)

const state = reactive({
  isScreenSharing: false,
  remoteScreenShare: false,
  isAudioEnabled: true,
  isVideoEnabled: true,
  isMatching: false,
  isInCall: false,
  showMainDialog: true,
  callMode: 'room',
  roomId: '',
  currentRoom: '',
  audioDevices: [],
  videoDevices: [],
  selectedAudio: '',
  selectedVideo: '',
  remoteStream: null,
  localStream: null,
  statusText: '等待对方加入...',
})

let ws = null
let pc = null
let localStream = null
let screenStream = null
let currentUserId = null
let isInitiator = false
let joinedFirst = false
let pendingCandidates = []
let videoTrackCount = 0
let pendingScreenStream = null

async function loadDevices() {
  try {
    const tempStream = await navigator.mediaDevices.getUserMedia({ audio: true, video: true })
    tempStream.getTracks().forEach(track => track.stop())
    const devices = await navigator.mediaDevices.enumerateDevices()
    state.audioDevices = devices.filter(d => d.kind === 'audioinput')
    state.videoDevices = devices.filter(d => d.kind === 'videoinput')
    if (state.audioDevices.length > 0) state.selectedAudio = state.audioDevices[0].deviceId
    if (state.videoDevices.length > 0) state.selectedVideo = state.videoDevices[0].deviceId
  } catch (err) {
    console.error('获取设备失败:', err)
  }
}

async function getLocalStream() {
  if (localStream) {
    localStream.getTracks().forEach(track => track.stop())
  }
  const videoConstraints = state.selectedVideo
    ? { deviceId: { exact: state.selectedVideo }, width: { ideal: 1920 }, height: { ideal: 1080 }, frameRate: { ideal: 30 } }
    : { width: { ideal: 1920 }, height: { ideal: 1080 }, frameRate: { ideal: 30 }, facingMode: 'user' }

  const audioConstraints = state.selectedAudio
    ? { deviceId: { exact: state.selectedAudio }, echoCancellation: true, noiseSuppression: true, sampleRate: 44100 }
    : { echoCancellation: true, noiseSuppression: true, sampleRate: 44100 }

  localStream = await navigator.mediaDevices.getUserMedia({
    video: videoConstraints,
    audio: audioConstraints
  })
  state.localStream = localStream
  state.isAudioEnabled = true
  state.isVideoEnabled = true
  if (localVideoEl.value) {
    localVideoEl.value.srcObject = localStream
  }
}

function createPeerConnection() {
  pc = new RTCPeerConnection(ICE_SERVERS)

  pc.onicecandidate = (event) => {
    if (event.candidate && ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({
        type: 'ice_candidate',
        candidate: event.candidate,
        roomId: state.currentRoom
      }))
    }
  }

  let videoTrackCount = 0
  let pendingScreenStream = null

  pc.ontrack = (event) => {
    if (event.track.kind === 'video') {
      videoTrackCount++
      console.log('收到视频轨道:', videoTrackCount, 'stream id:', event.streams[0]?.id)
      if (videoTrackCount === 1) {
        state.remoteStream = event.streams[0]
        if (remoteVideoEl.value) {
          remoteVideoEl.value.srcObject = event.streams[0]
        }
      } else if (videoTrackCount === 2) {
        console.log('收到屏幕共享流')
        nextTick(() => {
          if (screenShareEl.value) {
            screenShareEl.value.srcObject = event.streams[0]
            console.log('已绑定屏幕共享流到元素')
          } else {
            pendingScreenStream = event.streams[0]
            console.log('元素未就绪，缓存屏幕共享流')
          }
        })
      }
    }
    if (remoteAudioEl.value) {
      remoteAudioEl.value.srcObject = event.streams[0]
    }
  }

  pc.oniceconnectionstatechange = () => {
    console.log('ICE状态:', pc.iceConnectionState)
    if (pc.iceConnectionState === 'connected') {
      state.statusText = '连接成功'
    } else if (pc.iceConnectionState === 'disconnected' || pc.iceConnectionState === 'failed') {
      state.statusText = '连接断开'
    }
  }

  localStream.getTracks().forEach(track => {
    pc.addTrack(track, localStream)
  })

  return pc
}

function send(msg) {
  if (ws?.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(msg))
  }
}

async function createOffer() {
  const offer = await pc.createOffer()
  await pc.setLocalDescription(offer)
  send({ type: 'offer', offer: pc.localDescription, roomId: state.currentRoom })
}

async function handleOffer(msg) {
  await pc.setRemoteDescription(new RTCSessionDescription(msg.offer))
  for (const c of pendingCandidates) {
    await pc.addIceCandidate(new RTCIceCandidate(c))
  }
  pendingCandidates = []
  const answer = await pc.createAnswer()
  await pc.setLocalDescription(answer)
  send({ type: 'answer', answer: pc.localDescription, roomId: state.currentRoom })
}

async function handleAnswer(msg) {
  await pc.setRemoteDescription(new RTCSessionDescription(msg.answer))
  for (const c of pendingCandidates) {
    await pc.addIceCandidate(new RTCIceCandidate(c))
  }
  pendingCandidates = []
}

async function handleIceCandidate(msg) {
  if (msg.candidate) {
    if (pc.remoteDescription) {
      await pc.addIceCandidate(new RTCIceCandidate(msg.candidate))
    } else {
      pendingCandidates.push(msg.candidate)
    }
  }
}

function connect() {
  ws = new WebSocket(WS_URL)

  ws.onopen = async () => {
    console.log('WebSocket已连接')
    await getLocalStream()
    createPeerConnection()

    if (state.callMode === 'room') {
      send({ type: 'join', roomId: state.currentRoom, userId: currentUserId })
    } else {
      send({ type: 'random_match', userId: currentUserId })
    }
  }

  ws.onmessage = async (event) => {
    const msg = JSON.parse(event.data)
    console.log('收到:', msg.type, msg)

    switch (msg.type) {
      case 'waiting':
        state.statusText = '等待对方加入...'
        joinedFirst = true
        break

      case 'peer_joined':
        state.statusText = '对方已加入，正在建立连接...'
        break

      case 'ready':
        state.statusText = '对方已加入，正在建立连接...'
        await createOffer()
        break

      case 'match_found':
        state.currentRoom = msg.roomId
        state.isMatching = false
        if (!msg.isInitiator) {
          await createOffer()
        }
        break

      case 'waiting_for_match':
        state.statusText = '正在寻找匹配...'
        break

      case 'offer':
        await handleOffer(msg)
        break

      case 'answer':
        await handleAnswer(msg)
        break

      case 'ice_candidate':
        await handleIceCandidate(msg)
        break

      case 'peer_left':
        state.statusText = '对方已离开'
        state.remoteStream = null
        break

      case 'room_full':
        state.statusText = '房间已满'
        break

      case 'screen_share_start':
        state.remoteScreenShare = true
        break

      case 'screen_share_stop':
        state.remoteScreenShare = false
        break
    }
  }

  ws.onclose = () => console.log('WebSocket已关闭')
  ws.onerror = (err) => console.error('WebSocket错误:', err)
}

async function joinRoom() {
  if (!state.roomId.trim()) return
  currentUserId = 'user_' + Math.random().toString(36).substr(2, 9)
  state.currentRoom = state.roomId
  state.showMainDialog = false
  state.isInCall = true
  state.statusText = '正在连接...'
  joinedFirst = false
  isInitiator = false
  connect()
}

async function startMatch() {
  currentUserId = 'user_' + Math.random().toString(36).substr(2, 9)
  state.isMatching = true
  state.statusText = '正在寻找匹配...'
  state.showMainDialog = false
  state.isInCall = true
  joinedFirst = false
  isInitiator = false
  connect()
}

function toggleAudio() {
  if (localStream) {
    const track = localStream.getAudioTracks()[0]
    if (track) {
      track.enabled = !track.enabled
      state.isAudioEnabled = track.enabled
    }
  }
}

function toggleVideo() {
  if (localStream) {
    const track = localStream.getVideoTracks()[0]
    if (track) {
      track.enabled = !track.enabled
      state.isVideoEnabled = track.enabled
    }
  }
}

async function toggleScreenShare() {
  if (state.isScreenSharing) {
    stopScreenShare()
  } else {
    await startScreenShare()
  }
}

async function startScreenShare() {
  try {
    screenStream = await navigator.mediaDevices.getDisplayMedia({ video: true, audio: false })
    const screenTrack = screenStream.getVideoTracks()[0]

    if (pc) {
      pc.addTrack(screenTrack, screenStream)
      const offer = await pc.createOffer()
      await pc.setLocalDescription(offer)
      send({ type: 'offer', offer: pc.localDescription, roomId: state.currentRoom })
      send({ type: 'screen_share_start', roomId: state.currentRoom })
    }

    screenTrack.onended = () => stopScreenShare()
    state.isScreenSharing = true
    nextTick(() => {
      if (screenShareEl.value) {
        screenShareEl.value.srcObject = screenStream
        console.log('A本地绑定屏幕共享流')
      }
    })
  } catch (err) {
    console.error('屏幕共享失败:', err)
  }
}

function stopScreenShare() {
  if (screenStream) {
    screenStream.getTracks().forEach(track => track.stop())
    screenStream = null
  }
  if (pc) {
    const senders = pc.getSenders()
    senders.forEach(sender => {
      if (sender.track && sender.track.kind === 'video' && localStream && !localStream.getVideoTracks().includes(sender.track)) {
        pc.removeTrack(sender)
      }
    })
    send({ type: 'screen_share_stop', roomId: state.currentRoom })
  }
  state.isScreenSharing = false
  if (screenShareEl.value) {
    screenShareEl.value.srcObject = null
  }
}

function hangup() {
  send({ type: 'leave', roomId: state.currentRoom, userId: currentUserId })
  closeCall()
}

function closeCall() {
  if (localStream) {
    localStream.getTracks().forEach(track => track.stop())
    localStream = null
  }
  if (screenStream) {
    screenStream.getTracks().forEach(track => track.stop())
    screenStream = null
  }
  if (pc) {
    pc.close()
    pc = null
  }
  if (ws) {
    ws.close()
    ws = null
  }
  pendingCandidates = []
  joinedFirst = false
  isInitiator = false
  videoTrackCount = 0
  pendingScreenStream = null
  state.remoteScreenShare = false
  state.isInCall = false
  state.isScreenSharing = false
  state.remoteStream = null
  state.localStream = null
  state.currentRoom = ''
  state.showMainDialog = true
  state.isMatching = false
  state.statusText = '等待对方加入...'
  if (localVideoEl.value) localVideoEl.value.srcObject = null
  if (remoteVideoEl.value) remoteVideoEl.value.srcObject = null
  if (screenShareEl.value) screenShareEl.value.srcObject = null
}

onMounted(() => {
  loadDevices()
})

watch(() => state.remoteScreenShare, (newVal) => {
  if (newVal && pendingScreenStream && screenShareEl.value) {
    screenShareEl.value.srcObject = pendingScreenStream
    pendingScreenStream = null
    console.log('屏幕共享窗口就绪后绑定流')
  }
})

watch(screenShareEl, (el) => {
  if (el && pendingScreenStream) {
    el.srcObject = pendingScreenStream
    pendingScreenStream = null
    console.log('元素就绪后绑定屏幕共享流')
  }
})

onUnmounted(() => {
  closeCall()
})
</script>

<style scoped>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

.container {
  width: 100%;
  height: 100vh;
  background: #0a0a0f;
}

.el-container {
  height: 100%;
}

.el-header {
  background: transparent;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.header-content {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 24px;
}

.logo {
  width: 40px;
  height: 40px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 16px rgba(102, 126, 234, 0.3);
  flex-shrink: 0;
}

.title-area h1 {
  color: #fff;
  font-size: 15px;
  font-weight: 600;
  margin: 0;
}

.title-area p {
  color: rgba(255, 255, 255, 0.4);
  font-size: 10px;
  margin: 2px 0 0 0;
}

.main-content {
  flex: 1;
  padding: 8px 24px;
  min-height: 0;
}

.video-grid {
  display: grid;
  grid-template-columns: 1fr 300px;
  grid-template-rows: 1fr 1fr;
  gap: 8px;
  height: 100%;
}

.videoBox.remote {
  grid-column: 1 !important;
  grid-row: 1 / 3 !important;
}

.videoBox.local {
  grid-column: 2 !important;
  grid-row: 1 !important;
}

.has-screen .videoBox.screen-share {
  grid-column: 1 !important;
  grid-row: 1 / 3 !important;
}

.has-screen .videoBox.remote {
  grid-column: 2 !important;
  grid-row: 1 !important;
}

.has-screen .videoBox.local {
  grid-column: 2 !important;
  grid-row: 2 !important;
}

.videoBox {
  background: #12121a;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.videoBox video {
  position: absolute;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.box-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  z-index: 1;
}

.box-content span {
  color: rgba(255, 255, 255, 0.5);
  font-size: 12px;
  font-weight: 500;
}

.box-info {
  position: absolute;
  bottom: 10px;
  left: 10px;
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 10px;
  font-weight: 500;
  background: rgba(0, 0, 0, 0.5);
  color: rgba(255, 255, 255, 0.8);
  z-index: 2;
}

.box-badge {
  position: absolute;
  top: 10px;
  left: 10px;
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 10px;
  font-weight: 600;
  background: rgba(74, 222, 128, 0.2);
  color: #4ade80;
  border: 1px solid rgba(74, 222, 128, 0.3);
  z-index: 2;
}

.el-footer {
  padding: 6px 24px 8px;
  background: transparent;
  flex-shrink: 0;
}

.controls {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 12px;
}

.control-btn {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.85);
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.control-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  transform: translateY(-2px);
}

.control-btn.off {
  background: rgba(239, 68, 68, 0.15);
  border-color: rgba(239, 68, 68, 0.25);
  color: #f87171;
}

.control-btn.active {
  background: rgba(74, 222, 128, 0.15);
  border-color: rgba(74, 222, 128, 0.3);
  color: #4ade80;
}

.control-btn.hangup {
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  border: none;
  color: #fff;
  box-shadow: 0 4px 16px rgba(239, 68, 68, 0.4);
}

.control-btn.hangup:hover {
  box-shadow: 0 6px 24px rgba(239, 68, 68, 0.5);
}

.el-dialog {
  background: #13131c;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 24px;
  box-shadow: 0 32px 64px rgba(0, 0, 0, 0.6);
}

.el-dialog__header {
  padding: 32px 32px 0 32px !important;
  margin: 0 !important;
}

.el-dialog__headerbtn {
  display: none !important;
}

.el-dialog__body {
  padding: 24px 32px 32px !important;
}

.dialog-header {
  text-align: center;
  margin-bottom: 24px;
}

.header-icon {
  width: 64px;
  height: 64px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 16px;
  box-shadow: 0 8px 24px rgba(102, 126, 234, 0.35);
}

.dialog-header h2 {
  color: #fff;
  font-size: 22px;
  font-weight: 600;
  margin-bottom: 6px;
}

.dialog-header p {
  color: rgba(255, 255, 255, 0.4);
  font-size: 13px;
}

.dialog-body {
  padding: 0;
}

.mode-section {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}

.mode-btn {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  background: rgba(255, 255, 255, 0.02);
  border: 1.5px solid rgba(255, 255, 255, 0.06);
  border-radius: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.mode-btn:hover {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.12);
}

.mode-btn.active {
  background: rgba(102, 126, 234, 0.08);
  border-color: #667eea;
}

.mode-icon {
  width: 44px;
  height: 44px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  flex-shrink: 0;
}

.mode-icon.pink {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.mode-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.mode-title {
  color: #fff;
  font-size: 14px;
  font-weight: 500;
}

.mode-desc {
  color: rgba(255, 255, 255, 0.4);
  font-size: 11px;
}

.mode-radio {
  width: 18px;
  height: 18px;
  border: 2px solid rgba(255, 255, 255, 0.15);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.mode-btn.active .mode-radio {
  border-color: #667eea;
}

.radio-dot {
  width: 10px;
  height: 10px;
  background: #667eea;
  border-radius: 50%;
}

.room-input {
  margin-bottom: 20px;
}

.room-input .el-input__wrapper {
  background: rgba(255, 255, 255, 0.04) !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
  box-shadow: none !important;
  border-radius: 12px !important;
  padding: 4px 16px;
}

.room-input .el-input__inner {
  color: #fff !important;
}

.room-input .el-input__inner::placeholder {
  color: rgba(255, 255, 255, 0.25) !important;
}

.room-input .el-input__prefix {
  color: rgba(255, 255, 255, 0.4);
}

.device-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 20px;
}

.device-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.device-header {
  display: flex;
  align-items: center;
  gap: 6px;
  color: rgba(255, 255, 255, 0.6);
  font-size: 12px;
  font-weight: 500;
}

.device-item .el-select {
  width: 100%;
}

.device-item .el-input__wrapper {
  background: rgba(255, 255, 255, 0.04) !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
  box-shadow: none !important;
  border-radius: 8px !important;
}

.device-item .el-input__inner {
  color: #fff !important;
}

.action-section {
  display: flex;
  gap: 12px;
}

.action-btn {
  flex: 1;
  height: 48px;
  border-radius: 12px;
  font-size: 14px;
  font-weight: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  transition: all 0.2s ease;
}

.action-btn.primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  color: #fff;
  box-shadow: 0 4px 16px rgba(102, 126, 234, 0.3);
}

.action-btn.primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
}

.action-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

@media (max-width: 768px) {
  .container {
    height: 100dvh;
  }

  .el-header {
    height: auto;
    padding: 0;
  }

  .header-content {
    padding: 10px 14px;
    gap: 10px;
  }

  .logo {
    width: 34px;
    height: 34px;
    border-radius: 9px;
  }

  .title-area h1 {
    font-size: 14px;
  }

  .title-area p {
    font-size: 10px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 72vw;
  }

  .main-content {
    padding: 8px 10px 6px;
  }

  .video-grid {
    position: relative;
    display: block;
    height: 100%;
    min-height: 52vh;
  }

  .videoBox {
    border-radius: 10px;
  }

  .videoBox.remote,
  .has-screen .videoBox.screen-share {
    position: relative;
    min-height: 100%;
    height: 100%;
    width: 100%;
  }

  .has-screen .videoBox.remote,
  .videoBox.local,
  .has-screen .videoBox.local {
    position: absolute;
    top: auto;
    left: auto;
    right: 10px;
    bottom: 10px;
    width: 34vw;
    max-width: 144px;
    min-height: 19vh;
    height: 19vh;
    border: 1px solid rgba(255, 255, 255, 0.2);
    box-shadow: 0 10px 24px rgba(0, 0, 0, 0.45);
    z-index: 4;
  }

  .has-screen .videoBox.remote {
    top: 10px;
    left: 10px;
    right: auto;
    bottom: auto;
    width: 30vw;
    max-width: 128px;
    min-height: 16vh;
    height: 16vh;
    z-index: 4;
  }

  .box-info,
  .box-badge {
    font-size: 11px;
    padding: 5px 10px;
  }

  .el-footer {
    padding: 10px 12px calc(12px + env(safe-area-inset-bottom));
    background: transparent;
  }

  .controls {
    padding: 8px 10px;
    border-radius: 16px;
    background: rgba(13, 13, 20, 0.62);
    border: 1px solid rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    box-shadow: 0 10px 28px rgba(0, 0, 0, 0.36);
    gap: 10px;
    justify-content: space-between;
  }

  .control-btn {
    width: 52px;
    height: 52px;
  }

  .mode-section {
    flex-direction: column;
    gap: 10px;
  }

  .mode-btn {
    padding: 12px;
  }

  .mode-icon {
    width: 38px;
    height: 38px;
    border-radius: 10px;
  }

  .mode-title {
    font-size: 13px;
  }

  .mode-desc {
    font-size: 11px;
  }

  .action-section {
    flex-direction: column;
    gap: 10px;
  }

  .action-btn {
    height: 46px;
  }
}
</style>

<style>
.main-dialog .el-select .el-input__wrapper {
  background: rgba(255, 255, 255, 0.04) !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
  box-shadow: none !important;
  border-radius: 8px !important;
}

.main-dialog .el-select .el-input__inner {
  color: #fff !important;
}

.main-dialog .el-select-dropdown {
  background: #1a1a26 !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
  border-radius: 12px !important;
  box-shadow: 0 16px 32px rgba(0, 0, 0, 0.4) !important;
}

.main-dialog .el-select-dropdown__item {
  color: rgba(255, 255, 255, 0.8) !important;
  padding: 10px 16px !important;
}

.main-dialog .el-select-dropdown__item.hover,
.main-dialog .el-select-dropdown__item:hover {
  background: rgba(102, 126, 234, 0.15) !important;
}

.main-dialog .el-select-dropdown__item.selected {
  color: #667eea !important;
  font-weight: 600 !important;
}

.main-dialog .el-popper.is-light {
  background: #1a1a26 !important;
  border-color: rgba(255, 255, 255, 0.1) !important;
}

.main-dialog .el-popper .el-popper__arrow::before {
  background: #1a1a26 !important;
  border-color: rgba(255, 255, 255, 0.1) !important;
}

.el-overlay {
  background: rgba(0, 0, 0, 0.75) !important;
}

.el-select-dropdown__empty {
  background: #1a1a26 !important;
  color: rgba(255, 255, 255, 0.6) !important;
}

.el-input__wrapper {
  background: rgba(255, 255, 255, 0.04) !important;
  box-shadow: none !important;
}

.el-input__inner {
  color: #fff !important;
}

.el-input__inner::placeholder {
  color: rgba(255, 255, 255, 0.3) !important;
}

.el-select__caret {
  color: rgba(255, 255, 255, 0.4) !important;
}

@media (max-width: 768px) {
  .main-dialog {
    width: calc(100vw - 20px) !important;
    max-width: 560px;
  }

  .main-dialog .el-dialog {
    width: 100% !important;
    margin-top: 8vh !important;
    border-radius: 16px !important;
  }

  .main-dialog .el-dialog__header {
    padding: 20px 16px 0 16px !important;
  }

  .main-dialog .el-dialog__body {
    padding: 14px 16px 18px !important;
  }

  .main-dialog .header-icon {
    width: 52px;
    height: 52px;
    border-radius: 14px;
    margin-bottom: 12px;
  }

  .main-dialog .dialog-header {
    margin-bottom: 16px;
  }

  .main-dialog .dialog-header h2 {
    font-size: 18px;
  }
}
</style>