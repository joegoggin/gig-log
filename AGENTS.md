# AI Agent Guidelines

## Documentation Policy

When asked to add documentation to React components or layouts, **always include Storybook stories** as part of the documentation effort. This means:
1. JSDoc comments on the component (as described below)
2. A corresponding `.stories.tsx` file with visual examples

For route components, use MDX files instead of `.stories.tsx` files (see Route Documentation section).

## Component Documentation

All React components in `/web/src/components` and `/web/src/layouts` should follow these JSDoc documentation conventions.

### Components with Props

Components that accept props should include:
1. A description of the component
2. A `## Props` section listing each prop with a description
3. A `## Example` section showing usage

```tsx
/**
 *
 * A reusable button component with support for multiple variants and navigation.
 *
 * ## Props
 *
 * - `className` - Additional CSS class names to apply to the button
 * - `type` - HTML button type attribute (default: "button")
 * - `href` - URL to navigate to when button is clicked
 * - `onClick` - Click event handler
 * - `variant` - Visual style variant of the button (default: PRIMARY)
 * - `children` - Content to render inside the button
 *
 * ## Example
 *
 * ```tsx
 * <Button variant={ButtonVariant.PRIMARY} onClick={handleClick}>
 *   Click Me
 * </Button>
 * ```
 */
```

### Components without Props

Components without props (such as icon components) should include only a description:

```tsx
/**
 * A plus icon inside a circle, commonly used for add/create actions.
 */
```

### Type Definitions

Props types should have inline comments for each property:

```tsx
type ButtonProps = {
    /** Additional CSS class names to apply to the button */
    className?: string;
    /** HTML button type attribute */
    type?: "submit" | "button" | "reset";
};
```

### Enums

Enums should have a description and inline comments for each value:

```tsx
/**
 * Enum representing the available button style variants.
 */
export enum ButtonVariant {
    /** Primary button style with prominent styling */
    PRIMARY,
    /** Secondary button style with subtle styling */
    SECONDARY,
}
```

## Route Documentation

Routes should have both JSDoc comments on the component and an MDX file in Storybook. **Routes should NOT have `.stories.tsx` files** - only MDX documentation.

### JSDoc Comments

Documentation should be placed on the component function (not at the file level). The component should be named descriptively (e.g., `HomePage` instead of `App`).

Routes with a loader should include:
1. A description of the page
2. A "Route:" section with the path
3. A "Loader Data:" section listing each field returned by the loader

```tsx
/**
 * The home page and landing page for the application.
 * Displays an introduction to GigLog and provides navigation to
 * sign up, log in, or access the dashboard for authenticated users.
 *
 * Route: `/`
 *
 * Loader Data:
 * - `message` - The welcome message from the API
 */
function HomePage() {
```

Routes without a loader should include:
1. A description of the page
2. A "Route:" section with the path

```tsx
/**
 * The user settings page where users can update their profile
 * and application preferences.
 *
 * Route: `/settings`
 */
function SettingsPage() {
```

### MDX Files

Each route should also have an MDX file in `web/src/stories/docs/Routes/`:

```mdx
import { Meta } from "@storybook/addon-docs/blocks";

<Meta title="Routes/PageName" />

# Page Name

Description of what the page does and its purpose.

**Route:** `/path`

## States

- **State 1:** Description of this state
- **State 2:** Description of this state

## Related Components

- `ComponentName` - How it's used on this page
```

See `web/src/stories/docs/Routes/HomePage.mdx` for a complete example

## Storybook Documentation

When adding documentation to components and layouts, **always include Storybook stories** as part of the documentation. Stories serve as living documentation and visual testing.

### Configuration

Storybook is configured in `web/.storybook/` with:
- `@storybook/react-vite` framework
- Dark theme by default
- Addons: `@storybook/addon-docs`, `@storybook/addon-a11y`, `@storybook/addon-vitest`, `@chromatic-com/storybook`
- Path aliases: `@` → `src/`, `@sass` → `src/sass/`

### Story File Location

Stories are co-located with their components:
- Components: `web/src/components/core/ComponentName/ComponentName.stories.tsx`
- Layouts: `web/src/layouts/LayoutName/LayoutName.stories.tsx`
- Icons: `web/src/components/icons/Icons.stories.tsx` (single gallery file for all icons)

### Story File Structure

All stories use CSF3 format with types from `@storybook/react-vite`:

```tsx
import type { Meta, StoryObj } from "@storybook/react-vite";
import Button, { ButtonVariant } from "./Button";

const meta: Meta<typeof Button> = {
    title: "Core/Button",
    component: Button,
    tags: ["autodocs"],
    argTypes: {
        variant: {
            control: { type: "select" },
            options: [ButtonVariant.PRIMARY, ButtonVariant.SECONDARY],
        },
    },
};

export default meta;
type Story = StoryObj<typeof Button>;

export const Primary: Story = {
    args: {
        variant: ButtonVariant.PRIMARY,
        children: "Primary Button",
    },
};
```

### Title Hierarchy

Use these title prefixes for organization:
- `"Core/ComponentName"` - Core UI components (Button, Notification, etc.)
- `"Layouts/LayoutName"` - Layout components
- `"Icons/All Icons"` - Icon gallery

### Key Conventions

1. Always include `tags: ["autodocs"]` for automatic documentation generation
2. Use `argTypes` with `control: { type: "select" }` for enum props
3. Export multiple named stories to showcase different states/variants
4. For fullscreen layouts, add `parameters: { layout: "fullscreen" }` to meta
5. Story names should be descriptive: `Primary`, `Secondary`, `Default`, `WithClickHandler`, etc.

### Running Storybook

```bash
cd web && pnpm storybook
```

## Code Review Process

When asked to perform a code review, follow this interactive process:

### What to Check

- **Spelling mistakes** - Check for typos in code, comments, and strings
- **Documentation compliance** - Ensure all files follow the documentation formats defined in this file (JSDoc comments, Storybook stories, MDX files for routes, etc.)
- **Code quality issues** - Bugs, logic errors, and other problems

### Process

1. **Step through issues one at a time** - Do not provide all feedback in a single response
2. **For each issue found:**
   - Provide a clear description of the issue
   - Show a diff of the proposed fix
   - Ask the user whether to accept or reject the change
3. **Wait for user confirmation** before moving to the next issue
4. **After the user responds:**
   - If accepted: Apply the change and move to the next issue
   - If rejected: Skip the change and move to the next issue
5. **Continue until all issues have been addressed**
6. **After all issues are resolved:** Ask the user if they want to:
   - Commit the changes
   - Push to the remote branch
   - Create a PR with a summary of all the changes made during the review

### Example Format

For each issue, present it like this:

```
**Issue 1: [Brief title]**

[Description of the issue and why it should be changed]

**Proposed fix:**

\`\`\`diff
- old code
+ new code
\`\`\`

Do you want to accept this change?
```

## Component Creation

### SVG Icon Components

When converting an SVG into a React component, follow this format:

1. Place the file in `web/src/components/icons/`
2. Name the file `{IconName}Icon.tsx` (e.g., `HomeIcon.tsx`)
3. Use the following structure:

```tsx
/**
 * A brief description of what the icon represents and its common use case.
 */
const IconNameIcon: React.FC = () => {
    return (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            width="24"
            height="24"
            fill="#E0E0E0"
        >
            <path d="..." />
        </svg>
    );
};

export default IconNameIcon;
```

### Key Requirements

- **viewBox**: Use `"0 0 24 24"` for consistency (scale paths if needed)
- **width/height**: Always set to `"24"`
- **fill**: Use `"#E0E0E0"` as the default color
- **stroke-based icons**: If the SVG uses strokes instead of fills, set `stroke="#E0E0E0"` on the path elements and keep `fill="none"`
- **JSDoc**: Include a brief description of the icon's appearance and typical usage
- **Naming**: Component name should be `{IconName}Icon` and match the filename
