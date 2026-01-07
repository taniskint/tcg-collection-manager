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

    interface Booster {
        id: number;
        name: string;
    }

    interface Collection {
        id: number;
        name: string;
        game_id: number;
    }

    interface OpenedCard {
        id: number;
        name: string;
        collector_number: string;
        image_url: string | null;
        attributes: Record<string, string>;
        quantity: number;
    }

    // State
    let boosters: Booster[] = [];
    let collections: Collection[] = [];
    let currentGameId: number = 0;

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

    async function loadBoosters(gameId: number, setId: number): Promise<Booster[]> {
        const response = await fetch(`/api/games/${gameId}/sets/${setId}/boosters`);
        if (!response.ok) return [];
        return response.json();
    }

    async function loadCollections(): Promise<Collection[]> {
        const response = await fetch("/api/collections");
        if (!response.ok) return [];
        return response.json();
    }

    async function openPacks(boosterId: number, collectionId: number, count: number): Promise<OpenedCard[]> {
        const response = await fetch(`/api/boosters/${boosterId}/open`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ collection_id: collectionId, count }),
        });
        if (!response.ok) throw new Error("Failed to open packs");
        const data = await response.json();
        return data.cards;
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

    function showOpenPackModal(): void {
        const packTypeSelect = document.getElementById("pack-type") as HTMLSelectElement;
        const collectionSelect = document.getElementById("pack-collection") as HTMLSelectElement;

        // Populate boosters
        packTypeSelect.innerHTML = '<option value="">Select a pack...</option>' +
            boosters.map(b => `<option value="${b.id}">${b.name}</option>`).join("");

        // Populate collections (filtered by game)
        const gameCollections = collections.filter(c => c.game_id === currentGameId);
        collectionSelect.innerHTML = '<option value="">Select a collection...</option>' +
            gameCollections.map(c => `<option value="${c.id}">${c.name}</option>`).join("");

        // Reset quantity
        (document.getElementById("pack-quantity") as HTMLInputElement).value = "1";

        document.getElementById("open-pack-modal")!.hidden = false;
    }

    function hideOpenPackModal(): void {
        document.getElementById("open-pack-modal")!.hidden = true;
    }

    function showResultsModal(packName: string, count: number, collectionName: string, cards: OpenedCard[]): void {
        const totalCards = cards.reduce((sum, c) => sum + c.quantity, 0);

        document.getElementById("results-title")!.textContent =
            `You opened ${count} ${packName}${count > 1 ? "s" : ""}!`;

        document.getElementById("results-summary")!.innerHTML =
            `<strong>${totalCards} cards</strong> added to <strong>${collectionName}</strong>`;

        const grid = document.getElementById("pack-results-grid")!;
        grid.innerHTML = cards.map(card => `
            <div class="pack-card">
                ${card.image_url
                    ? `<img src="${card.image_url}" alt="${card.name}" class="pack-card-image">`
                    : '<div class="pack-card-image"></div>'}
                <div class="pack-card-info">
                    <p class="pack-card-name">${card.name}</p>
                    <p class="pack-card-meta">${card.quantity > 1 ? `x${card.quantity} • ` : ""}${card.collector_number}</p>
                </div>
            </div>
        `).join("");

        document.getElementById("pack-results-modal")!.hidden = false;
    }

    function hideResultsModal(): void {
        document.getElementById("pack-results-modal")!.hidden = true;
    }

    function setupBoosterEventHandlers(): void {
        const openPackBtn = document.getElementById("open-pack-btn");
        const openPackModal = document.getElementById("open-pack-modal");
        const packModalClose = document.getElementById("pack-modal-close");
        const packModalCancel = document.getElementById("pack-modal-cancel");
        const openPackForm = document.getElementById("open-pack-form");
        const packResultsModal = document.getElementById("pack-results-modal");
        const resultsModalClose = document.getElementById("results-modal-close");
        const resultsOpenAnother = document.getElementById("results-open-another");
        const resultsDone = document.getElementById("results-done");

        openPackBtn?.addEventListener("click", showOpenPackModal);
        packModalClose?.addEventListener("click", hideOpenPackModal);
        packModalCancel?.addEventListener("click", hideOpenPackModal);

        openPackModal?.addEventListener("click", (e) => {
            if (e.target === openPackModal) hideOpenPackModal();
        });

        openPackForm?.addEventListener("submit", async (e) => {
            e.preventDefault();
            const boosterId = parseInt((document.getElementById("pack-type") as HTMLSelectElement).value);
            const collectionId = parseInt((document.getElementById("pack-collection") as HTMLSelectElement).value);
            const count = parseInt((document.getElementById("pack-quantity") as HTMLInputElement).value);

            const booster = boosters.find(b => b.id === boosterId);
            const collection = collections.find(c => c.id === collectionId);

            if (!booster || !collection) return;

            try {
                const cards = await openPacks(boosterId, collectionId, count);
                hideOpenPackModal();
                showResultsModal(booster.name, count, collection.name, cards);
            } catch (err) {
                console.error("Failed to open packs:", err);
            }
        });

        resultsModalClose?.addEventListener("click", hideResultsModal);
        resultsDone?.addEventListener("click", hideResultsModal);

        resultsOpenAnother?.addEventListener("click", () => {
            hideResultsModal();
            showOpenPackModal();
        });

        packResultsModal?.addEventListener("click", (e) => {
            if (e.target === packResultsModal) hideResultsModal();
        });
    }

    async function init(): Promise<void> {
        const isLoggedIn = await checkSession();
        setupNav(isLoggedIn);

        const params = getParams();
        if (!params) {
            showError();
            return;
        }

        // Store gameId for filtering collections
        currentGameId = params.gameId;

        try {
            const [game, set, cards] = await Promise.all([
                loadGame(params.gameId),
                loadSet(params.gameId, params.setId),
                loadCards(params.gameId, params.setId),
            ]);

            renderHeader(game, set, cards.length);
            renderCards(cards);

            // Load boosters and collections for the booster pack feature
            if (isLoggedIn) {
                const [loadedBoosters, loadedCollections] = await Promise.all([
                    loadBoosters(params.gameId, params.setId),
                    loadCollections(),
                ]);

                boosters = loadedBoosters;
                collections = loadedCollections;

                // Enable the button if boosters are available
                const openPackBtn = document.getElementById("open-pack-btn") as HTMLButtonElement;
                if (openPackBtn && boosters.length > 0) {
                    openPackBtn.disabled = false;
                }
            }

            setupBoosterEventHandlers();
        } catch (error) {
            console.error("Error loading cards:", error);
            showError();
        }
    }

    document.addEventListener("DOMContentLoaded", init);
})();
