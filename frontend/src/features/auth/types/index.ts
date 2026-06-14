export type AuthUser = {
    id: string;
    email: string;
    createdAt: string;
    updatedAt: string;
};

export type AuthResponse = {
    token: string;
    user: AuthUser;
};

export type AuthSession = AuthResponse;

export type LoginInput = {
    email: string;
    password: string;
};

export type RegisterInput = {
    email: string;
    password: string;
};

export type ChangePasswordInput = {
    currentPassword: string;
    newPassword: string;
};

export type ForgotPasswordInput = {
    email: string;
};

export type ForgotPasswordResponse = {
    message: string;
};

export type ResetPasswordInput = {
    token: string;
    newPassword: string;
};
