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
    }

    interface Card {
        id: number;
        name: string;
        collector_number: string;
        image_url: string | null;
        attributes: Record<string, string>;
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

    function getParams(): { gameId: number; setId: number } | null {
        const params = new URLSearchParams(window.location.search);
        const gameId = params.get("game");
        const setId = params.get("set");

        if (!gameId || !setId) {
            return null;
        }

        return {
            gameId: parseInt(gameId, 10),
            setId: parseInt(setId, 10),
        };
    }

    async function loadGame(gameId: number): Promise<Game> {
        const response = await fetch(`/api/games/${gameId}`);
        if (!response.ok) {
            throw new Error("Game not found");
        }
        return response.json();
    }

    async function loadSet(gameId: number, setId: number): Promise<Set> {
        const response = await fetch(`/api/games/${gameId}/sets/${setId}`);
        if (!response.ok) {
            throw new Error("Set not found");
        }
        return response.json();
    }

    async function loadCards(gameId: number, setId: number): Promise<Card[]> {
        const response = await fetch(`/api/games/${gameId}/sets/${setId}/cards`);
        if (!response.ok) {
            throw new Error("Failed to load cards");
        }
        return response.json();
    }

    function renderHeader(game: Game, set: Set, cardCount: number): void {
        const headerEl = document.getElementById("page-header");
        const breadcrumbGameEl = document.getElementById("breadcrumb-game") as HTMLAnchorElement;
        const breadcrumbSetEl = document.getElementById("breadcrumb-set");
        const logoEl = document.getElementById("set-logo") as HTMLImageElement;
        const nameEl = document.getElementById("set-name");
        const countEl = document.getElementById("card-count");

        if (!headerEl || !breadcrumbGameEl || !breadcrumbSetEl || !logoEl || !nameEl || !countEl) return;

        document.title = `${set.name} - TCG Collection Manager`;

        breadcrumbGameEl.textContent = game.name;
        breadcrumbGameEl.href = `sets.html?game=${game.id}`;
        breadcrumbSetEl.textContent = set.name;

        nameEl.textContent = set.name;
        countEl.textContent = `${cardCount} ${cardCount === 1 ? "card" : "cards"} in this set`;

        if (set.image_url) {
            logoEl.src = set.image_url;
            logoEl.alt = set.name;
            logoEl.hidden = false;
        }

        headerEl.hidden = false;
    }

    function renderCards(cards: Card[]): void {
        const loadingEl = document.getElementById("cards-loading");
        const emptyEl = document.getElementById("cards-empty");
        const gridEl = document.getElementById("cards-grid");

        if (!loadingEl || !emptyEl || !gridEl) return;

        loadingEl.hidden = true;

        if (cards.length === 0) {
            emptyEl.hidden = false;
            return;
        }

        gridEl.innerHTML = cards
            .map(
                (card) => `
                <div class="card-chiclet">
                    <img
                        src="${card.image_url || "images/placeholder-card.png"}"
                        alt="${card.name}"
                        class="card-image"
                    >
                    <div class="card-info">
                        <p class="card-name" title="${card.name}">${card.name}</p>
                        <p class="card-number">${card.collector_number}</p>
                    </div>
                </div>
            `
            )
            .join("");

        gridEl.hidden = false;
    }

    function showError(): void {
        const loadingEl = document.getElementById("cards-loading");
        const errorEl = document.getElementById("cards-error");

        if (loadingEl) loadingEl.hidden = true;
        if (errorEl) errorEl.hidden = false;
    }

    async function init(): Promise<void> {
        const isLoggedIn = await checkSession();
        setupNav(isLoggedIn);

        const params = getParams();
        if (!params) {
            showError();
            return;
        }

        try {
            const [game, set, cards] = await Promise.all([
                loadGame(params.gameId),
                loadSet(params.gameId, params.setId),
                loadCards(params.gameId, params.setId),
            ]);

            renderHeader(game, set, cards.length);
            renderCards(cards);
        } catch (error) {
            console.error("Error loading cards:", error);
            showError();
        }
    }

    document.addEventListener("DOMContentLoaded", init);
})();
