import { Hono } from 'hono'
import { WebSocketServer } from 'ws'
import { serve } from '@hono/node-server'

const app = new Hono()
const rooms = new Map()
const waitingUsers = []

const PORT = 8083 

app.get('/', (c) => c.text('Node WSS 服务运行中'))

const server = serve({
  fetch: app.fetch,
  port: PORT, 
})

console.log(`🚀 WebSocketServer 运行:${PORT}`)

const wss = new WebSocketServer({ server })

wss.on('connection', (ws) => {
  ws.on('message', (data) => {
    const msg = JSON.parse(data.toString())
    console.log('Received:', msg.type)

    switch (msg.type) {
      case 'join':
        handleJoin(ws, msg)
        break
      case 'random_match':
        handleRandomMatch(ws, msg)
        break
      case 'leave':
        handleLeave(ws, msg)
        break
      case 'offer':
      case 'answer':
      case 'ice_candidate':
      case 'screen_share_start':
      case 'screen_share_stop':
        forwardToRoom(ws, msg)
        break
    }
  })

  ws.on('close', () => {
    if (ws.roomId) {
      handleLeave(ws, { roomId: ws.roomId, userId: ws.userId })
    }
    const idx = waitingUsers.findIndex(u => u.ws === ws)
    if (idx !== -1) waitingUsers.splice(idx, 1)
  })
})

function handleJoin(ws, msg) {
  const { roomId, userId } = msg
  ws.userId = userId
  ws.roomId = roomId

  if (!rooms.has(roomId)) {
    rooms.set(roomId, new Map())
  }
  const room = rooms.get(roomId)
  room.set(userId, ws)

  console.log(`User ${userId} joined room ${roomId}, users in room: ${room.size}`)

  if (room.size === 2) {
    const users = Array.from(room.keys())
    const otherId = users.find(id => id !== userId)
    if (otherId) {
      room.get(otherId).send(JSON.stringify({ type: 'peer_joined', userId }))
    }
    ws.send(JSON.stringify({ type: 'ready' }))
  } else {
    ws.send(JSON.stringify({ type: 'waiting' }))
  }
}

function handleRandomMatch(ws, msg) {
  const { userId } = msg
  ws.userId = userId

  if (waitingUsers.length === 0) {
    waitingUsers.push({ ws, userId })
    ws.send(JSON.stringify({ type: 'waiting_for_match' }))
  } else {
    const other = waitingUsers.pop()
    const roomId = 'match_' + Date.now()

    rooms.set(roomId, new Map([[userId, ws], [other.userId, other.ws]]))
    ws.roomId = roomId
    other.ws.roomId = roomId

    ws.send(JSON.stringify({ type: 'match_found', roomId, isInitiator: true }))
    other.ws.send(JSON.stringify({ type: 'match_found', roomId, isInitiator: false }))
  }
}

function handleLeave(ws, msg) {
  const { roomId, userId } = msg
  if (!roomId || !rooms.has(roomId)) return

  const room = rooms.get(roomId)
  room.delete(userId)

  if (room.size === 0) {
    rooms.delete(roomId)
  } else {
    room.forEach((client) => {
      if (client.readyState === 1) {
        client.send(JSON.stringify({ type: 'peer_left', userId }))
      }
    })
  }
}

function forwardToRoom(ws, msg) {
  if (!ws.roomId || !rooms.has(ws.roomId)) return

  const room = rooms.get(ws.roomId)
  room.forEach((client) => {
    if (client !== ws && client.readyState === 1) {
      client.send(JSON.stringify(msg))
    }
  })
}

process.on('SIGINT', () => {
  console.log('\n正在关闭服务器...')
  wss.close()
  server.close()
  process.exit(0)
})