ALTER TABLE user_color_palettes
    ADD COLUMN background_seed_hex TEXT,
    ADD COLUMN text_seed_hex TEXT,
    ADD COLUMN primary_seed_hex TEXT,
    ADD COLUMN secondary_seed_hex TEXT;

UPDATE user_color_palettes
SET
    background_seed_hex = '#a9b1d6',
    text_seed_hex = '#1a1b26',
    primary_seed_hex = green_seed_hex,
    secondary_seed_hex = blue_seed_hex
WHERE
    background_seed_hex IS NULL
    OR text_seed_hex IS NULL
    OR primary_seed_hex IS NULL
    OR secondary_seed_hex IS NULL;

ALTER TABLE user_color_palettes
    ALTER COLUMN background_seed_hex SET NOT NULL,
    ALTER COLUMN text_seed_hex SET NOT NULL,
    ALTER COLUMN primary_seed_hex SET NOT NULL,
    ALTER COLUMN secondary_seed_hex SET NOT NULL;

UPDATE user_color_palettes
SET
    generated_tokens = generated_tokens || jsonb_build_object(
        'background', COALESCE(generated_tokens ->> 'white', '169, 177, 214'),
        'text', COALESCE(generated_tokens ->> 'black', '26, 27, 38'),
        'primary_100', COALESCE(generated_tokens ->> 'green_100', '158, 206, 106'),
        'primary_80', COALESCE(generated_tokens ->> 'green_80', '177, 216, 136'),
        'primary_60', COALESCE(generated_tokens ->> 'green_60', '197, 226, 166'),
        'secondary_100', COALESCE(generated_tokens ->> 'blue_100', '122, 162, 247'),
        'secondary_80', COALESCE(generated_tokens ->> 'blue_80', '149, 181, 249'),
        'secondary_60', COALESCE(generated_tokens ->> 'blue_60', '175, 199, 250')
    ),
    generation_version = GREATEST(generation_version, 2);
