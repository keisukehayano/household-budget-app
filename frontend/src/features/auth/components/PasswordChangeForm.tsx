import { useState } from 'react';
import type { FormEvent } from 'react';
import { AuthApiError, changePassword } from '../api/authApi';
import type { AuthSession } from '../types';

type PasswordChangeFormProps = {
    token: string;
    onChanged: (session: AuthSession) => void;
    onUnauthorized: () => void;
    onCancel: () => void;
};

export const PasswordChangeForm = ({
    token,
    onChanged,
    onUnauthorized,
    onCancel,
}: PasswordChangeFormProps) => {
    const [currentPassword, setCurrentPassword] = useState('');
    const [newPassword, setNewPassword] = useState('');
    const [newPasswordConfirmation, setNewPasswordConfirmation] = useState('');
    const [errorMessage, setErrorMessage] = useState('');
    const [isSubmitting, setIsSubmitting] = useState(false);

    const validateForm = (): string | null => {
        if (!currentPassword) {
            return '現在のパスワードを入力してください。';
        }

        if (!newPassword) {
            return '新しいパスワードを入力してください。';
        }

        if (newPassword.length < 8) {
            return '新しいパスワードは8文字以上で入力してください。';
        }

        if (newPassword.length > 128) {
            return '新しいパスワードは128文字以内で入力してください。';
        }

        if (!newPasswordConfirmation) {
            return '新しいパスワード確認を入力してください。';
        }

        if (newPassword !== newPasswordConfirmation) {
            return '新しいパスワードと確認用パスワードが一致しません。';
        }

        if (currentPassword === newPassword) {
            return '新しいパスワードは現在のパスワードと異なるものを入力してください。';
        }

        return null;
    };

    const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();

        const validationError = validateForm();

        if (validationError !== null) {
            setErrorMessage(validationError);
            return;
        }

        try {
            setIsSubmitting(true);
            setErrorMessage('');

            const session = await changePassword(token, {
                currentPassword,
                newPassword,
            });

            onChanged(session);
        } catch (error) {
            console.error(error);

            if (error instanceof AuthApiError && error.status === 401) {
                onUnauthorized();
                return;
            }

            setErrorMessage(
                error instanceof Error ? error.message : 'パスワード変更に失敗しました。',
            );
        } finally {
            setIsSubmitting(false);
        }
    };

    return (
        <section className="password-change-panel">
            <div className="password-change-card">
                <div className="password-change-heading">
                    <h2>パスワード変更</h2>

                    <button type="button" onClick={onCancel} disabled={isSubmitting}>
                        閉じる
                    </button>
                </div>

                {errorMessage && <p className="page-error">{errorMessage}</p>}

                <form className="password-change-form" onSubmit={handleSubmit}>
                    <div className="form-row">
                        <label htmlFor="current-password">現在のパスワード</label>
                        <input
                            id="current-password"
                            type="password"
                            value={currentPassword}
                            autoComplete="current-password"
                            disabled={isSubmitting}
                            onChange={(event) => setCurrentPassword(event.target.value)}
                        />
                    </div>

                    <div className="form-row">
                        <label htmlFor="new-password">新しいパスワード</label>
                        <input
                            id="new-password"
                            type="password"
                            value={newPassword}
                            autoComplete="new-password"
                            disabled={isSubmitting}
                            onChange={(event) => setNewPassword(event.target.value)}
                        />
                    </div>

                    <div className="form-row">
                        <label htmlFor="new-password-confirmation">新しいパスワード確認</label>
                        <input
                            id="new-password-confirmation"
                            type="password"
                            value={newPasswordConfirmation}
                            autoComplete="new-password"
                            disabled={isSubmitting}
                            onChange={(event) =>
                                setNewPasswordConfirmation(event.target.value)
                            }
                        />
                    </div>

                    <div className="password-change-actions">
                        <button type="submit" disabled={isSubmitting}>
                            {isSubmitting ? '変更中...' : '変更する'}
                        </button>

                        <button type="button" onClick={onCancel} disabled={isSubmitting}>
                            キャンセル
                        </button>
                    </div>
                </form>
            </div>
        </section>
    );
};
