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
