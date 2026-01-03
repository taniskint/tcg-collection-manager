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

    interface CollectionCard {
        id: number;
        name: string;
        collector_number: string;
        image_url: string | null;
        attributes: Record<string, string>;
        set_id: number;
        set_name: string;
        quantity: number;
    }

    interface GameSet {
        id: number;
        name: string;
        code: string;
    }

    interface GameCard {
        id: number;
        name: string;
        collector_number: string;
        image_url: string | null;
        attributes: Record<string, string>;
        set_id: number;
        set_name: string;
    }

    interface CardQuantityUpdate {
        card_id: number;
        quantity: number;
    }

    // State
    let currentCollection: CollectionDetail | null = null;
    let collectionCards: CollectionCard[] = [];
    let gameSets: GameSet[] = [];
    let allGameCards: GameCard[] = [];
    let pendingUpdates: Map<number, number> = new Map();

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

    async function loadCollectionCards(collectionId: number): Promise<CollectionCard[]> {
        const response = await fetch(`/api/collections/${collectionId}/cards`);
        if (!response.ok) {
            throw new Error("Failed to load collection cards");
        }
        return response.json();
    }

    async function loadGameSets(gameId: number): Promise<GameSet[]> {
        const response = await fetch(`/api/games/${gameId}/sets`);
        if (!response.ok) {
            throw new Error("Failed to load sets");
        }
        return response.json();
    }

    async function loadSetCards(gameId: number, setId: number): Promise<GameCard[]> {
        const response = await fetch(`/api/games/${gameId}/sets/${setId}/cards`);
        if (!response.ok) {
            throw new Error("Failed to load cards");
        }
        const cards = await response.json();
        // Add set info to each card
        const set = gameSets.find((s) => s.id === setId);
        return cards.map((c: GameCard) => ({
            ...c,
            set_id: setId,
            set_name: set?.name || "",
        }));
    }

    async function loadAllGameCards(gameId: number): Promise<GameCard[]> {
        const sets = await loadGameSets(gameId);
        gameSets = sets;

        const cardPromises = sets.map((set) => loadSetCards(gameId, set.id));
        const cardsArrays = await Promise.all(cardPromises);
        return cardsArrays.flat();
    }

    async function updateCollectionCards(
        collectionId: number,
        updates: CardQuantityUpdate[]
    ): Promise<void> {
        const response = await fetch(`/api/collections/${collectionId}/cards`, {
            method: "PATCH",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(updates),
        });

        if (!response.ok) {
            throw new Error("Failed to update collection cards");
        }
    }

    function formatDate(isoDate: string): string {
        const date = new Date(isoDate);
        return date.toLocaleDateString();
    }

    function renderCollection(collection: CollectionDetail): void {
        const loadingEl = document.getElementById("collection-loading");
        const headerEl = document.getElementById("page-header");
        const filtersEl = document.getElementById("filters-section");
        const sortingEl = document.getElementById("sorting-section");
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
        if (filtersEl) filtersEl.hidden = false;
        if (sortingEl) sortingEl.hidden = false;
    }

    interface SortLevel {
        field: string;
        order: string;
    }

    function getSortLevels(): SortLevel[] {
        const levels: SortLevel[] = [];
        const sortGroups = document.querySelectorAll(".sort-group");

        sortGroups.forEach((group) => {
            const fieldSelect = group.querySelector(".sort-field") as HTMLSelectElement;
            const orderSelect = group.querySelector(".order-select") as HTMLSelectElement;

            if (fieldSelect && orderSelect) {
                levels.push({
                    field: fieldSelect.value || "id",
                    order: orderSelect.value || "asc",
                });
            }
        });

        return levels.length > 0 ? levels : [{ field: "id", order: "asc" }];
    }

    function getCardSortValue(card: CollectionCard, field: string): string | number {
        switch (field) {
            case "id":
                return card.id;
            case "name":
                return card.name.toLowerCase();
            case "collector_number":
                return card.collector_number;
            case "quantity":
                return card.quantity;
            case "set_name":
                return card.set_name.toLowerCase();
            default:
                return card.id;
        }
    }

    function sortCards(cards: CollectionCard[]): CollectionCard[] {
        const levels = getSortLevels();

        const sorted = [...cards].sort((a, b) => {
            for (const level of levels) {
                const aVal = getCardSortValue(a, level.field);
                const bVal = getCardSortValue(b, level.field);

                if (aVal < bVal) return level.order === "asc" ? -1 : 1;
                if (aVal > bVal) return level.order === "asc" ? 1 : -1;
            }
            return 0;
        });

        return sorted;
    }

    function addSortLevel(): void {
        const sortingRows = document.getElementById("sorting-rows");
        if (!sortingRows) return;

        const existingGroups = sortingRows.querySelectorAll(".sort-group");
        const newLevel = existingGroups.length;

        const group = document.createElement("div");
        group.className = "sort-group";
        group.dataset.sortLevel = newLevel.toString();
        group.innerHTML = `
            <label>Then by</label>
            <select class="filter-select sort-field">
                <option value="id">Default</option>
                <option value="name">Name</option>
                <option value="collector_number">Card Number</option>
                <option value="quantity">Quantity</option>
                <option value="set_name">Set</option>
            </select>
            <select class="filter-select order-select">
                <option value="asc">Asc</option>
                <option value="desc">Desc</option>
            </select>
            <button class="btn-icon sort-remove" title="Remove sort level">&times;</button>
        `;

        sortingRows.appendChild(group);
        renderCardsGrid(sortCards(getFilteredCards()));
    }

    function removeSortLevel(button: HTMLElement): void {
        const group = button.closest(".sort-group");
        if (group) {
            group.remove();
            renderCardsGrid(sortCards(getFilteredCards()));
        }
    }

    function getFilteredCards(): CollectionCard[] {
        const searchInput = document.getElementById("card-search") as HTMLInputElement;
        const setSelect = document.getElementById("filter-set") as HTMLSelectElement;

        const search = searchInput?.value.toLowerCase() || "";
        const setId = setSelect?.value ? parseInt(setSelect.value, 10) : null;

        // Get all dynamic attribute filters
        const attributeFilters: { key: string; value: string }[] = [];
        document.querySelectorAll(".attribute-filter").forEach((select) => {
            const sel = select as HTMLSelectElement;
            const key = sel.dataset.attribute;
            const value = sel.value;
            if (key && value) {
                attributeFilters.push({ key, value });
            }
        });

        return collectionCards.filter((card) => {
            if (search && !card.name.toLowerCase().includes(search)) {
                return false;
            }
            if (setId && card.set_id !== setId) {
                return false;
            }
            // Check all attribute filters
            for (const filter of attributeFilters) {
                if (card.attributes[filter.key] !== filter.value) {
                    return false;
                }
            }
            return true;
        });
    }

    function extractUniqueValues(key: string): string[] {
        const values = new Set<string>();
        collectionCards.forEach((card) => {
            const value = card.attributes[key];
            if (value) {
                values.add(value);
            }
        });
        return Array.from(values).sort();
    }

    function extractAllAttributeKeys(): string[] {
        const keys = new Set<string>();
        collectionCards.forEach((card) => {
            Object.keys(card.attributes).forEach((key) => keys.add(key));
        });
        return Array.from(keys).sort();
    }

    function populateFilterDropdowns(): void {
        const setSelect = document.getElementById("filter-set") as HTMLSelectElement;
        const attributeFiltersContainer = document.getElementById("attribute-filters");

        // Populate sets
        if (setSelect) {
            setSelect.innerHTML = '<option value="">All Sets</option>';
            const setsInCollection = new Set(collectionCards.map((c) => c.set_id));
            gameSets
                .filter((s) => setsInCollection.has(s.id))
                .forEach((set) => {
                    const option = document.createElement("option");
                    option.value = set.id.toString();
                    option.textContent = set.name;
                    setSelect.appendChild(option);
                });
        }

        // Populate dynamic attribute filters
        if (attributeFiltersContainer) {
            attributeFiltersContainer.innerHTML = "";
            const attributeKeys = extractAllAttributeKeys();

            attributeKeys.forEach((key) => {
                const values = extractUniqueValues(key);
                if (values.length === 0) return;

                const group = document.createElement("div");
                group.className = "filter-group";
                group.innerHTML = `
                    <label for="filter-${key}">${key}</label>
                    <select id="filter-${key}" class="filter-select attribute-filter" data-attribute="${key}">
                        <option value="">All</option>
                        ${values.map((v) => `<option value="${v}">${v}</option>`).join("")}
                    </select>
                `;
                attributeFiltersContainer.appendChild(group);
            });
        }
    }

    function renderCardsGrid(cards: CollectionCard[]): void {
        const gridEl = document.getElementById("cards-grid");
        const emptyEl = document.getElementById("cards-empty");

        if (!gridEl || !emptyEl) return;

        if (cards.length === 0) {
            gridEl.hidden = true;
            emptyEl.hidden = false;
            return;
        }

        emptyEl.hidden = true;
        gridEl.innerHTML = cards
            .map(
                (card) => `
                <div class="card-chiclet">
                    <div class="card-image-wrapper">
                        ${
                            card.image_url
                                ? `<img src="${card.image_url}" alt="${card.name}" class="card-image">`
                                : '<div class="card-image"></div>'
                        }
                        <span class="quantity-badge">x${card.quantity}</span>
                    </div>
                    <div class="card-info">
                        <p class="card-name">${card.name}</p>
                        <p class="card-number">${card.collector_number}</p>
                    </div>
                </div>
            `
            )
            .join("");
        gridEl.hidden = false;
    }

    function updateCardCount(): void {
        const cardCountEl = document.getElementById("stat-cards");
        if (cardCountEl && currentCollection) {
            const totalCount = collectionCards.reduce((sum, card) => sum + card.quantity, 0);
            cardCountEl.textContent = totalCount.toString();
        }
    }

    // Modal functions
    function getFilteredModalCards(): GameCard[] {
        const searchInput = document.getElementById("add-card-search") as HTMLInputElement;
        const setSelect = document.getElementById("add-card-set") as HTMLSelectElement;

        const search = searchInput?.value.toLowerCase() || "";
        const setId = setSelect?.value ? parseInt(setSelect.value, 10) : null;

        return allGameCards.filter((card) => {
            if (search && !card.name.toLowerCase().includes(search)) {
                return false;
            }
            if (setId && card.set_id !== setId) {
                return false;
            }
            return true;
        });
    }

    function getCardQuantity(cardId: number): number {
        // Check pending updates first
        if (pendingUpdates.has(cardId)) {
            return pendingUpdates.get(cardId)!;
        }
        // Check existing collection cards
        const existing = collectionCards.find((c) => c.id === cardId);
        return existing ? existing.quantity : 0;
    }

    function renderModalResults(cards: GameCard[]): void {
        const resultsEl = document.getElementById("add-cards-results");
        if (!resultsEl) return;

        resultsEl.innerHTML = cards
            .map(
                (card) => `
                <div class="add-card-item" data-card-id="${card.id}">
                    ${
                        card.image_url
                            ? `<img src="${card.image_url}" alt="${card.name}" class="add-card-image">`
                            : '<div class="add-card-image"></div>'
                    }
                    <div class="add-card-info">
                        <p class="add-card-name">${card.name}</p>
                        <p class="add-card-meta">${card.collector_number} &bull; ${card.set_name}</p>
                    </div>
                    <div class="add-card-quantity">
                        <button class="qty-btn qty-minus">-</button>
                        <input type="number" value="${getCardQuantity(card.id)}" min="0" class="qty-input" data-card-id="${card.id}">
                        <button class="qty-btn qty-plus">+</button>
                    </div>
                </div>
            `
            )
            .join("");
    }

    function populateModalSetFilter(): void {
        const setSelect = document.getElementById("add-card-set") as HTMLSelectElement;
        if (!setSelect) return;

        setSelect.innerHTML = '<option value="">All Sets</option>';
        gameSets.forEach((set) => {
            const option = document.createElement("option");
            option.value = set.id.toString();
            option.textContent = set.name;
            setSelect.appendChild(option);
        });
    }

    function showModal(): void {
        const modal = document.getElementById("add-cards-modal");
        if (modal) {
            modal.hidden = false;
            pendingUpdates.clear();
            renderModalResults(getFilteredModalCards());
        }
    }

    function hideModal(): void {
        const modal = document.getElementById("add-cards-modal");
        const searchInput = document.getElementById("add-card-search") as HTMLInputElement;
        const setSelect = document.getElementById("add-card-set") as HTMLSelectElement;

        if (modal) modal.hidden = true;
        if (searchInput) searchInput.value = "";
        if (setSelect) setSelect.value = "";
        pendingUpdates.clear();
    }

    async function submitModalChanges(): Promise<void> {
        if (!currentCollection) return;

        const updates: CardQuantityUpdate[] = [];
        pendingUpdates.forEach((quantity, cardId) => {
            updates.push({ card_id: cardId, quantity });
        });

        if (updates.length === 0) {
            hideModal();
            return;
        }

        try {
            await updateCollectionCards(currentCollection.id, updates);
            // Reload collection cards
            collectionCards = await loadCollectionCards(currentCollection.id);
            populateFilterDropdowns();
            renderCardsGrid(sortCards(getFilteredCards()));
            updateCardCount();
            hideModal();
        } catch (error) {
            console.error("Error updating cards:", error);
            alert("Failed to update collection cards. Please try again.");
        }
    }

    function setupModalEventHandlers(): void {
        const modal = document.getElementById("add-cards-modal");
        const addCardsBtn = document.getElementById("add-cards-btn");
        const closeBtn = document.getElementById("modal-close");
        const cancelBtn = document.getElementById("modal-cancel");
        const submitBtn = document.getElementById("modal-submit");
        const searchInput = document.getElementById("add-card-search");
        const setSelect = document.getElementById("add-card-set");
        const resultsEl = document.getElementById("add-cards-results");

        if (addCardsBtn) {
            addCardsBtn.addEventListener("click", showModal);
        }

        if (closeBtn) {
            closeBtn.addEventListener("click", hideModal);
        }

        if (cancelBtn) {
            cancelBtn.addEventListener("click", hideModal);
        }

        if (submitBtn) {
            submitBtn.addEventListener("click", submitModalChanges);
        }

        // Close on overlay click
        if (modal) {
            modal.addEventListener("click", (e) => {
                if (e.target === modal) {
                    hideModal();
                }
            });
        }

        // Search and filter handlers
        if (searchInput) {
            searchInput.addEventListener("input", () => {
                renderModalResults(getFilteredModalCards());
            });
        }

        if (setSelect) {
            setSelect.addEventListener("change", () => {
                renderModalResults(getFilteredModalCards());
            });
        }

        // Quantity button handlers (using event delegation)
        if (resultsEl) {
            resultsEl.addEventListener("click", (e) => {
                const target = e.target as HTMLElement;
                if (!target.classList.contains("qty-btn")) return;

                const cardItem = target.closest(".add-card-item");
                const input = cardItem?.querySelector(".qty-input") as HTMLInputElement;
                if (!input) return;

                const cardId = parseInt(input.dataset.cardId || "0", 10);
                let value = parseInt(input.value, 10) || 0;

                if (target.classList.contains("qty-plus")) {
                    value++;
                } else if (target.classList.contains("qty-minus") && value > 0) {
                    value--;
                }

                input.value = value.toString();
                pendingUpdates.set(cardId, value);
            });

            resultsEl.addEventListener("change", (e) => {
                const target = e.target as HTMLInputElement;
                if (!target.classList.contains("qty-input")) return;

                const cardId = parseInt(target.dataset.cardId || "0", 10);
                const value = Math.max(0, parseInt(target.value, 10) || 0);
                target.value = value.toString();
                pendingUpdates.set(cardId, value);
            });
        }
    }

    function setupFilterEventHandlers(): void {
        const searchInput = document.getElementById("card-search");
        const setSelect = document.getElementById("filter-set");
        const clearBtn = document.getElementById("clear-filters");
        const filtersRow = document.getElementById("basic-filters");
        const sortingRows = document.getElementById("sorting-rows");
        const addSortBtn = document.getElementById("add-sort-level");

        const applyFilters = () => {
            renderCardsGrid(sortCards(getFilteredCards()));
        };

        if (searchInput) {
            searchInput.addEventListener("input", applyFilters);
        }

        if (setSelect) {
            setSelect.addEventListener("change", applyFilters);
        }

        // Event delegation for dynamic attribute filters
        if (filtersRow) {
            filtersRow.addEventListener("change", (e) => {
                const target = e.target as HTMLElement;
                if (target.classList.contains("attribute-filter")) {
                    applyFilters();
                }
            });
        }

        // Event delegation for sorting controls
        if (sortingRows) {
            sortingRows.addEventListener("change", (e) => {
                const target = e.target as HTMLElement;
                if (target.classList.contains("sort-field") || target.classList.contains("order-select")) {
                    applyFilters();
                }
            });

            sortingRows.addEventListener("click", (e) => {
                const target = e.target as HTMLElement;
                if (target.classList.contains("sort-remove")) {
                    removeSortLevel(target);
                }
            });
        }

        if (addSortBtn) {
            addSortBtn.addEventListener("click", addSortLevel);
        }

        if (clearBtn) {
            clearBtn.addEventListener("click", () => {
                (document.getElementById("card-search") as HTMLInputElement).value = "";
                (document.getElementById("filter-set") as HTMLSelectElement).value = "";
                // Reset all dynamic attribute filters
                document.querySelectorAll(".attribute-filter").forEach((select) => {
                    (select as HTMLSelectElement).value = "";
                });
                applyFilters();
            });
        }
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
            // Load collection details
            currentCollection = await loadCollection(collectionId);
            renderCollection(currentCollection);

            // Load game data (sets and cards) and collection cards in parallel
            const [cards, gameCards] = await Promise.all([
                loadCollectionCards(collectionId),
                loadAllGameCards(currentCollection.game_id),
            ]);

            collectionCards = cards;
            allGameCards = gameCards;

            // Setup UI
            populateFilterDropdowns();
            populateModalSetFilter();
            renderCardsGrid(sortCards(collectionCards));
            setupFilterEventHandlers();
            setupModalEventHandlers();
        } catch (error) {
            console.error("Error loading collection:", error);
            showError();
        }
    }

    document.addEventListener("DOMContentLoaded", init);
})();
