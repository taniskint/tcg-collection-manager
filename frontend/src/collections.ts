(function () {
    interface Collection {
        id: number;
        game_id: number;
        name: string;
        created_at: string;
        game_name: string;
        game_image_url: string | null;
    }

    interface Game {
        id: number;
        name: string;
        image_url: string | null;
        set_count: number;
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

    async function checkSession(): Promise<boolean> {
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

    function setupNav(isLoggedIn: boolean): void {
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
            if (isLoggedIn) {
                logoutBtn.addEventListener("click", logout);
            } else {
                logoutBtn.parentElement?.remove();
            }
        }
    }

    async function loadCollections(): Promise<Collection[]> {
        const response = await fetch("/api/collections");
        if (!response.ok) {
            throw new Error("Failed to load collections");
        }
        return response.json();
    }

    async function loadGames(): Promise<Game[]> {
        const response = await fetch("/api/games");
        if (!response.ok) {
            throw new Error("Failed to load games");
        }
        return response.json();
    }

    function formatDate(isoDate: string): string {
        const date = new Date(isoDate);
        return date.toLocaleDateString();
    }

    function renderCollections(collections: Collection[]): void {
        const loadingEl = document.getElementById("collections-loading");
        const emptyEl = document.getElementById("collections-empty");
        const gridEl = document.getElementById("collections-grid");

        if (!loadingEl || !emptyEl || !gridEl) return;

        loadingEl.hidden = true;

        if (collections.length === 0) {
            emptyEl.hidden = false;
            gridEl.hidden = true;
            return;
        }

        emptyEl.hidden = true;
        gridEl.innerHTML = collections
            .map(
                (collection) => `
                <a href="collection.html?id=${collection.id}" class="collection-card">
                    <div class="collection-game">
                        ${collection.game_image_url ? `<img src="${collection.game_image_url}" alt="${collection.game_name}">` : ""}
                        <span class="collection-game-name">${collection.game_name}</span>
                    </div>
                    <div class="collection-info">
                        <h3>${collection.name}</h3>
                        <p class="collection-meta">Created ${formatDate(collection.created_at)}</p>
                    </div>
                </a>
            `
            )
            .join("");

        gridEl.hidden = false;
    }

    function populateGameSelect(games: Game[]): void {
        const select = document.getElementById("game-select") as HTMLSelectElement;
        if (!select) return;

        // Keep the placeholder option
        select.innerHTML = '<option value="">Select a game...</option>';

        games.forEach((game) => {
            const option = document.createElement("option");
            option.value = game.id.toString();
            option.textContent = game.name;
            select.appendChild(option);
        });
    }

    function showModal(): void {
        const modal = document.getElementById("create-modal");
        if (modal) modal.hidden = false;
    }

    function hideModal(): void {
        const modal = document.getElementById("create-modal");
        const form = document.getElementById("create-collection-form") as HTMLFormElement;
        const errorEl = document.getElementById("form-error");

        if (modal) modal.hidden = true;
        if (form) form.reset();
        if (errorEl) errorEl.hidden = true;
    }

    function showFormError(message: string): void {
        const errorEl = document.getElementById("form-error");
        if (errorEl) {
            errorEl.textContent = message;
            errorEl.hidden = false;
        }
    }

    async function createCollection(gameId: number, name: string): Promise<void> {
        const response = await fetch("/api/collections", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ game_id: gameId, name }),
        });

        if (!response.ok) {
            const data = await response.json();
            throw new Error(data.error || "Failed to create collection");
        }
    }

    function setupModal(): void {
        const newBtn = document.getElementById("new-collection-btn");
        const closeBtn = document.getElementById("close-modal-btn");
        const cancelBtn = document.getElementById("cancel-btn");
        const modal = document.getElementById("create-modal");
        const form = document.getElementById("create-collection-form") as HTMLFormElement;

        if (newBtn) {
            newBtn.hidden = false;
            newBtn.addEventListener("click", showModal);
        }

        if (closeBtn) {
            closeBtn.addEventListener("click", hideModal);
        }

        if (cancelBtn) {
            cancelBtn.addEventListener("click", hideModal);
        }

        // Close modal when clicking outside
        if (modal) {
            modal.addEventListener("click", (e) => {
                if (e.target === modal) {
                    hideModal();
                }
            });
        }

        if (form) {
            form.addEventListener("submit", async (e) => {
                e.preventDefault();

                const gameSelect = document.getElementById("game-select") as HTMLSelectElement;
                const nameInput = document.getElementById("collection-name") as HTMLInputElement;

                const gameId = parseInt(gameSelect.value, 10);
                const name = nameInput.value.trim();

                if (!gameId || !name) {
                    showFormError("Please fill in all fields");
                    return;
                }

                try {
                    await createCollection(gameId, name);
                    hideModal();

                    // Reload collections
                    const collections = await loadCollections();
                    renderCollections(collections);
                } catch (error) {
                    showFormError(error instanceof Error ? error.message : "Failed to create collection");
                }
            });
        }
    }

    function showError(): void {
        const loadingEl = document.getElementById("collections-loading");
        const errorEl = document.getElementById("collections-error");

        if (loadingEl) loadingEl.hidden = true;
        if (errorEl) errorEl.hidden = false;
    }

    async function init(): Promise<void> {
        const isLoggedIn = await checkSession();
        setupNav(isLoggedIn);

        if (!isLoggedIn) {
            showError();
            return;
        }

        try {
            // Load games for the dropdown and collections in parallel
            const [games, collections] = await Promise.all([
                loadGames(),
                loadCollections(),
            ]);

            populateGameSelect(games);
            setupModal();
            renderCollections(collections);
        } catch (error) {
            console.error("Error loading data:", error);
            const loadingEl = document.getElementById("collections-loading");
            if (loadingEl) {
                loadingEl.innerHTML = "<p>Failed to load collections. Please try again later.</p>";
            }
        }
    }

    document.addEventListener("DOMContentLoaded", init);
})();
