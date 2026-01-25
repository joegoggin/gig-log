import { createContext, useContext, useState, useCallback, type ReactNode } from "react";
import type { NotificationProps } from "@/components/core/Notification/Notification";

type Notification = NotificationProps & {
    id: string;
};

type NotificationContextType = {
    notifications: Notification[];
    addNotification: (notification: Omit<Notification, "id">) => void;
    removeNotification: (id: string) => void;
};

const NotificationContext = createContext<NotificationContextType | null>(null);

type NotificationProviderProps = {
    children: ReactNode;
};

export function NotificationProvider({ children }: NotificationProviderProps) {
    const [notifications, setNotifications] = useState<Notification[]>([]);

    const addNotification = useCallback((notification: Omit<Notification, "id">) => {
        const id = crypto.randomUUID();
        setNotifications((prev) => [...prev, { ...notification, id }]);
    }, []);

    const removeNotification = useCallback((id: string) => {
        setNotifications((prev) => prev.filter((n) => n.id !== id));
    }, []);

    return (
        <NotificationContext.Provider
            value={{ notifications, addNotification, removeNotification }}
        >
            {children}
        </NotificationContext.Provider>
    );
}

export function useNotification() {
    const context = useContext(NotificationContext);

    if (!context) {
        throw new Error("useNotification must be used within a NotificationProvider");
    }

    return context;
}
