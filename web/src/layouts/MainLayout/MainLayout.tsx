import { useMutation } from "@tanstack/react-query";
import { useLocation, useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import styles from "./MainLayout.module.scss";
import type { AxiosError } from "axios";
import type { ReactNode } from "react";
import { NotificationType } from "@/components/core/Notification/Notification";
import CompanyIcon from "@/components/icons/CompanyIcon";
import CloseIcon from "@/components/icons/CloseIcon";
import GigLogLogoIcon from "@/components/icons/GigLogLogoIcon";
import HamburgerIcon from "@/components/icons/HamburgerIcon";
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
    const [isMobileMenuOpen, setIsMobileMenuOpen] = useState<boolean>(false);

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
        setIsMobileMenuOpen(false);
        navigate({ to: path });
    };

    const closeMobileMenu = () => {
        setIsMobileMenuOpen(false);
    };

    useEffect(() => {
        setIsMobileMenuOpen(false);
    }, [pathname]);

    useEffect(() => {
        if (!isMobileMenuOpen) {
            return;
        }

        const handleEscape = (event: KeyboardEvent) => {
            if (event.key === "Escape") {
                setIsMobileMenuOpen(false);
            }
        };

        window.addEventListener("keydown", handleEscape);

        return () => {
            window.removeEventListener("keydown", handleEscape);
        };
    }, [isMobileMenuOpen]);

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
                <aside
                    className={`${styles["main-layout__sidebar"]} ${
                        isMobileMenuOpen ? styles["main-layout__sidebar--mobile-open"] : ""
                    }`}
                >
                    <div className={styles["main-layout__top-bar"]}>
                        <div className={styles["main-layout__brand"]}>
                            <button
                                type="button"
                                aria-label="Go to dashboard"
                                className={styles["main-layout__brand-button"]}
                                onClick={() => navigateTo("/dashboard")}
                            >
                                <span className={styles["main-layout__brand-logo-mark"]}>
                                    <GigLogLogoIcon showWordmark={false} />
                                </span>
                                <span className={styles["main-layout__brand-logo-full"]}>
                                    <GigLogLogoIcon showSubtitle={false} />
                                </span>
                            </button>
                        </div>
                        <button
                            type="button"
                            aria-label={isMobileMenuOpen ? "Close navigation menu" : "Open navigation menu"}
                            aria-controls="main-layout-mobile-drawer"
                            aria-expanded={isMobileMenuOpen}
                            className={styles["main-layout__menu-toggle"]}
                            onClick={() => setIsMobileMenuOpen((prev) => !prev)}
                        >
                            {isMobileMenuOpen ? <CloseIcon /> : <HamburgerIcon />}
                        </button>
                    </div>
                    <div
                        id="main-layout-mobile-drawer"
                        className={`${styles["main-layout__drawer"]} ${
                            isMobileMenuOpen ? styles["main-layout__drawer--open"] : ""
                        }`}
                    >
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
                            onClick={() => {
                                closeMobileMenu();
                                logoutMutation.mutate();
                            }}
                        >
                            <LogOutIcon />
                            <p>Log Out</p>
                        </button>
                    </div>
                </aside>
                <button
                    type="button"
                    aria-label="Dismiss navigation overlay"
                    aria-hidden={!isMobileMenuOpen}
                    tabIndex={isMobileMenuOpen ? 0 : -1}
                    disabled={!isMobileMenuOpen}
                    className={`${styles["main-layout__backdrop"]} ${
                        isMobileMenuOpen ? styles["main-layout__backdrop--visible"] : ""
                    }`}
                    onClick={closeMobileMenu}
                />
                <main className={contentClassName}>{children}</main>
            </div>
        </RootLayout>
    );
}

export default MainLayout;
