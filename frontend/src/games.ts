(function () {
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

    async function loadGames(): Promise<Game[]> {
        const response = await fetch("/api/games");
        if (!response.ok) {
            throw new Error("Failed to load games");
        }
        return response.json();
    }

    function renderGames(games: Game[]): void {
        const loadingEl = document.getElementById("games-loading");
        const emptyEl = document.getElementById("games-empty");
        const gridEl = document.getElementById("games-grid");

        if (!loadingEl || !emptyEl || !gridEl) return;

        loadingEl.hidden = true;

        if (games.length === 0) {
            emptyEl.hidden = false;
            return;
        }

        gridEl.innerHTML = games
            .map(
                (game) => `
                <a href="sets.html?game=${game.id}" class="game-card">
                    <div class="game-logo">
                        ${game.image_url ? `<img src="${game.image_url}" alt="${game.name}">` : ""}
                    </div>
                    <div class="game-info">
                        <h3>${game.name}</h3>
                        <p class="game-meta">${game.set_count} ${game.set_count === 1 ? "set" : "sets"} available</p>
                    </div>
                </a>
            `
            )
            .join("");

        gridEl.hidden = false;
    }

    async function init(): Promise<void> {
        const isLoggedIn = await checkSession();
        setupNav(isLoggedIn);

        try {
            const games = await loadGames();
            renderGames(games);
        } catch (error) {
            console.error("Error loading games:", error);
            const loadingEl = document.getElementById("games-loading");
            if (loadingEl) {
                loadingEl.innerHTML = "<p>Failed to load games. Please try again later.</p>";
            }
        }
    }

    document.addEventListener("DOMContentLoaded", init);
})();
