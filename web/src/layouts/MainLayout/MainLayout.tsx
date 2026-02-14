import { useMutation } from "@tanstack/react-query";
import { useLocation, useNavigate } from "@tanstack/react-router";
import styles from "./MainLayout.module.scss";
import type { AxiosError } from "axios";
import type { ReactNode } from "react";
import { NotificationType } from "@/components/core/Notification/Notification";
import CompanyIcon from "@/components/icons/CompanyIcon";
import HomeIcon from "@/components/icons/HomeIcon";
import JobsIcon from "@/components/icons/JobsIcon";
import LogOutIcon from "@/components/icons/LogOutIcon";
import PaymentIcon from "@/components/icons/PaymentIcon";
import SettingsIcon from "@/components/icons/SettingsIcon";
import { useAuth } from "@/contexts/AuthContext";
import { useNotification } from "@/contexts/NotificationContext";
import api from "@/lib/axios";
import RootLayout from "@/layouts/RootLayout/RootLayout";

type LogOutResponse = {
    message: string;
};

type ApiErrorResponse = {
    error: string;
};

type NavItem = {
    label: string;
    path: string;
    icon: ReactNode;
};

/**
 * Props for the MainLayout component.
 */
type MainLayoutProps = {
    /** Additional CSS class names to apply to the content area */
    className?: string;
    /** Content to render inside the layout */
    children: ReactNode;
};

const navItems: Array<NavItem> = [
    {
        label: "Dashboard",
        path: "/dashboard",
        icon: <HomeIcon />,
    },
    {
        label: "Companies",
        path: "/companies",
        icon: <CompanyIcon />,
    },
    {
        label: "Jobs",
        path: "/jobs",
        icon: <JobsIcon />,
    },
    {
        label: "Payments",
        path: "/payments",
        icon: <PaymentIcon />,
    },
    {
        label: "Settings",
        path: "/settings",
        icon: <SettingsIcon />,
    },
];

/**
 *
 * Primary authenticated layout with persistent app navigation.
 * Renders sidebar navigation links, highlights the active route,
 * and provides a log-out action.
 *
 * ## Props
 *
 * - `className` - Additional CSS class names to apply to the content area
 * - `children` - Content to render inside the layout
 *
 * ## Example
 *
 * ```tsx
 * <MainLayout>
 *   <h1>Dashboard</h1>
 * </MainLayout>
 * ```
 */
function MainLayout({ className = "", children }: MainLayoutProps) {
    const navigate = useNavigate();
    const { pathname } = useLocation();
    const { setUser } = useAuth();
    const { addNotification } = useNotification();

    const logoutMutation = useMutation({
        mutationFn: async () => {
            const response = await api.post<LogOutResponse>("/auth/log-out");
            return response.data;
        },
        onSuccess: () => {
            setUser(null);
            navigate({ to: "/auth/log-in" });
        },
        onError: (error: AxiosError<ApiErrorResponse>) => {
            const message = error.response?.data.error || "Failed to log out";
            addNotification({
                type: NotificationType.ERROR,
                title: "Log Out Failed",
                message,
            });
        },
    });

    const navigateTo = (path: string) => {
        navigate({ to: path });
    };

    const isPathActive = (path: string) => {
        return pathname === path || pathname.startsWith(`${path}/`);
    };

    const getNavItemClassName = (path: string) => {
        let classNameValue = styles["main-layout__menu-item"];

        if (isPathActive(path)) {
            classNameValue += ` ${styles["main-layout__menu-item--active"]}`;
        }

        return classNameValue;
    };

    const contentClassName = className
        ? `${styles["main-layout__content"]} ${className}`
        : styles["main-layout__content"];

    return (
        <RootLayout>
            <div className={styles["main-layout"]}>
                <aside className={styles["main-layout__sidebar"]}>
                    <div className={styles["main-layout__brand"]}>
                        <h5>GigLog</h5>
                    </div>
                    <nav className={styles["main-layout__menu"]}>
                        {navItems.map((item) => {
                            const isActive = isPathActive(item.path);

                            return (
                                <button
                                    key={item.path}
                                    type="button"
                                    aria-label={item.label}
                                    aria-current={isActive ? "page" : undefined}
                                    className={getNavItemClassName(item.path)}
                                    onClick={() => navigateTo(item.path)}
                                >
                                    {item.icon}
                                    <p>{item.label}</p>
                                </button>
                            );
                        })}
                    </nav>
                    <button
                        type="button"
                        aria-label="Log Out"
                        className={`${styles["main-layout__menu-item"]} ${styles["main-layout__log-out"]}`}
                        onClick={() => logoutMutation.mutate()}
                    >
                        <LogOutIcon />
                        <p>Log Out</p>
                    </button>
                </aside>
                <main className={contentClassName}>{children}</main>
            </div>
        </RootLayout>
    );
}

export default MainLayout;
