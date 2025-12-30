(function () {
    interface ApiError {
        error: string;
    }

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

    async function checkExistingSession(): Promise<boolean> {
        const sessionId = getSessionId();
        if (!sessionId) {
            return false;
        }

        try {
            const response = await fetch(`/api/sessions/${sessionId}`);
            return response.ok;
        } catch {
            return false;
        }
    }

    function showError(message: string): void {
        const errorEl = document.getElementById("error-message");
        if (errorEl) {
            errorEl.textContent = message;
            errorEl.classList.add("visible");
        }
    }

    function hideError(): void {
        const errorEl = document.getElementById("error-message");
        if (errorEl) {
            errorEl.classList.remove("visible");
        }
    }

    function setLoading(loading: boolean): void {
        const submitBtn = document.getElementById("submit-btn") as HTMLButtonElement;
        if (submitBtn) {
            submitBtn.disabled = loading;
            submitBtn.textContent = loading ? "Creating Account..." : "Create Account";
        }
    }

    async function createUser(
        username: string,
        email: string,
        password: string
    ): Promise<{ id: number } | null> {
        const response = await fetch("/api/users", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ username, email, password }),
        });

        if (!response.ok) {
            const data = (await response.json()) as ApiError;
            throw new Error(data.error || "Failed to create account");
        }

        return response.json();
    }

    async function createSession(emailOrUsername: string, password: string): Promise<void> {
        const response = await fetch("/api/sessions", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                email_or_username: emailOrUsername,
                password,
            }),
        });

        if (!response.ok) {
            const data = (await response.json()) as ApiError;
            throw new Error(data.error || "Failed to log in");
        }
    }

    async function handleSubmit(event: Event): Promise<void> {
        event.preventDefault();
        hideError();

        const form = event.target as HTMLFormElement;
        const username = (form.elements.namedItem("username") as HTMLInputElement).value.trim();
        const email = (form.elements.namedItem("email") as HTMLInputElement).value.trim();
        const password = (form.elements.namedItem("password") as HTMLInputElement).value;
        const confirmPassword = (form.elements.namedItem("confirm-password") as HTMLInputElement).value;

        // Validate passwords match
        if (password !== confirmPassword) {
            showError("Passwords do not match.");
            return;
        }

        setLoading(true);

        try {
            // Step 1: Create the user
            await createUser(username, email, password);

            // Step 2: Create a session (log them in)
            await createSession(username, password);

            // Step 3: Redirect to the home page
            window.location.href = "index.html";
        } catch (error) {
            const message = error instanceof Error ? error.message : "An unexpected error occurred";
            showError(message);
            setLoading(false);
        }
    }

    async function init(): Promise<void> {
        // Redirect to index if already logged in
        const hasValidSession = await checkExistingSession();
        if (hasValidSession) {
            window.location.href = "index.html";
            return;
        }

        const form = document.getElementById("signup-form");
        if (form) {
            form.addEventListener("submit", handleSubmit);
        }
    }

    document.addEventListener("DOMContentLoaded", init);
})();
