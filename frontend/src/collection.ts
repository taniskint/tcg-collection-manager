(function () {
    interface CollectionDetail {
        id: number;
        game_id: number;
        name: string;
        created_at: string;
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

    function getCollectionId(): number | null {
        const params = new URLSearchParams(window.location.search);
        const id = params.get("id");
        return id ? parseInt(id, 10) : null;
    }

    async function loadCollection(id: number): Promise<CollectionDetail> {
        const response = await fetch(`/api/collections/${id}`);
        if (!response.ok) {
            throw new Error("Collection not found");
        }
        return response.json();
    }

    function formatDate(isoDate: string): string {
        const date = new Date(isoDate);
        return date.toLocaleDateString();
    }

    function renderCollection(collection: CollectionDetail): void {
        const loadingEl = document.getElementById("collection-loading");
        const headerEl = document.getElementById("page-header");
        const placeholderEl = document.getElementById("cards-placeholder");
        const breadcrumbEl = document.getElementById("breadcrumb-collection");
        const nameEl = document.getElementById("collection-name");
        const gameEl = document.getElementById("collection-game");
        const logoEl = document.getElementById("game-logo") as HTMLImageElement;
        const cardCountEl = document.getElementById("stat-cards");
        const createdEl = document.getElementById("stat-created");

        if (loadingEl) loadingEl.hidden = true;

        if (breadcrumbEl) breadcrumbEl.textContent = collection.name;
        if (nameEl) nameEl.textContent = collection.name;
        if (gameEl) gameEl.textContent = collection.game_name;
        if (cardCountEl) cardCountEl.textContent = collection.card_count.toString();
        if (createdEl) createdEl.textContent = formatDate(collection.created_at);

        if (logoEl) {
            if (collection.game_image_url) {
                logoEl.src = collection.game_image_url;
                logoEl.alt = collection.game_name;
            } else {
                logoEl.hidden = true;
            }
        }

        document.title = `${collection.name} - TCG Collection Manager`;

        if (headerEl) headerEl.hidden = false;
        if (placeholderEl) placeholderEl.hidden = false;
    }

    function showError(): void {
        const loadingEl = document.getElementById("collection-loading");
        const errorEl = document.getElementById("collection-error");

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

        const collectionId = getCollectionId();
        if (!collectionId) {
            showError();
            return;
        }

        try {
            const collection = await loadCollection(collectionId);
            renderCollection(collection);
        } catch (error) {
            console.error("Error loading collection:", error);
            showError();
        }
    }

    document.addEventListener("DOMContentLoaded", init);
})();
