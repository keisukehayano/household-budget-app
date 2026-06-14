import type {
    AuthResponse,
    AuthUser,
    ChangePasswordInput,
    ForgotPasswordInput,
    ForgotPasswordResponse,
    LoginInput,
    RegisterInput,
    ResetPasswordInput,
} from '../types';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? 'http://127.0.0.1:8080';

type ApiErrorResponse = {
    message: string;
    details?: string[];
};

export class AuthApiError extends Error {
    public readonly status: number;

    constructor(
        status: number,
        message: string,
    ) {
        super(message);
        this.name = 'AuthApiError';
        this.status = status;
    }
}

const createApiErrorMessage = (errorResponse: ApiErrorResponse): string => {
    if (errorResponse.details && errorResponse.details.length > 0) {
        return `${errorResponse.message}\n${errorResponse.details.join('\n')}`;
    }

    return errorResponse.message;
};

const handleResponse = async <T>(response: Response): Promise<T> => {
    if (!response.ok) {
        let errorMessage = `API request failed: ${response.status}`;

        try {
            const errorResponse = (await response.json()) as ApiErrorResponse;
            errorMessage = createApiErrorMessage(errorResponse);
        } catch {
            // JSON形式ではないエラーの場合は、デフォルトメッセージを使う。
        }

        throw new AuthApiError(response.status, errorMessage);
    }

    return response.json() as Promise<T>;
};

export const register = async (input: RegisterInput): Promise<AuthResponse> => {
    const response = await fetch(`${API_BASE_URL}/api/auth/register`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(input),
    });

    return handleResponse<AuthResponse>(response);
};

export const login = async (input: LoginInput): Promise<AuthResponse> => {
    const response = await fetch(`${API_BASE_URL}/api/auth/login`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(input),
    });

    return handleResponse<AuthResponse>(response);
};

export const fetchMe = async (token: string): Promise<AuthUser> => {
    const response = await fetch(`${API_BASE_URL}/api/auth/me`, {
        headers: {
            Authorization: `Bearer ${token}`,
        },
    });

    return handleResponse<AuthUser>(response);
};

export const changePassword = async (
    token: string,
    input: ChangePasswordInput,
): Promise<AuthResponse> => {
    const response = await fetch(`${API_BASE_URL}/api/auth/change-password`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify(input),
    });

    return handleResponse<AuthResponse>(response);
};

export const forgotPassword = async (
    input: ForgotPasswordInput,
): Promise<ForgotPasswordResponse> => {
    const response = await fetch(`${API_BASE_URL}/api/auth/forgot-password`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(input),
    });

    return handleResponse<ForgotPasswordResponse>(response);
};

export const resetPassword = async (input: ResetPasswordInput): Promise<void> => {
    const response = await fetch(`${API_BASE_URL}/api/auth/reset-password`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(input),
    });

    if (!response.ok) {
        let errorMessage = `API request failed: ${response.status}`;

        try {
            const errorResponse = (await response.json()) as ApiErrorResponse;
            errorMessage = createApiErrorMessage(errorResponse);
        } catch {
            // JSON形式ではないエラーの場合は、デフォルトメッセージを使う。
        }

        throw new AuthApiError(response.status, errorMessage);
    }
};
