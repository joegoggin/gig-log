
/**
 * The authenticated dashboard page for signed-in users.
 * Displays dashboard content and provides a log-out action.
 *
 * Route: `/dashboard`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `MainLayout` - Wraps the page with primary app navigation.
 */
function DashboardPage() {
    return (
        <>
            <h1>Dashboard</h1>
            <p>Welcome back. Use the sidebar to navigate across the app.</p>
        </>
    );
}

export default DashboardPage;
