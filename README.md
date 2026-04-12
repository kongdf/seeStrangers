# seeStrangers

一个基于 WebRTC 的视频通话应用，支持与陌生人进行 P2P 视频聊天，包含随机匹配、房间通话和屏幕共享功能。

## 在线体验

https://mojian.kongdf.com/

## 技术栈

- **前端**: Vue 3, Element Plus, Vite
- **后端**: Hono, WebSocket (ws), @hono/node-server

## 快速开始

### 环境要求

- Node.js (推荐 v18+)

### 安装

```bash
# 安装后端依赖
cd backend
npm install

# 安装前端依赖
cd ../frontend
npm install
```

### 运行

```bash
# 启动后端服务 (端口 8083)
cd backend
npm start

# 新开终端，启动前端开发服务器
cd frontend
npm run dev
```

## 功能特性

- 随机匹配用户进行视频通话
- 房间模式视频通话
- 屏幕共享
- WebRTC P2P 直连
- WebSocket 实时信令
- TURN 服务器支持 (备用)

## TURN 服务器配置

WebRTC 默认优先使用 P2P 直连，当无法直连时会自动回退到 TURN 中继服务器。

> 项目中的 TURN 服务器配置我没隐藏,每个月只有500M的流量,请在使用时注意流量消耗。
 
