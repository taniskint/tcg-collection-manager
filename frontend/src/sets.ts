(function () {
    interface Game {
        id: number;
        name: string;
        image_url: string | null;
        set_count: number;
    }

    interface Set {
        id: number;
        name: string;
        image_url: string | null;
        publish_date: string;
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

    function getGameId(): number | null {
        const params = new URLSearchParams(window.location.search);
        const gameId = params.get("game");
        return gameId ? parseInt(gameId, 10) : null;
    }

    async function loadGame(gameId: number): Promise<Game> {
        const response = await fetch(`/api/games/${gameId}`);
        if (!response.ok) {
            throw new Error("Game not found");
        }
        return response.json();
    }

    async function loadSets(gameId: number): Promise<Set[]> {
        const response = await fetch(`/api/games/${gameId}/sets`);
        if (!response.ok) {
            throw new Error("Failed to load sets");
        }
        return response.json();
    }

    function renderHeader(game: Game): void {
        const headerEl = document.getElementById("page-header");
        const breadcrumbEl = document.getElementById("breadcrumb-game");
        const logoEl = document.getElementById("game-logo") as HTMLImageElement;
        const nameEl = document.getElementById("game-name");

        if (!headerEl || !breadcrumbEl || !logoEl || !nameEl) return;

        document.title = `${game.name} Sets - TCG Collection Manager`;
        breadcrumbEl.textContent = game.name;
        nameEl.textContent = game.name;

        if (game.image_url) {
            logoEl.src = game.image_url;
            logoEl.alt = game.name;
        } else {
            logoEl.hidden = true;
        }

        headerEl.hidden = false;
    }

    function renderSets(sets: Set[], gameId: number): void {
        const loadingEl = document.getElementById("sets-loading");
        const emptyEl = document.getElementById("sets-empty");
        const gridEl = document.getElementById("sets-grid");

        if (!loadingEl || !emptyEl || !gridEl) return;

        loadingEl.hidden = true;

        if (sets.length === 0) {
            emptyEl.hidden = false;
            return;
        }

        gridEl.innerHTML = sets
            .map(
                (set) => `
                <a href="cards.html?game=${gameId}&set=${set.id}" class="set-card">
                    ${set.image_url ? `<div class="set-logo"><img src="${set.image_url}" alt="${set.name}"></div>` : ""}
                    <div class="set-info">
                        <h3>${set.name}</h3>
                    </div>
                </a>
            `
            )
            .join("");

        gridEl.hidden = false;
    }

    function showError(): void {
        const loadingEl = document.getElementById("sets-loading");
        const errorEl = document.getElementById("sets-error");

        if (loadingEl) loadingEl.hidden = true;
        if (errorEl) errorEl.hidden = false;
    }

    async function init(): Promise<void> {
        const isLoggedIn = await checkSession();
        setupNav(isLoggedIn);

        const gameId = getGameId();
        if (!gameId) {
            showError();
            return;
        }

        try {
            const [game, sets] = await Promise.all([
                loadGame(gameId),
                loadSets(gameId),
            ]);

            // Sort sets by publish date (newest first)
            sets.sort((a, b) => {
                return new Date(b.publish_date).getTime() - new Date(a.publish_date).getTime();
            });

            renderHeader(game);
            renderSets(sets, gameId);
        } catch (error) {
            console.error("Error loading sets:", error);
            showError();
        }
    }

    document.addEventListener("DOMContentLoaded", init);
})();
