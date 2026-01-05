(function () {
    interface DeckDetail {
        id: number;
        collection_id: number;
        name: string;
        created_at: string;
        collection_name: string;
        game_name: string;
        game_image_url: string | null;
        card_count: number;
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

    function getDeckId(): number | null {
        const params = new URLSearchParams(window.location.search);
        const id = params.get("id");
        return id ? parseInt(id, 10) : null;
    }

    async function loadDeck(id: number): Promise<DeckDetail> {
        const response = await fetch(`/api/decks/${id}`);
        if (!response.ok) {
            throw new Error("Deck not found");
        }
        return response.json();
    }

    function formatDate(isoDate: string): string {
        const date = new Date(isoDate);
        return date.toLocaleDateString();
    }

    function renderDeck(deck: DeckDetail): void {
        const loadingEl = document.getElementById("deck-loading");
        const headerEl = document.getElementById("page-header");
        const emptyEl = document.getElementById("cards-empty");
        const breadcrumbEl = document.getElementById("breadcrumb-deck");
        const nameEl = document.getElementById("deck-name");
        const collectionEl = document.getElementById("deck-collection");
        const logoEl = document.getElementById("game-logo") as HTMLImageElement;
        const cardCountEl = document.getElementById("stat-cards");
        const createdEl = document.getElementById("stat-created");

        if (loadingEl) loadingEl.hidden = true;

        if (breadcrumbEl) breadcrumbEl.textContent = deck.name;
        if (nameEl) nameEl.textContent = deck.name;
        if (collectionEl) collectionEl.textContent = `${deck.collection_name} · ${deck.game_name}`;
        if (cardCountEl) cardCountEl.textContent = deck.card_count.toString();
        if (createdEl) createdEl.textContent = formatDate(deck.created_at);

        if (logoEl) {
            if (deck.game_image_url) {
                logoEl.src = deck.game_image_url;
                logoEl.alt = deck.game_name;
            } else {
                logoEl.hidden = true;
            }
        }

        document.title = `${deck.name} - TCG Collection Manager`;

        if (headerEl) headerEl.hidden = false;

        // Show empty state since card management is not implemented yet
        if (emptyEl) emptyEl.hidden = false;
    }

    function showError(): void {
        const loadingEl = document.getElementById("deck-loading");
        const errorEl = document.getElementById("deck-error");

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

        const deckId = getDeckId();
        if (!deckId) {
            showError();
            return;
        }

        try {
            const deck = await loadDeck(deckId);
            renderDeck(deck);
        } catch (error) {
            console.error("Error loading deck:", error);
            showError();
        }
    }

    document.addEventListener("DOMContentLoaded", init);
})();
