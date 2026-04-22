# seeStrangers

基于 WebRTC 的视频聊天项目，支持随机匹配、房间通话和屏幕共享。后端已从 Node.js 迁移为 Rust（Axum WebSocket）。

## 在线体验

https://mojian.kongdf.com/

## 技术栈

- 前端：Vue 3 + Vite + Element Plus
- 后端：Rust + Axum（WebSocket 信令）

## 本地开发

### 启动后端（3101）

```bash
cd backend
cargo run
```

### 启动前端（3000）

```bash
cd frontend
npm install
npm run dev
```

## 打包

### 前端

```bash
cd frontend
npm install
npm run build
```

产物：`frontend/dist`

### 后端（macOS 本地编译 Linux，可部署 CentOS）

```bash
brew install zig
cargo install cargo-zigbuild
rustup target add x86_64-unknown-linux-musl
cd backend
cargo zigbuild --release --target x86_64-unknown-linux-musl
```

产物：`backend/target/x86_64-unknown-linux-musl/release/webrtc-backend`

## 部署到 CentOS

### 1) 上传产物

```bash
scp backend/target/x86_64-unknown-linux-musl/release/webrtc-backend root@<SERVER_IP>:/opt/server/seeStrangers/webrtc-backend
scp -r frontend/dist root@<SERVER_IP>:/opt/server/seeStrangers/frontend/
```

### 2) 配置 systemd

创建 `/etc/systemd/system/seestrangers.service`：

```ini
[Unit]
Description=seeStrangers Rust Backend
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/server/seeStrangers
ExecStart=/opt/server/seeStrangers/webrtc-backend
Restart=always
RestartSec=3
User=root

[Install]
WantedBy=multi-user.target
```

启动：

```bash
chmod +x /opt/server/seeStrangers/webrtc-backend
systemctl daemon-reload
systemctl enable --now seestrangers.service
systemctl restart seestrangers.service
systemctl status seestrangers.service
```
 
记得配nginx反代

## 常见问题

- `Exec format error`：上传了错误平台二进制（例如 Mach-O），Linux 需要 ELF。
- `status=203/EXEC`：`ExecStart` 路径不对，或文件不可执行。
- WebSocket 失败：检查前端 `WS_URL`、Nginx Upgrade 头、后端 `3101` 是否在运行。
