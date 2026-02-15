# AI Agent Guidelines

## Documentation Policy

When asked to add documentation to React components, layouts, or pages, **always include Storybook stories** as part of the documentation effort. This means:
1. JSDoc comments on the component (as described below)
2. A corresponding `.stories.tsx` file with visual examples

Always perform a documentation consistency check when making changes that affect behavior, workflows, scripts, architecture, testing expectations, or project status:
1. Compare the implementation and any newly added docs against `README.md`
2. Update `README.md` if anything has become outdated or contradictory
3. Ensure related docs (component docs, Storybook stories, Rustdoc comments, and route/payload docs) stay aligned with `README.md`
4. Flag and resolve conflicts instead of leaving mismatched documentation

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

## Page Documentation

Page components in `/web/src/pages` should follow the same documentation conventions as other components.

### JSDoc Comments

Documentation should be placed on the component function (not at the file level). The component should be named descriptively (e.g., `HomePage` instead of `App`).

Pages should include:
1. A description of the page
2. A `Route:` section with the path
3. A `## Props` section listing each prop with a description (or `None` if the page has no props)
4. A `## Related Components` section listing components used on the page

```tsx
/**
 * The home page and landing page for the application.
 * Displays an introduction to GigLog and provides navigation to
 * sign up, log in, or access the dashboard for authenticated users.
 *
 * Route: `/`
 *
 * ## Props
 *
 * - `isLoggedIn` - Whether the user is currently authenticated
 *
 * ## Related Components
 *
 * - `Button` - Used for navigation actions
 * - `FullscreenCenteredLayout` - Page layout wrapper
 */
function HomePage({ isLoggedIn }: HomePageProps) {
```

## Rust Model Documentation

All Rust model files in `/api/src/models/` should follow these documentation conventions using Rust doc comments.

### Module-Level Documentation

Each file should start with a module-level doc comment (`//!`) describing the file's purpose:

```rust
//! User model representing authenticated users of the application.
```

### Struct Documentation

Structs should have a doc comment describing what they represent and any important behavior:

```rust
/// Represents a registered user of the application.
///
/// Users can create companies, jobs, work sessions, and track payments.
/// The password is hashed and excluded from serialization for security.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
```

### Field Documentation

Every field should have an inline doc comment:

```rust
pub struct User {
    /// Unique identifier for the user.
    pub id: Uuid,
    /// User's email address (used for login).
    pub email: String,
    /// Hashed password (excluded from JSON serialization).
    #[serde(skip_serializing)]
    pub hashed_password: String,
}
```

### Enum Documentation

Enums should have a description and inline comments for each variant:

```rust
/// Defines how a job compensates the worker.
#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq)]
pub enum PaymentType {
    /// Paid based on hours worked at an hourly rate.
    Hourly,
    /// Paid in fixed amounts (payouts) regardless of time spent.
    Payouts,
}
```

### Conditional Fields

When a field is only applicable under certain conditions, document that relationship:

```rust
/// Hourly rate for the job. Only applicable when `payment_type` is `Hourly`.
pub hourly_rate: Option<Decimal>,
```

## Rust Route Documentation

All Rust route modules in `/api/src/routes/` should follow the structure established in the `auth/` module. Each route domain is a directory with handlers and payloads that cross-reference each other in documentation.

### Directory Structure

Each route domain should be a directory containing:

- `mod.rs` - Module-level docs and re-exports
- `handlers.rs` - HTTP handler functions
- `payloads.rs` - Request/response structs

```
api/src/routes/
├── mod.rs           # Top-level routes module
├── auth/            # Primary example - follow this structure
│   ├── mod.rs
│   ├── handlers.rs
│   └── payloads.rs
└── health/          # Special case - no payloads needed for simple JSON responses
    ├── mod.rs
    └── handlers.rs
```

**Note:** The `health/` module is a special case that doesn't require `payloads.rs` since it returns a simple inline JSON response. Most route modules should follow the `auth/` pattern with a dedicated `payloads.rs` file.

### Top-Level Module (`routes/mod.rs`)

Document the module purpose and list all route domains:

```rust
//! HTTP route handlers for the API.
//!
//! This module organizes all route handlers by domain:
//!
//! - [`auth`] - Authentication routes (sign-up, login, logout, password reset)
//! - [`health`] - Health check endpoint for monitoring

pub mod auth;
pub mod health;
```

### Route Module (`routes/{domain}/mod.rs`)

Include module-level docs describing the domain, list submodules, and re-export handlers and commonly-used payloads:

```rust
//! Authentication handlers for user registration, login, and password management.
//!
//! This module provides HTTP handlers for all authentication-related endpoints:
//! - User registration and email confirmation
//! - Login and logout with JWT tokens stored in HTTP-only cookies
//! - Password reset flow (forgot password, verify code, set new password)
//! - Current user retrieval for authenticated sessions
//!
//! # Module Structure
//!
//! - [`handlers`] - HTTP handler functions for authentication endpoints
//! - [`payloads`] - Request and response data structures

pub mod handlers;
pub mod payloads;

// Re-export handlers at module level for easy route registration
pub use handlers::{
    confirm_email, current_user, forgot_password, log_in, log_out, set_password, sign_up,
    verify_forgot_password,
};

// Re-export payload types that are used by other modules
pub use payloads::{SetPasswordRequest, SignUpRequest};
```

### Handler Documentation (`routes/{domain}/handlers.rs`)

Each handler function should include:

1. A description of what the handler does
2. `# Route` section with the HTTP method and path
3. `# Request Body` section linking to the payload type (if applicable)
4. `# Response Body` section linking to the payload type
5. `# Errors` section listing possible error cases

**Important:** Link to payload types using rustdoc syntax (e.g., `[`SignUpRequest`]`) to enable navigation between handlers and payloads.

```rust
//! HTTP handler functions for authentication endpoints.
//!
//! This module contains all the handler functions that process authentication
//! requests including user registration, login, logout, email confirmation,
//! and password management.

use super::payloads::{SignUpRequest, SignUpResponse};

/// Registers a new user account.
///
/// Creates a new user with the provided credentials, generates an email
/// confirmation code, and sends it to the user's email address.
///
/// # Route
///
/// `POST /auth/sign-up`
///
/// # Request Body ([`SignUpRequest`])
///
/// - `first_name` - User's first name
/// - `last_name` - User's last name
/// - `email` - User's email address (must be unique)
/// - `password` - User's chosen password (minimum 8 characters)
/// - `confirm` - Password confirmation (must match `password`)
///
/// # Response Body ([`SignUpResponse`])
///
/// - `message` - Success message instructing user to check email
/// - `user_id` - The newly created user's unique identifier
///
/// # Errors
///
/// - `EmailAlreadyExists` - If the email is already registered
/// - `InternalError` - If password hashing or database operations fail
#[post("/auth/sign-up")]
pub async fn sign_up(
    pool: web::Data<Pool<Postgres>>,
    env: web::Data<Env>,
    body: ValidatedJson<SignUpRequest>,
) -> ApiResult<HttpResponse> {
    // ...
}
```

### Payloads Documentation (`routes/{domain}/payloads.rs`)

Each payload struct should include:

1. A description of what the payload represents
2. A `See [handler_name](super::handlers::handler_name)` link back to the handler
3. Field-level documentation for each field

**Important:** Always link back to the handler that uses the payload using `super::handlers::handler_name` syntax.

```rust
//! Request and response payloads for authentication endpoints.
//!
//! This module contains all the data structures used for serializing and
//! deserializing HTTP request bodies and response payloads in the auth handlers.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Request body for user registration.
///
/// Validates that passwords match and meet minimum length requirements.
///
/// See [`sign_up`](super::handlers::sign_up) for the handler that processes this request.
#[derive(Debug, Deserialize, Validate)]
pub struct SignUpRequest {
    /// User's first name.
    #[validate(length(min = 1, message = "First name is required"))]
    pub first_name: String,

    /// User's last name.
    #[validate(length(min = 1, message = "Last name is required"))]
    pub last_name: String,

    /// User's email address (must be unique).
    #[validate(email(message = "Email is invalid"))]
    pub email: String,

    /// User's chosen password (minimum 8 characters).
    #[validate(length(min = 8, message = "Password must have at least 8 characters"))]
    pub password: String,

    /// Password confirmation (must match `password`).
    #[validate(length(min = 1, message = "Confirm password is required"))]
    pub confirm: String,
}

/// Response body for successful user registration.
///
/// See [`sign_up`](super::handlers::sign_up) for the handler that produces this response.
#[derive(Debug, Serialize)]
pub struct SignUpResponse {
    /// Success message instructing user to check email.
    pub message: String,
    /// The newly created user's unique identifier.
    pub user_id: Uuid,
}
```

## Rust Validator Documentation

All Rust validator modules in `/api/src/validators/` should follow the structure established in the `password_match` module. Validators provide cross-field validation that cannot be expressed with simple field-level attributes.

### Directory Structure

```
api/src/validators/
├── mod.rs              # Module-level docs and re-exports
└── password_match.rs   # Password confirmation validation
```

### Module Documentation (`validators/mod.rs`)

The top-level module should describe the purpose and list all submodules:

```rust
//! Custom validation functions for request payloads.
//!
//! This module provides validation helpers used with the `validator` crate
//! to perform cross-field validation that cannot be expressed with simple
//! field-level attributes.
//!
//! # Modules
//!
//! - [`password_match`] - Password confirmation validation for sign-up and password reset flows

pub mod password_match;
```

### Validator File Documentation

Each validator file should include:

1. Module-level docs (`//!`) describing the file's purpose
2. Private helper functions with `# Arguments` and `# Errors` sections
3. Public validation functions that reference the handlers/payloads that use them

```rust
//! Custom validation functions for request payloads.
//!
//! This module provides validation helpers used with the `validator` crate
//! to perform cross-field validation that cannot be expressed with simple
//! field-level attributes.

use crate::routes::auth::{SetPasswordRequest, SignUpRequest};

/// Validates that two password fields match.
///
/// This is a private helper function used by the public validation functions
/// for specific request types.
///
/// # Arguments
///
/// * `password` - The password field value
/// * `confirm` - The confirmation password field value
///
/// # Errors
///
/// Returns a `ValidationError` with code `password_mismatch` if the passwords
/// do not match.
fn validate_passwords_match(
    password: &str,
    confirm: &str,
) -> Result<(), validator::ValidationError> {
    if password != confirm {
        let mut error = validator::ValidationError::new("password_mismatch");
        error.message = Some("Passwords do not match".into());
        return Err(error);
    }
    Ok(())
}

/// Validates that the password and confirm fields match in a sign-up request.
///
/// Used with the `#[validate(custom(...))]` attribute on [`SignUpRequest`].
///
/// See [`sign_up`](crate::routes::auth::handlers::sign_up) for the handler
/// that uses this validation.
pub fn validate_signup_passwords_match(
    req: &SignUpRequest,
) -> Result<(), validator::ValidationError> {
    validate_passwords_match(&req.password, &req.confirm)
}
```

### Key Conventions

1. **Module-level docs** - Always include `//!` comments describing the file's purpose
2. **Private helpers** - Document with `# Arguments` and `# Errors` sections
3. **Public functions** - Include:
   - Description of what validation is performed
   - Which attribute/struct uses this validator (e.g., `#[validate(custom(...))]`)
   - Link to the handler that triggers this validation using rustdoc syntax
4. **Cross-references** - Link to related payload types and handlers for navigation

### Reference Example: `api/src/auth`

For Rust utility modules (outside models/routes/validators), use the documentation style now applied in `/api/src/auth/` as a reference baseline:

1. Add module-level `//!` docs explaining file purpose and scope
2. Document all public structs/functions with `///` comments
3. Add field-level docs for public struct fields
4. Include `# Arguments` and `# Errors` sections where behavior or failure modes are non-obvious
5. Prefer concise, implementation-aware wording (what the helper guarantees and how it is intended to be used)

## Storybook Documentation

When adding documentation to components and layouts, **always include Storybook stories** as part of the documentation. Stories serve as living documentation and visual testing.

### Configuration

Storybook is configured in `web/.storybook/` with:
- `@storybook/react-vite` framework
- Dark theme by default
- Addons: `@storybook/addon-docs`, `@storybook/addon-a11y`, `@storybook/addon-vitest`, `@chromatic-com/storybook`
- Path aliases: `@` → `src/`, `@sass` → `src/sass/`

### Story File Location

Stories are centralized in the `web/src/stories/` directory, mirroring the component structure:
- Components: `web/src/stories/components/core/ComponentName.stories.tsx`
- Layouts: `web/src/stories/layouts/LayoutName.stories.tsx`
- Icons: `web/src/stories/components/icons/Icons.stories.tsx` (single gallery file for all icons)
- Pages: `web/src/stories/pages/PageName.stories.tsx`

Stories should use absolute imports with the `@/` alias to reference components:
```tsx
import Button from "@/components/core/Button/Button";
```

### Story File Structure

All stories use CSF3 format with types from `@storybook/react-vite`:

```tsx
import type { Meta, StoryObj } from "@storybook/react-vite";
import Button, { ButtonVariant } from "@/components/core/Button/Button";

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
- `"Pages/PageName"` - Page components

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

## Web Testing Conventions

When implementing or updating tests in `/web`, use the following coverage standard.

### Testing Strategy

1. **Storybook interaction tests for user-facing behavior**
   - Prefer Storybook `play` tests for page flows and visible UI behavior.
   - Assert real behavior, not just rendering:
     - Form submission payloads
     - Route navigation/redirect outcomes
     - Success/error notification behavior
     - Validation error rendering
2. **Component behavior coverage**
   - Use Storybook interaction tests for interactive components (buttons, links, form controls, notifications, modals, auth wrappers).
   - Assert behavior, not only presence:
     - Event callbacks are fired (`onClick`, `onClose`, etc.)
     - Navigation actions route to expected targets
     - Variant/state changes affect output (classes, text, disabled/loading states, visibility)
     - Accessibility-relevant behavior is preserved (roles, labels, focusable controls)
   - For mostly presentational components, still test meaningful variants/states, not just a single default render.
3. **Route-wrapper behavior coverage**
   - Add stories for auth/protected route wrappers when logic exists in route files.
   - Cover loading, redirect, and authenticated rendering states.
4. **Targeted unit tests for internal side effects**
   - Use `*.test.tsx` for non-visual callback assertions that are awkward in Storybook:
     - auth context callbacks (e.g., `refreshUser`, `setUser`)
     - side-effect ordering and callback invocation guarantees
5. **Deterministic API mocking**
   - Mock API client calls in behavior tests and assert expected request payloads.
   - Include success and validation/error branches for form flows.

### Minimum Coverage Expectations for Component Tests

For component-level tests (especially in `web/src/stories/components`), include at least:

1. One behavior assertion beyond visibility (event callback, navigation, state change, or accessibility-relevant interaction)
2. Coverage for key variants/states exposed by props (e.g., variant enums, disabled/loading/error states)
3. Negative or alternate-path coverage where meaningful (e.g., no callback when disabled, hidden state when closed)

### Minimum Coverage Expectations for Auth/Page Flows

For page-level behavior tests, include at least:

1. One success-path test asserting payload + resulting navigation
2. One failure-path test asserting validation or error notification behavior
3. Route-wrapper state coverage (where applicable): loading + redirect + allowed render

### Test Documentation in Code

When adding or updating test files, add a concise top-of-file comment documenting:

1. What behavior the file covers
2. Which scenarios are tested
3. Which regressions the tests are meant to prevent

## API Testing Conventions

When implementing or updating tests in `/api`, maintain the same coverage bar now established in the auth testing suite.

### Required Coverage for API Changes

1. **Unit coverage for pure logic**
   - Add unit tests for helper logic in auth utilities, validators, extractors, cookie/JWT/code helpers, and similar pure modules.
   - Cover both success and failure/edge behavior.
2. **Handler-level route coverage**
   - Add tests in the route handler module (for example `api/src/routes/**/handlers.rs`) for request validation, auth-guard behavior, and non-DB early-return branches.
3. **Integration flow coverage**
   - Add integration tests in `api/tests/` for primary end-to-end flows with real database persistence checks.
   - Assert both API responses and key database side effects.
4. **Success + failure + security behavior**
   - Include at least one success path and one failure path for changed flows.
   - Include security-relevant behavior where applicable (for example: anti-enumeration responses, token type checks, cookie behavior, auth-code lifecycle).

### External Service Isolation

1. Do not call third-party services (email providers, external APIs) in tests.
2. Use trait-based dependency injection with mock implementations for external dependencies.

### Deterministic Test Setup

1. Use deterministic test environment configuration (dotenv/test env vars).
2. Use unique test data to avoid collisions.
3. Keep DB-backed integration tests reliable through proper isolation/serialization when needed.

## Code Review Process

When asked to perform a code review, follow this interactive process:

### What to Check

- **Spelling mistakes** - Check for typos in code, comments, and strings
- **Documentation compliance** - Ensure all files follow the documentation formats defined in this file (JSDoc comments, Storybook stories, MDX files for routes, etc.)
- **Web testing convention compliance** - Verify new/updated web component and page tests follow the `Web Testing Conventions` section (behavior-focused story tests, component variant/state coverage, route-wrapper coverage where needed, and targeted unit tests for internal side effects)
- **API testing convention compliance** - Verify new/updated API changes follow the `API Testing Conventions` section (unit + handler-level + integration flow coverage, with failure/security assertions and mocked external services)
- **Code quality issues** - Bugs, logic errors, and other problems

### Process

1. **Step through issues one at a time** - Do not provide all feedback in a single response
2. **For each issue found:**
   - Provide a clear description of the issue
   - Show a diff of the proposed fix
     - display this the same way you display changes to the code being made
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
     - The summary and title should reflect all the changes made on the current branch
     - If a PR for this branch already exists, update the summary to reflect
       any new changes that might be missing

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
            fill="currentColor"
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
- **fill**: Use `"currentColor"` to inherit from parent's CSS `color` property (enables theme support)
- **stroke-based icons**: If the SVG uses strokes instead of fills, set `stroke="currentColor"` on the path elements and keep `fill="none"`
- **JSDoc**: Include a brief description of the icon's appearance and typical usage
- **Naming**: Component name should be `{IconName}Icon` and match the filename

### Theme Support

Icons use `currentColor` to automatically adapt to light/dark themes. To ensure icons inherit the correct color, wrap them in an element with `color: var(--text-color)` or ensure a parent element sets this property.

## GitHub

### Project

All issues for this project should be included in the [GigLog Project](https://github.com/users/joegoggin/projects/1)
This project consists of three different fields:

#### Labels

- Feature
- Bug
- DevOps
- Documentation
- Refactor
- Testing
- Update

#### Status

- Todo
- In Progress
- Done

#### Priority

- Low 
- Medium
- High
- Urgent

### Issues

#### Creating Issues

When creating issues on GitHub for this project you should use the following
conventions:

- All new issues should be given a status of `Todo`
- If a priority isn't provided set the priority to `Medium` by default 
- Create a main issue with a summary of the full task that needs to be completed
- Break up the full task into small tasks and add those as sub-issues
- Each sub task should have a summary of the small task and match the priority
  of the main issue
- The order of the sub-task should be in the order the tasks should be
  implemented

#### Implementing Issues

When asked to implement an issue you should do the following:

- Read the main issue for context and update its status to `In Progress`
- Walk through each sub-issue and do the following
    - Read the sub-issue for context
    - Update the status of the sub-issue to `In Progress`
    - Implement the task    
    - Describe what you did and provide steps to test  
    - Wait for me to tell you to move to the next task
    - Once you move to the next task set the status of the sub-issue to `Done`
- Once you are done with all the sub-tasks prompt me for a code review
