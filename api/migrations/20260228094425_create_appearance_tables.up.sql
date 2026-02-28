CREATE TYPE palette_type AS ENUM ('preset', 'custom');
CREATE TYPE preset_palette AS ENUM ('catppuccin', 'tokyo-night', 'everforest');

CREATE TABLE user_color_palettes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    seed_green VARCHAR NOT NULL,
    seed_red VARCHAR NOT NULL,
    seed_yellow VARCHAR NOT NULL,
    seed_blue VARCHAR NOT NULL,
    seed_magenta VARCHAR NOT NULL,
    seed_cyan VARCHAR NOT NULL,
    generated_tokens JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(user_id, name)
);

CREATE TABLE user_appearance_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE UNIQUE,
    active_palette_type palette_type NOT NULL,
    active_preset_palette preset_palette,
    active_custom_palette_id UUID REFERENCES user_color_palettes(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
