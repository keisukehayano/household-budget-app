import type { AuthSession } from '../types';

const AUTH_STORAGE_KEY = 'household-budget-auth-session';

export const loadAuthSession = (): AuthSession | null => {
    try {
        const rawValue = window.localStorage.getItem(AUTH_STORAGE_KEY);

        if (rawValue === null) {
            return null;
        }

        return JSON.parse(rawValue) as AuthSession;
    } catch {
        return null;
    }
};

export const saveAuthSession = (session: AuthSession): void => {
    window.localStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(session));
};

export const clearAuthSession = (): void => {
    window.localStorage.removeItem(AUTH_STORAGE_KEY);
};
