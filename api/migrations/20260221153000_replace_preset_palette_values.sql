ALTER TABLE user_appearance_preferences
    DROP CONSTRAINT IF EXISTS check_user_appearance_palette_selection;

UPDATE user_appearance_preferences
SET
    active_preset_palette = CASE active_preset_palette
        WHEN 'default' THEN 'tokyo-night'
        WHEN 'sunset' THEN 'catppuccin'
        WHEN 'forest' THEN 'everforest'
        ELSE active_preset_palette
    END,
    updated_at = NOW()
WHERE
    active_palette_type = 'preset'
    AND active_preset_palette IN ('default', 'sunset', 'forest');

ALTER TABLE user_appearance_preferences
    ADD CONSTRAINT check_user_appearance_palette_selection
        CHECK (
            (active_palette_type = 'preset'
                AND active_preset_palette IN ('catppuccin', 'tokyo-night', 'everforest')
                AND active_custom_palette_id IS NULL)
            OR
            (active_palette_type = 'custom'
                AND active_custom_palette_id IS NOT NULL
                AND active_preset_palette IS NULL)
        );
