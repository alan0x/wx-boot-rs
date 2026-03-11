# wx-boot-rs

微信小程序后端脚手架，基于 Rust 构建。提供开箱即用的用户体系、微信鉴权、支付接入点、通知、文件上传等基础能力，适合作为新项目的起点。

## 技术栈

| 层 | 选型 |
|---|---|
| Web 框架 | [Salvo](https://salvo.rs/) 0.37 |
| ORM | [Diesel](https://diesel.rs/) 2.1 + PostgreSQL |
| 认证 | JWT（salvo jwt-auth） |
| 缓存 | Redis |
| 邮件 | Lettre + Handlebars 模板 |
| 异步运行时 | Tokio |

---

## 快速开始

### 前置依赖

- Rust stable
- PostgreSQL
- Redis
- Diesel CLI：`cargo install diesel_cli --no-default-features --features postgres`

### 1. 克隆并配置环境变量

复制示例并填写实际值：

```bash
cp .env.example .env
```

| 变量 | 说明 |
|---|---|
| `DATABASE_URL` | PostgreSQL 连接串，如 `postgres://user:pass@localhost/mydb` |
| `DATABASE_CONNS` | 连接池大小，推荐 `5` |
| `REDIS_URL` | Redis 连接串，如 `redis://127.0.0.1/` |
| `SECRET_KEY` | JWT 签名密钥，随机长字符串 |
| `SUPER_AUTH_TOKEN` | 超管静态 token，用于运维脚本直接访问 |
| `COOKIE_DOMAIN` | Cookie 域名，如 `example.com` |
| `SPACE_PATH` | 本地文件存储根目录，如 `/data/space` |
| `WECHAT_MP_APPID` | 微信小程序 AppID |
| `WECHAT_MP_SECRET` | 微信小程序 AppSecret |

### 2. 初始化数据库

```bash
diesel migration run
```

### 3. 启动

```bash
cargo run
```

服务默认监听 `0.0.0.0:8080`（见 `src/main.rs`）。

---

## 核心流程说明

### 微信小程序鉴权

小程序不使用账号密码，走静默注册 + 登录一体化流程：

```
小程序端                          后端
  │                                │
  │  wx.login() → code             │
  │─────────────────────────────►  │
  │  POST /auth/weixin_account_    │
  │       create_and_login         │
  │  { code }                      │
  │                                │  调用微信 jscode2session 接口
  │                                │  GET api.weixin.qq.com/sns/jscode2session
  │                                │    → openid, session_key
  │                                │
  │                                │  openid 已存在 → 直接登录
  │                                │  openid 不存在 → 自动注册新用户
  │                                │    ident_name: UUID
  │                                │    display_name: 微信用户-xxxx
  │                                │    password: openid（不对外暴露）
  │                                │
  │  { token, user }               │
  │◄─────────────────────────────  │
```

**响应示例：**
```json
{
  "token": "eyJ...",
  "user": {
    "id": 42,
    "display_name": "微信用户-Ab3x",
    "weixin_openid": "o6_xxx",
    ...
  }
}
```

后续所有请求在 Header 中携带 token：
```
Authorization: Bearer eyJ...
```

---

### 管理后台鉴权（账密登录）

管理前端使用标准账密登录，不走微信鉴权：

```
POST /auth/login
Content-Type: application/json

{
  "user": "admin@example.com",   // 支持邮箱或 ident_name
  "password": "YourPassword123"
}
```

**响应：**
```json
{
  "token": "eyJ...",
  "user": { ... },
  "error": null
}
```

管理员用户的 `in_kernel` 字段为 `true`，所有需要管理权限的接口会校验此字段。

**其他鉴权接口：**

| 方法 | 路径 | 说明 |
|---|---|---|
| `POST` | `/auth/logout` | 注销当前 token（需认证） |
| `POST` | `/auth/refresh_token` | 刷新 token（需认证） |

---

### 微信支付接入

脚手架已内置订单模型和支付流程的骨架，你只需在两处填入微信支付 SDK 调用：

#### 第一步：创建订单时调起支付

文件：`src/routers/order.rs` → `create` handler

```rust
// TODO: Integrate your payment provider here (e.g., WeChat Pay)
// After payment confirmation, update order trade_state to "SUCCESS"
```

在此处调用微信支付 `jsapi` 或 `native` 下单接口，将返回的 `prepay_id` 等参数透传给小程序端，由小程序调用 `wx.requestPayment()`。

典型流程：

```
小程序                            后端                         微信支付
  │                                │                              │
  │  POST /orders                  │                              │
  │  { reason, description }       │                              │
  │──────────────────────────────► │                              │
  │                                │  POST /v3/pay/transactions/jsapi
  │                                │─────────────────────────────►│
  │                                │  ◄── { prepay_id, ... }      │
  │  { order_id, prepay_params }   │                              │
  │◄────────────────────────────── │                              │
  │                                │                              │
  │  wx.requestPayment(prepay_params)                             │
  │──────────────────────────────────────────────────────────────►│
  │  ◄── 支付结果回调                                              │
```

#### 第二步：接收微信支付回调

文件：`src/routers/order.rs` → `notify` handler（路由：`POST /orders/notify`，公开路由，无需鉴权）

```rust
// TODO: Implement payment notification handler for your payment provider
// This endpoint receives callbacks from the payment provider (e.g., WeChat Pay)
// After verifying the payment, update the order status accordingly
```

验签通过后，调用 `things::order::update_user_by_order` 更新订单状态和用户会员信息。该函数已实现月度/年度/永久会员的到期时间计算逻辑，可直接复用。

**订单状态流转：**
```
NEW → SUCCESS（支付成功）
    → CLOSED（超时或关闭）
```

---

## API 路由总览

### 公开路由（无需 token）

| 方法 | 路径 | 说明 |
|---|---|---|
| `POST` | `/auth/login` | 账密登录 |
| `POST` | `/auth/weixin_account_create_and_login` | 微信小程序静默注册 + 登录 |
| `POST` | `/account/create` | 邮箱注册 |
| `POST` | `/account/reset_password` | 重置密码 |
| `POST` | `/account/send_security_code` | 发送安全验证码 |
| `POST` | `/orders/notify` | 支付回调（供支付服务商调用） |
| `GET`  | `/users/is_other_taken` | 检查用户名/邮箱是否被占用 |
| `GET`  | `/health` | 健康检查 |

### 需认证路由（需在 Header 携带 `Authorization: Bearer <token>`）

| 资源 | 说明 |
|---|---|
| `PATCH /account` | 更新个人资料 |
| `POST /account/update_password` | 修改密码 |
| `GET/POST /account/notifications` | 通知列表 / 标记已读 |
| `GET/POST /orders` | 订单列表 / 创建订单 |
| `POST /orders/calc_amount` | 预计算金额（含折扣） |
| `GET /account/help_tickets` | 工单列表 |
| `GET/POST /users` | 用户列表 / 管理（需 `in_kernel`） |
| `POST /auth/logout` | 注销 |
| `POST /auth/refresh_token` | 刷新 token |

---

## 项目结构

```
src/
├── main.rs              # 启动入口
├── shared.rs            # 全局常量、token 工具函数
├── macros.rs            # 通用宏（list_records! 等）
├── error.rs             # 统一错误类型
├── context.rs           # 请求上下文（current_user、render_* 帮助函数）
├── routers/
│   ├── auth.rs          # 登录、微信鉴权、登出、刷新 token
│   ├── account.rs       # 用户自身操作（改密、头像、通知等）
│   ├── user/            # 用户管理（管理员视角）
│   ├── order.rs         # 订单 + 支付回调骨架
│   ├── notification.rs  # 通知
│   ├── help_ticket.rs   # 工单
│   └── home.rs          # 健康检查、日志查看
├── models/              # Diesel 模型结构体
├── things/              # 业务逻辑（订单金额计算、会员更新等）
├── db/                  # 连接池、级联删除
├── utils/               # 验证器、文件操作、密码哈希
├── email.rs             # 邮件发送（Handlebars 模板）
└── schema.rs            # Diesel schema（自动生成，勿手动修改）
migrations/
conf/
└── emails/
    └── layout.hbs       # 邮件 HTML 外壳模板
```

---

## 扩展指引

- **添加新业务模块**：在 `src/routers/` 新建文件，在 `routers.rs` 的 `root()` 中 `.push()` 挂载
- **添加新数据表**：`diesel migration generate <name>` 编写 SQL，`diesel migration run`，再运行 `diesel print-schema` 更新 `schema.rs`
- **权限控制**：在 handler 内用 `current_user!(depot, res)` 获取当前用户，通过 `cuser.in_kernel` 判断是否管理员
- **实现支付**：参考上方「微信支付接入」章节，填写 `order.rs` 中标注 `TODO` 的两处

## License

MIT
