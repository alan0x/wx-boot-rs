-- Users
CREATE TABLE IF NOT EXISTS public.users (
    id bigserial PRIMARY KEY NOT NULL,
    ident_name character varying(255) NOT NULL,
    display_name character varying(255) NOT NULL,
    password character varying(255) NOT NULL,
    is_disabled boolean NOT NULL DEFAULT false,
    disabled_by bigint,
    disabled_at timestamp with time zone,
    is_verified boolean NOT NULL DEFAULT false,
    verified_at timestamp with time zone,
    updated_by bigint,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    in_kernel boolean NOT NULL DEFAULT false,
    weixin_openid character varying(255),
    profile jsonb NOT NULL DEFAULT '{}'::jsonb,
    avatar character varying(255),
    contribute bigint,
    enable_ranking boolean,
    latest_export timestamp with time zone,
    is_member boolean,
    expired_at timestamp with time zone
);

-- Emails
CREATE TABLE IF NOT EXISTS public.emails (
    id bigserial PRIMARY KEY NOT NULL,
    user_id bigint NOT NULL,
    value character varying(255) NOT NULL,
    domain character varying(255) NOT NULL,
    is_verified boolean NOT NULL DEFAULT false,
    updated_by bigint,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Access Tokens
CREATE TABLE IF NOT EXISTS public.access_tokens (
    id bigserial PRIMARY KEY NOT NULL,
    user_id bigint NOT NULL,
    name character varying(255),
    kind character varying(255) NOT NULL DEFAULT 'web',
    value character varying(255) NOT NULL,
    device character varying(255),
    expired_at timestamp with time zone NOT NULL,
    updated_by bigint,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Security Codes
CREATE TABLE IF NOT EXISTS public.security_codes (
    id bigserial PRIMARY KEY NOT NULL,
    user_id bigint NOT NULL,
    email character varying(255),
    value character varying(255) NOT NULL,
    send_method character varying(255) NOT NULL,
    consumed_at timestamp with time zone,
    expired_at timestamp with time zone NOT NULL,
    updated_by bigint,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Notifications
CREATE TABLE IF NOT EXISTS public.notifications (
    id bigserial PRIMARY KEY NOT NULL,
    owner_id bigint NOT NULL,
    sender_id bigint,
    subject character varying(255) NOT NULL DEFAULT '',
    body character varying NOT NULL DEFAULT '',
    kind character varying(50) NOT NULL DEFAULT 'general',
    is_read boolean NOT NULL DEFAULT false,
    extra jsonb NOT NULL DEFAULT '{}'::jsonb,
    updated_by bigint,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);


-- Help Tickets
CREATE TABLE IF NOT EXISTS public.help_tickets (
    id bigserial PRIMARY KEY NOT NULL,
    owner_id bigint NOT NULL,
    kind character varying(50) NOT NULL,
    subject character varying(200) NOT NULL,
    label_ids bigint[] NOT NULL DEFAULT '{}',
    is_recalled boolean NOT NULL DEFAULT false,
    content text NOT NULL DEFAULT '',
    updated_by bigint,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_resolved boolean NOT NULL DEFAULT false,
    extra jsonb,
    is_processed boolean NOT NULL DEFAULT false
);

-- Orders
CREATE TABLE IF NOT EXISTS public.orders (
    id bigserial PRIMARY KEY NOT NULL,
    order_id character varying(255) NOT NULL,
    paid_reason character varying(255) NOT NULL,
    amount numeric NOT NULL DEFAULT 0,
    trade_state character varying(50) NOT NULL DEFAULT 'NEW',
    payment_id character varying(255) NOT NULL DEFAULT '',
    paid_at timestamp with time zone,
    paid_by bigint,
    updated_by bigint,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    discount jsonb NOT NULL DEFAULT '{"discounts":[],"origin_amount":"0","actual_amount":"0"}'::jsonb,
    actual_amount numeric NOT NULL DEFAULT 0
);

-- User Last Login
CREATE TABLE IF NOT EXISTS public.user_last_login (
    id bigserial PRIMARY KEY NOT NULL,
    user_id bigint NOT NULL,
    last_login timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);