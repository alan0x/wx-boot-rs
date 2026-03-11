# wx-boot-rs

A bootstrap template for WeChat mini-program backends built with Rust.

## Tech Stack

- **Web framework**: [Salvo](https://salvo.rs/) 0.37
- **ORM**: [Diesel](https://diesel.rs/) 2.1 + PostgreSQL
- **Auth**: JWT (via `salvo` jwt-auth feature) + WeChat OAuth
- **Cache**: Redis
- **Email**: Lettre
- **Async runtime**: Tokio

## Project Structure

```
src/
├── main.rs           # Entry point, server bootstrap
├── shared.rs         # Global constants, helpers, token utilities
├── macros.rs         # Shared macros (list_records!, get_record_by_param!, etc.)
├── error.rs          # Unified error type
├── context.rs        # Request context helpers (current_user, render_* helpers)
├── db/               # Database connection pool, delete helpers
├── models/           # Diesel model structs and static filter/search definitions
├── schema.rs         # Diesel schema (auto-generated)
├── routers/          # Route handlers
│   ├── auth.rs       # Login, register, JWT token issuance
│   ├── account.rs    # Account self-management (profile, avatar, password, email)
│   ├── user/         # User management (admin-facing)
│   ├── oauth.rs      # WeChat OAuth exchange
│   ├── order.rs      # Order creation and payment notify
│   ├── label.rs      # Custom labels (user-owned + system labels)
│   ├── notification.rs
│   ├── help_ticket.rs
│   ├── interflow/    # File/stream interflow
│   └── home.rs       # Health check, user state
├── utils/            # Validators, filesystem helpers, etc.
├── things/           # Background tasks / scheduled jobs
├── helpers/          # Shared handler helper functions
├── email.rs          # Email template rendering (Handlebars)
└── redis.rs          # Redis client wrapper
migrations/           # Diesel migrations
conf/                 # Environment config files
```

## Getting Started

### Prerequisites

- Rust (stable)
- PostgreSQL
- Redis
- `diesel_cli` — `cargo install diesel_cli --no-default-features --features postgres`

### Setup

1. Copy the example env file and fill in your values:

   ```bash
   cp conf/dev.toml.example conf/dev.toml
   ```

   Required environment variables:

   | Variable | Description |
   |---|---|
   | `DATABASE_URL` | PostgreSQL connection string |
   | `DATABASE_CONNS` | Connection pool size |
   | `REDIS_URL` | Redis connection string |
   | `SECRET_KEY` | JWT signing secret |
   | `SUPER_AUTH_TOKEN` | Super admin token |
   | `COOKIE_DOMAIN` | Cookie domain |
   | `SPACE_PATH` | Local file storage root |
   | `WECHAT_MP_APPID` | WeChat mini-program App ID |
   | `WECHAT_MP_SECRET` | WeChat mini-program App Secret |

2. Run migrations:

   ```bash
   diesel migration run
   ```

3. Start the server:

   ```bash
   cargo run
   ```

## Key Conventions

- All authenticated routes are mounted under a JWT auth hoop. Use `current_user!(depot, res)` to get the current user.
- Kernel users (`in_kernel = true`) have full access; regular users are subject to ownership checks.
- Active membership is checked via `is_active_member(&user)` — a promotion period override exists until **2026-03-03**.
- Custom labels must be 1–4 Chinese characters (`validate_custom_label_name`).
- Passwords require mixed case + digits, minimum 8 characters.

## License

MIT
