(function () {
    interface SessionUser {
        id: number;
        username: string;
        email: string;
    }

    interface ApiError {
        error: string;
    }

    let currentUser: SessionUser | null = null;

    function getSessionId(): string | null {
        const cookies = document.cookie.split(";");
        for (const cookie of cookies) {
            const [name, value] = cookie.trim().split("=");
            if (name === "session_id") {
                return value;
            }
        }
        return null;
    }

    async function checkSession(): Promise<SessionUser | null> {
        const sessionId = getSessionId();
        if (!sessionId) {
            return null;
        }

        try {
            const response = await fetch(`/api/sessions/${sessionId}`);
            if (!response.ok) {
                return null;
            }
            return await response.json();
        } catch {
            return null;
        }
    }

    async function logout(): Promise<void> {
        const sessionId = getSessionId();
        if (sessionId) {
            try {
                await fetch(`/api/sessions/${sessionId}`, { method: "DELETE" });
            } catch {
                // Ignore errors
            }
        }
        window.location.href = "index.html";
    }

    function setupNav(): void {
        const showLinksDiv = document.getElementById("show-links");
        const navLinksUL = document.getElementById("nav-links");

        if (showLinksDiv && navLinksUL) {
            showLinksDiv.addEventListener("click", () => {
                navLinksUL.hidden = !navLinksUL.hidden;
                showLinksDiv.textContent = navLinksUL.hidden ? "+" : "-";
            });
        }

        const logoutBtn = document.getElementById("logout-btn");
        if (logoutBtn) {
            logoutBtn.addEventListener("click", logout);
        }
    }

    function displayUserInfo(user: SessionUser): void {
        const usernameInput = document.getElementById("current-username") as HTMLInputElement;
        const emailInput = document.getElementById("current-email") as HTMLInputElement;

        if (usernameInput) usernameInput.value = user.username;
        if (emailInput) emailInput.value = user.email;
    }

    async function updateAccount(
        userId: number,
        currentPassword: string,
        username?: string,
        email?: string,
        password?: string
    ): Promise<void> {
        const body: Record<string, string> = { current_password: currentPassword };
        if (username) body.username = username;
        if (email) body.email = email;
        if (password) body.password = password;

        const response = await fetch(`/api/users/${userId}`, {
            method: "PATCH",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(body),
        });

        if (!response.ok) {
            const data = (await response.json()) as ApiError;
            throw new Error(data.error || "Failed to update account");
        }
    }

    async function deleteAccount(userId: number, password: string): Promise<void> {
        const response = await fetch(`/api/users/${userId}`, {
            method: "DELETE",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ password }),
        });

        if (!response.ok) {
            const data = (await response.json()) as ApiError;
            throw new Error(data.error || "Failed to delete account");
        }
    }

    function showUpdateError(message: string): void {
        const errorEl = document.getElementById("update-error");
        const successEl = document.getElementById("update-success");

        if (errorEl) {
            errorEl.textContent = message;
            errorEl.hidden = false;
        }
        if (successEl) {
            successEl.hidden = true;
        }
    }

    function showUpdateSuccess(): void {
        const errorEl = document.getElementById("update-error");
        const successEl = document.getElementById("update-success");

        if (errorEl) {
            errorEl.hidden = true;
        }
        if (successEl) {
            successEl.hidden = false;
        }
    }

    function showDeleteError(message: string): void {
        const errorEl = document.getElementById("delete-error");
        if (errorEl) {
            errorEl.textContent = message;
            errorEl.hidden = false;
        }
    }

    function hideDeleteError(): void {
        const errorEl = document.getElementById("delete-error");
        if (errorEl) {
            errorEl.hidden = true;
        }
    }

    function setupUpdateForm(): void {
        const form = document.getElementById("update-form") as HTMLFormElement;
        if (!form) return;

        form.addEventListener("submit", async (e) => {
            e.preventDefault();

            if (!currentUser) return;

            const newUsernameInput = document.getElementById("new-username") as HTMLInputElement;
            const newEmailInput = document.getElementById("new-email") as HTMLInputElement;
            const newPasswordInput = document.getElementById("new-password") as HTMLInputElement;
            const confirmPasswordInput = document.getElementById("confirm-password") as HTMLInputElement;
            const currentPasswordInput = document.getElementById("current-password") as HTMLInputElement;

            const newUsername = newUsernameInput.value.trim();
            const newEmail = newEmailInput.value.trim();
            const newPassword = newPasswordInput.value;
            const confirmPassword = confirmPasswordInput.value;
            const currentPassword = currentPasswordInput.value;

            // Validate at least one field is being changed
            if (!newUsername && !newEmail && !newPassword) {
                showUpdateError("Please provide at least one field to update");
                return;
            }

            // Validate password confirmation
            if (newPassword && newPassword !== confirmPassword) {
                showUpdateError("New password and confirmation do not match");
                return;
            }

            // Validate current password is provided
            if (!currentPassword) {
                showUpdateError("Current password is required to make changes");
                return;
            }

            try {
                await updateAccount(
                    currentUser.id,
                    currentPassword,
                    newUsername || undefined,
                    newEmail || undefined,
                    newPassword || undefined
                );

                // Update current user info
                if (newUsername) currentUser.username = newUsername;
                if (newEmail) currentUser.email = newEmail;
                displayUserInfo(currentUser);

                // Clear form
                newUsernameInput.value = "";
                newEmailInput.value = "";
                newPasswordInput.value = "";
                confirmPasswordInput.value = "";
                currentPasswordInput.value = "";

                showUpdateSuccess();

                // Hide success message after 5 seconds
                setTimeout(() => {
                    const successEl = document.getElementById("update-success");
                    if (successEl) successEl.hidden = true;
                }, 5000);
            } catch (error) {
                console.error("Error updating account:", error);
                showUpdateError(error instanceof Error ? error.message : "Failed to update account");
            }
        });
    }

    function showDeleteModal(): void {
        const modal = document.getElementById("delete-modal");
        if (modal) {
            modal.hidden = false;
            hideDeleteError();
            // Clear password field
            const passwordInput = document.getElementById("delete-password") as HTMLInputElement;
            if (passwordInput) passwordInput.value = "";
        }
    }

    function hideDeleteModal(): void {
        const modal = document.getElementById("delete-modal");
        if (modal) modal.hidden = true;
    }

    function setupDeleteFlow(): void {
        const deleteBtn = document.getElementById("delete-account-btn");
        const modal = document.getElementById("delete-modal");
        const closeBtn = document.getElementById("delete-modal-close");
        const cancelBtn = document.getElementById("delete-cancel");
        const deleteForm = document.getElementById("delete-form") as HTMLFormElement;

        if (deleteBtn) {
            deleteBtn.addEventListener("click", showDeleteModal);
        }

        if (closeBtn) {
            closeBtn.addEventListener("click", hideDeleteModal);
        }

        if (cancelBtn) {
            cancelBtn.addEventListener("click", hideDeleteModal);
        }

        if (deleteForm) {
            deleteForm.addEventListener("submit", async (e) => {
                e.preventDefault();

                if (!currentUser) return;

                const passwordInput = document.getElementById("delete-password") as HTMLInputElement;
                const password = passwordInput.value;

                if (!password) {
                    showDeleteError("Password is required");
                    return;
                }

                try {
                    await deleteAccount(currentUser.id, password);
                    // Redirect to index on success
                    window.location.href = "index.html";
                } catch (error) {
                    console.error("Error deleting account:", error);
                    showDeleteError(error instanceof Error ? error.message : "Failed to delete account");
                }
            });
        }
    }

    function showContent(): void {
        const loadingEl = document.getElementById("account-loading");
        const contentEl = document.getElementById("account-content");

        if (loadingEl) loadingEl.hidden = true;
        if (contentEl) contentEl.hidden = false;
    }

    function showError(): void {
        const loadingEl = document.getElementById("account-loading");
        const errorEl = document.getElementById("account-error");

        if (loadingEl) loadingEl.hidden = true;
        if (errorEl) errorEl.hidden = false;
    }

    async function init(): Promise<void> {
        setupNav();

        const user = await checkSession();
        if (!user) {
            showError();
            return;
        }

        currentUser = user;
        displayUserInfo(user);
        setupUpdateForm();
        setupDeleteFlow();
        showContent();
    }

    document.addEventListener("DOMContentLoaded", init);
})();
