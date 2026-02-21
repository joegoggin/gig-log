import type {
    ColorPalette,
    PaletteRgbTokens,
    PaletteSeedHexColors,
} from "@/lib/appearance";

export type ActivePaletteSelection = {
    palette_type: "preset" | "custom";
    preset_palette: ColorPalette | null;
    custom_palette_id: string | null;
};

export type CustomPalette = {
    id: string;
    name: string;
    green_seed_hex: string;
    red_seed_hex: string;
    yellow_seed_hex: string;
    blue_seed_hex: string;
    magenta_seed_hex: string;
    cyan_seed_hex: string;
    generated_tokens: PaletteRgbTokens;
    generation_version: number;
    created_at: string;
    updated_at: string;
};

export type AppearanceSettingsResponse = {
    active_palette: ActivePaletteSelection;
    custom_palettes: Array<CustomPalette>;
};

export type CreateCustomPaletteRequest = {
    name: string;
} & PaletteSeedHexColors;

export type CreateCustomPaletteResponse = {
    message: string;
    palette: CustomPalette;
    active_palette: ActivePaletteSelection;
};

export type SetActivePaletteRequest =
    | {
          palette_type: "preset";
          preset_palette: ColorPalette;
      }
    | {
          palette_type: "custom";
          custom_palette_id: string;
      };

export type SetActivePaletteResponse = {
    message: string;
    active_palette: ActivePaletteSelection;
};
