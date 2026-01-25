import { createContext, useCallback, useContext, useEffect, useState, type ReactNode } from "react";
import api from "@/lib/axios";

type AuthUser = {
    id: string;
    first_name: string;
    last_name: string;
    email: string;
    email_confirmed: boolean;
    created_at: string;
    updated_at: string;
};

type AuthContextValue = {
    user: AuthUser | null;
    isLoggedIn: boolean;
    isLoading: boolean;
    refreshUser: () => Promise<void>;
    setUser: (user: AuthUser | null) => void;
};

type CurrentUserResponse = {
    user: AuthUser;
};

export const AuthContext = createContext<AuthContextValue | null>(null);

type AuthProviderProps = {
    children: ReactNode;
};

export function AuthProvider({ children }: AuthProviderProps) {
    const [user, setUser] = useState<AuthUser | null>(null);
    const [isLoading, setIsLoading] = useState(true);

    const refreshUser = useCallback(async () => {
        setIsLoading(true);

        try {
            const response = await api.get<CurrentUserResponse>("/auth/me");
            setUser(response.data.user);
        } catch {
            setUser(null);
        } finally {
            setIsLoading(false);
        }
    }, []);

    useEffect(() => {
        void refreshUser();
    }, [refreshUser]);

    return (
        <AuthContext.Provider
            value={{
                user,
                isLoggedIn: Boolean(user),
                isLoading,
                refreshUser,
                setUser,
            }}
        >
            {children}
        </AuthContext.Provider>
    );
}

export function useAuth() {
    const context = useContext(AuthContext);

    if (!context) {
        throw new Error("useAuth must be used within an AuthProvider");
    }

    return context;
}
