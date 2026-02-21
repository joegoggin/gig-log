CREATE TABLE user_color_palettes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    green_seed_hex TEXT NOT NULL,
    red_seed_hex TEXT NOT NULL,
    yellow_seed_hex TEXT NOT NULL,
    blue_seed_hex TEXT NOT NULL,
    magenta_seed_hex TEXT NOT NULL,
    cyan_seed_hex TEXT NOT NULL,
    generated_tokens JSONB NOT NULL,
    generation_version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_user_color_palettes_user_lower_name
    ON user_color_palettes (user_id, LOWER(name));

CREATE INDEX idx_user_color_palettes_user_created_at
    ON user_color_palettes (user_id, created_at DESC);

CREATE TABLE user_appearance_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    active_palette_type TEXT NOT NULL DEFAULT 'preset',
    active_preset_palette TEXT,
    active_custom_palette_id UUID REFERENCES user_color_palettes(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT check_active_palette_type
        CHECK (active_palette_type IN ('preset', 'custom')),
    CONSTRAINT check_user_appearance_palette_selection
        CHECK (
            (active_palette_type = 'preset'
                AND active_preset_palette IN ('default', 'sunset', 'forest')
                AND active_custom_palette_id IS NULL)
            OR
            (active_palette_type = 'custom'
                AND active_custom_palette_id IS NOT NULL
                AND active_preset_palette IS NULL)
        )
);
