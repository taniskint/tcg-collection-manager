(function () {
    interface Deck {
        id: number;
        collection_id: number;
        name: string;
        created_at: string;
        collection_name: string;
        game_name: string;
        game_image_url: string | null;
        card_count: number;
    }

    interface Collection {
        id: number;
        game_id: number;
        name: string;
        created_at: string;
        game_name: string;
        game_image_url: string | null;
    }

    interface DeckGroup {
        collection_id: number;
        collection_name: string;
        game_name: string;
        game_image_url: string | null;
        decks: Deck[];
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

    async function loadDecks(): Promise<Deck[]> {
        const response = await fetch("/api/decks");
        if (!response.ok) {
            throw new Error("Failed to load decks");
        }
        return response.json();
    }

    async function loadCollections(): Promise<Collection[]> {
        const response = await fetch("/api/collections");
        if (!response.ok) {
            throw new Error("Failed to load collections");
        }
        return response.json();
    }

    function formatDate(isoDate: string): string {
        const date = new Date(isoDate);
        return date.toLocaleDateString();
    }

    function groupDecksByCollection(decks: Deck[]): DeckGroup[] {
        const groupMap = new Map<number, DeckGroup>();

        for (const deck of decks) {
            if (!groupMap.has(deck.collection_id)) {
                groupMap.set(deck.collection_id, {
                    collection_id: deck.collection_id,
                    collection_name: deck.collection_name,
                    game_name: deck.game_name,
                    game_image_url: deck.game_image_url,
                    decks: [],
                });
            }
            groupMap.get(deck.collection_id)!.decks.push(deck);
        }

        return Array.from(groupMap.values());
    }

    function renderDecks(decks: Deck[]): void {
        const loadingEl = document.getElementById("decks-loading");
        const emptyEl = document.getElementById("decks-empty");
        const containerEl = document.getElementById("decks-container");

        if (!loadingEl || !emptyEl || !containerEl) return;

        loadingEl.hidden = true;

        if (decks.length === 0) {
            emptyEl.hidden = false;
            containerEl.hidden = true;
            return;
        }

        emptyEl.hidden = true;

        const groups = groupDecksByCollection(decks);

        containerEl.innerHTML = groups
            .map(
                (group) => `
                <div class="collection-group">
                    <div class="collection-group-header">
                        ${group.game_image_url ? `<img src="${group.game_image_url}" alt="${group.game_name}">` : ""}
                        <div>
                            <h2>${group.collection_name}</h2>
                            <span class="game-name">${group.game_name}</span>
                        </div>
                    </div>
                    <div class="decks-grid">
                        ${group.decks
                            .map(
                                (deck) => `
                            <a href="deck.html?id=${deck.id}" class="deck-card">
                                <h3>${deck.name}</h3>
                                <div class="deck-meta">
                                    <p>${deck.card_count} cards</p>
                                    <p>Created ${formatDate(deck.created_at)}</p>
                                </div>
                            </a>
                        `
                            )
                            .join("")}
                    </div>
                </div>
            `
            )
            .join("");

        containerEl.hidden = false;
    }

    function populateCollectionSelect(collections: Collection[]): void {
        const select = document.getElementById("collection-select") as HTMLSelectElement;
        if (!select) return;

        select.innerHTML = '<option value="">Select a collection...</option>';

        collections.forEach((collection) => {
            const option = document.createElement("option");
            option.value = collection.id.toString();
            option.textContent = `${collection.name} (${collection.game_name})`;
            select.appendChild(option);
        });
    }

    function showModal(): void {
        const modal = document.getElementById("create-modal");
        if (modal) modal.hidden = false;
    }

    function hideModal(): void {
        const modal = document.getElementById("create-modal");
        const form = document.getElementById("create-deck-form") as HTMLFormElement;
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

    async function createDeck(collectionId: number, name: string): Promise<void> {
        const response = await fetch("/api/decks", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ collection_id: collectionId, name }),
        });

        if (!response.ok) {
            const data = await response.json();
            throw new Error(data.error || "Failed to create deck");
        }
    }

    function setupModal(): void {
        const newBtn = document.getElementById("new-deck-btn");
        const closeBtn = document.getElementById("close-modal-btn");
        const cancelBtn = document.getElementById("cancel-btn");
        const modal = document.getElementById("create-modal");
        const form = document.getElementById("create-deck-form") as HTMLFormElement;

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

                const collectionSelect = document.getElementById("collection-select") as HTMLSelectElement;
                const nameInput = document.getElementById("deck-name") as HTMLInputElement;

                const collectionId = parseInt(collectionSelect.value, 10);
                const name = nameInput.value.trim();

                if (!collectionId || !name) {
                    showFormError("Please fill in all fields");
                    return;
                }

                try {
                    await createDeck(collectionId, name);
                    hideModal();

                    const decks = await loadDecks();
                    renderDecks(decks);
                } catch (error) {
                    showFormError(error instanceof Error ? error.message : "Failed to create deck");
                }
            });
        }
    }

    function showError(): void {
        const loadingEl = document.getElementById("decks-loading");
        const errorEl = document.getElementById("decks-error");

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
            const [collections, decks] = await Promise.all([
                loadCollections(),
                loadDecks(),
            ]);

            populateCollectionSelect(collections);
            setupModal();
            renderDecks(decks);
        } catch (error) {
            console.error("Error loading data:", error);
            const loadingEl = document.getElementById("decks-loading");
            if (loadingEl) {
                loadingEl.innerHTML = "<p>Failed to load decks. Please try again later.</p>";
            }
        }
    }

    document.addEventListener("DOMContentLoaded", init);
})();
