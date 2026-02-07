// This file has been automatically migrated to valid ESM format by Storybook.
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
import type { StorybookConfig } from '@storybook/react-vite';
import type { PluginOption } from 'vite';

const __dirname = dirname(fileURLToPath(import.meta.url));

const hasPluginName = (plugin: PluginOption): plugin is { name: string } => {
  if (!plugin || Array.isArray(plugin) || plugin instanceof Promise) {
    return false;
  }

  return (
    typeof plugin === 'object' &&
    'name' in plugin &&
    typeof plugin.name === 'string'
  );
};

const config: StorybookConfig = {
  "stories": [
    "../src/**/*.mdx",
    "../src/**/*.stories.@(js|jsx|mjs|ts|tsx)"
  ],
  "addons": [
    "@chromatic-com/storybook",
    "@storybook/addon-vitest",
    "@storybook/addon-a11y",
    "@storybook/addon-docs",
    "@storybook/addon-onboarding",
    "@storybook/addon-themes"
  ],
  "framework": "@storybook/react-vite",
  viteFinal: async (config) => {
    // Filter out TanStack devtools plugins to avoid port conflict with dev server
    config.plugins = (config.plugins || []).flat().filter((plugin) => {
      if (!hasPluginName(plugin)) {
        return true;
      }

      return !plugin.name.startsWith('@tanstack/devtools');
    });

    config.resolve = config.resolve || {};
    config.resolve.alias = {
      ...config.resolve.alias,
      '@': resolve(__dirname, '../src'),
      '@sass': resolve(__dirname, '../src/sass'),
    };

    config.optimizeDeps = config.optimizeDeps || {};
    config.optimizeDeps.include = [
      ...(config.optimizeDeps.include || []),
      "@tanstack/react-query",
    ];
    return config;
  },
};
export default config;
