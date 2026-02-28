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
    UNIQUE(user_id, name),
    UNIQUE(id, user_id)
);

CREATE TABLE user_appearance_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE UNIQUE,
    active_palette_type palette_type NOT NULL,
    active_preset_palette preset_palette,
    active_custom_palette_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_active_custom_palette_owner
        FOREIGN KEY (active_custom_palette_id, user_id)
        REFERENCES user_color_palettes(id, user_id),
    CONSTRAINT chk_active_palette_selection CHECK (
        (active_palette_type = 'preset'
         AND active_preset_palette IS NOT NULL
         AND active_custom_palette_id IS NULL)
        OR
        (active_palette_type = 'custom'
         AND active_preset_palette IS NULL
         AND active_custom_palette_id IS NOT NULL)
    )
);
