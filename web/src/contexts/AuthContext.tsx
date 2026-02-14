import {
    createContext,
    useCallback,
    useContext,
    useEffect,
    useRef,
    useState,
} from "react";
import type {ReactNode} from "react";
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

type RefreshUserOptions = {
    throwOnError?: boolean;
};

type AuthContextValue = {
    user: AuthUser | null;
    isLoggedIn: boolean;
    isLoading: boolean;
    refreshUser: (options?: RefreshUserOptions) => Promise<void>;
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
    const inFlightRefreshRef = useRef<Promise<void> | null>(null);

    const refreshUser = useCallback(async (options?: RefreshUserOptions) => {
        if (inFlightRefreshRef.current) {
            return inFlightRefreshRef.current;
        }

        const refreshPromise = (async () => {
            try {
                const response = await api.get<CurrentUserResponse>("/auth/me");
                setUser(response.data.user);
            } catch (error) {
                setUser(null);

                if (options?.throwOnError) {
                    throw error;
                }
            } finally {
                inFlightRefreshRef.current = null;
            }
        })();

        inFlightRefreshRef.current = refreshPromise;

        return refreshPromise;
    }, []);

    useEffect(() => {
        const checkAuthOnAppStart = async () => {
            setIsLoading(true);

            try {
                await refreshUser();
            } finally {
                setIsLoading(false);
            }
        };

        void checkAuthOnAppStart();
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
