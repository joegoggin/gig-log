# Claude Code Guidelines

## Component Documentation

All React components in `/web/src/components` and `/web/src/layouts` should follow these JSDoc documentation conventions.

### Components with Props

Components that accept props should include:
1. A description of the component
2. A "Props:" section listing each prop with a description
3. An `@example` section showing usage

```tsx
/**
 * A reusable button component with support for multiple variants and navigation.
 *
 * Props:
 * - `className` - Additional CSS class names to apply to the button
 * - `type` - HTML button type attribute (default: "button")
 * - `href` - URL to navigate to when button is clicked
 * - `onClick` - Click event handler
 * - `variant` - Visual style variant of the button (default: PRIMARY)
 * - `children` - Content to render inside the button
 *
 * @example
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

All route files in `/web/src/routes` should follow these conventions.

### Route Components

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
