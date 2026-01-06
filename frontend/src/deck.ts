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

    interface DeckCard {
        id: number;
        name: string;
        collector_number: string;
        image_url: string | null;
        attributes: Record<string, string>;
        set_id: number;
        set_name: string;
        quantity: number;
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

    interface CardQuantityUpdate {
        card_id: number;
        quantity: number;
    }

    interface SortLevel {
        field: string;
        order: string;
    }

    // State
    let currentDeck: DeckDetail | null = null;
    let deckCards: DeckCard[] = [];
    let collectionCards: CollectionCard[] = [];
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

    async function loadDeckCards(deckId: number): Promise<DeckCard[]> {
        const response = await fetch(`/api/decks/${deckId}/cards`);
        if (!response.ok) {
            throw new Error("Failed to load deck cards");
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

    async function updateDeckCards(
        deckId: number,
        updates: CardQuantityUpdate[]
    ): Promise<void> {
        const response = await fetch(`/api/decks/${deckId}/cards`, {
            method: "PATCH",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(updates),
        });

        if (!response.ok) {
            throw new Error("Failed to update deck cards");
        }
    }

    function formatDate(isoDate: string): string {
        const date = new Date(isoDate);
        return date.toLocaleDateString();
    }

    function renderDeck(deck: DeckDetail): void {
        const loadingEl = document.getElementById("deck-loading");
        const headerEl = document.getElementById("page-header");
        const filtersEl = document.getElementById("filters-section");
        const sortingEl = document.getElementById("sorting-section");
        const breadcrumbEl = document.getElementById("breadcrumb-deck");
        const nameEl = document.getElementById("deck-name");
        const collectionEl = document.getElementById("deck-collection");
        const logoEl = document.getElementById("game-logo") as HTMLImageElement;
        const cardCountEl = document.getElementById("stat-cards");
        const createdEl = document.getElementById("stat-created");
        const addCardsBtn = document.getElementById("add-cards-btn") as HTMLButtonElement;

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
        if (filtersEl) filtersEl.hidden = false;
        if (sortingEl) sortingEl.hidden = false;
        if (addCardsBtn) addCardsBtn.disabled = false;
    }

    // Sorting functions
    function getSortLevels(): SortLevel[] {
        const levels: SortLevel[] = [];
        const sortGroups = document.querySelectorAll("#sorting-section .sort-group");

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

    function getCardSortValue(card: DeckCard, field: string): string | number {
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

    function sortCards(cards: DeckCard[]): DeckCard[] {
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

    // Filter functions
    function getFilteredCards(): DeckCard[] {
        const searchInput = document.getElementById("card-search") as HTMLInputElement;
        const setSelect = document.getElementById("filter-set") as HTMLSelectElement;

        const search = searchInput?.value.toLowerCase() || "";
        const setId = setSelect?.value ? parseInt(setSelect.value, 10) : null;

        // Get all dynamic attribute filters
        const attributeFilters: { key: string; value: string }[] = [];
        document.querySelectorAll("#attribute-filters .attribute-filter").forEach((select) => {
            const sel = select as HTMLSelectElement;
            const key = sel.dataset.attribute;
            const value = sel.value;
            if (key && value) {
                attributeFilters.push({ key, value });
            }
        });

        return deckCards.filter((card) => {
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
        deckCards.forEach((card) => {
            const value = card.attributes[key];
            if (value) {
                values.add(value);
            }
        });
        return Array.from(values).sort();
    }

    function extractAllAttributeKeys(): string[] {
        const keys = new Set<string>();
        deckCards.forEach((card) => {
            Object.keys(card.attributes).forEach((key) => keys.add(key));
        });
        return Array.from(keys).sort();
    }

    function populateFilterDropdowns(): void {
        const setSelect = document.getElementById("filter-set") as HTMLSelectElement;
        const attributeFiltersContainer = document.getElementById("attribute-filters");

        // Populate sets from deck cards
        if (setSelect) {
            setSelect.innerHTML = '<option value="">All Sets</option>';
            const setsInDeck = new Map<number, string>();
            deckCards.forEach((c) => setsInDeck.set(c.set_id, c.set_name));
            Array.from(setsInDeck.entries())
                .sort((a, b) => a[1].localeCompare(b[1]))
                .forEach(([id, name]) => {
                    const option = document.createElement("option");
                    option.value = id.toString();
                    option.textContent = name;
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

    function renderCardsGrid(cards: DeckCard[]): void {
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
        if (cardCountEl && currentDeck) {
            const totalCount = deckCards.reduce((sum, card) => sum + card.quantity, 0);
            cardCountEl.textContent = totalCount.toString();
        }
    }

    // Modal functions
    function getModalSortLevel(): SortLevel {
        const fieldSelect = document.getElementById("modal-sort-field") as HTMLSelectElement;
        const orderSelect = document.getElementById("modal-sort-order") as HTMLSelectElement;

        return {
            field: fieldSelect?.value || "id",
            order: orderSelect?.value || "asc",
        };
    }

    function getCollectionCardSortValue(card: CollectionCard, field: string): string | number {
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

    function sortModalCards(cards: CollectionCard[]): CollectionCard[] {
        const level = getModalSortLevel();

        return [...cards].sort((a, b) => {
            const aVal = getCollectionCardSortValue(a, level.field);
            const bVal = getCollectionCardSortValue(b, level.field);

            if (aVal < bVal) return level.order === "asc" ? -1 : 1;
            if (aVal > bVal) return level.order === "asc" ? 1 : -1;
            return 0;
        });
    }

    function getFilteredModalCards(): CollectionCard[] {
        const searchInput = document.getElementById("add-card-search") as HTMLInputElement;
        const setSelect = document.getElementById("add-card-set") as HTMLSelectElement;

        const search = searchInput?.value.toLowerCase() || "";
        const setId = setSelect?.value ? parseInt(setSelect.value, 10) : null;

        // Get modal attribute filters
        const attributeFilters: { key: string; value: string }[] = [];
        document.querySelectorAll("#modal-attribute-filters .modal-attr-filter").forEach((select) => {
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

    function getDeckCardQuantity(cardId: number): number {
        // Check pending updates first
        if (pendingUpdates.has(cardId)) {
            return pendingUpdates.get(cardId)!;
        }
        // Check existing deck cards
        const existing = deckCards.find((c) => c.id === cardId);
        return existing ? existing.quantity : 0;
    }

    function renderModalResults(cards: CollectionCard[]): void {
        const resultsEl = document.getElementById("add-cards-results");
        if (!resultsEl) return;

        const sortedCards = sortModalCards(cards);

        resultsEl.innerHTML = sortedCards
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
                        <input type="number" value="${getDeckCardQuantity(card.id)}" min="0" class="qty-input" data-card-id="${card.id}">
                        <button class="qty-btn qty-plus">+</button>
                        <span class="collection-qty">(${card.quantity} in collection)</span>
                    </div>
                </div>
            `
            )
            .join("");
    }

    function extractCollectionAttributeKeys(): string[] {
        const keys = new Set<string>();
        collectionCards.forEach((card) => {
            Object.keys(card.attributes).forEach((key) => keys.add(key));
        });
        return Array.from(keys).sort();
    }

    function extractCollectionUniqueValues(key: string): string[] {
        const values = new Set<string>();
        collectionCards.forEach((card) => {
            const value = card.attributes[key];
            if (value) {
                values.add(value);
            }
        });
        return Array.from(values).sort();
    }

    function populateModalFilters(): void {
        const setSelect = document.getElementById("add-card-set") as HTMLSelectElement;
        const attributeFiltersContainer = document.getElementById("modal-attribute-filters");

        // Populate sets from collection cards
        if (setSelect) {
            setSelect.innerHTML = '<option value="">All Sets</option>';
            const setsInCollection = new Map<number, string>();
            collectionCards.forEach((c) => setsInCollection.set(c.set_id, c.set_name));
            Array.from(setsInCollection.entries())
                .sort((a, b) => a[1].localeCompare(b[1]))
                .forEach(([id, name]) => {
                    const option = document.createElement("option");
                    option.value = id.toString();
                    option.textContent = name;
                    setSelect.appendChild(option);
                });
        }

        // Populate dynamic attribute filters
        if (attributeFiltersContainer) {
            attributeFiltersContainer.innerHTML = "";
            const attributeKeys = extractCollectionAttributeKeys();

            attributeKeys.forEach((key) => {
                const values = extractCollectionUniqueValues(key);
                if (values.length === 0) return;

                const group = document.createElement("div");
                group.className = "filter-group";
                group.innerHTML = `
                    <label for="modal-filter-${key}">${key}</label>
                    <select id="modal-filter-${key}" class="filter-select modal-attr-filter" data-attribute="${key}">
                        <option value="">All</option>
                        ${values.map((v) => `<option value="${v}">${v}</option>`).join("")}
                    </select>
                `;
                attributeFiltersContainer.appendChild(group);
            });
        }
    }

    function showModal(): void {
        const modal = document.getElementById("add-cards-modal");
        if (modal) {
            modal.hidden = false;
            pendingUpdates.clear();
            populateModalFilters();
            renderModalResults(getFilteredModalCards());
        }
    }

    function hideModal(): void {
        const modal = document.getElementById("add-cards-modal");
        const searchInput = document.getElementById("add-card-search") as HTMLInputElement;
        const setSelect = document.getElementById("add-card-set") as HTMLSelectElement;
        const sortField = document.getElementById("modal-sort-field") as HTMLSelectElement;
        const sortOrder = document.getElementById("modal-sort-order") as HTMLSelectElement;

        if (modal) modal.hidden = true;
        if (searchInput) searchInput.value = "";
        if (setSelect) setSelect.value = "";
        if (sortField) sortField.value = "id";
        if (sortOrder) sortOrder.value = "asc";
        pendingUpdates.clear();
    }

    async function submitModalChanges(): Promise<void> {
        if (!currentDeck) return;

        const updates: CardQuantityUpdate[] = [];
        pendingUpdates.forEach((quantity, cardId) => {
            updates.push({ card_id: cardId, quantity });
        });

        if (updates.length === 0) {
            hideModal();
            return;
        }

        try {
            await updateDeckCards(currentDeck.id, updates);
            // Reload deck cards
            deckCards = await loadDeckCards(currentDeck.id);
            populateFilterDropdowns();
            renderCardsGrid(sortCards(getFilteredCards()));
            updateCardCount();
            hideModal();
        } catch (error) {
            console.error("Error updating cards:", error);
            alert("Failed to update deck cards. Please try again.");
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
        const sortField = document.getElementById("modal-sort-field");
        const sortOrder = document.getElementById("modal-sort-order");
        const resultsEl = document.getElementById("add-cards-results");
        const attributeFiltersContainer = document.getElementById("modal-attribute-filters");

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
        const updateModalResults = () => {
            renderModalResults(getFilteredModalCards());
        };

        if (searchInput) {
            searchInput.addEventListener("input", updateModalResults);
        }

        if (setSelect) {
            setSelect.addEventListener("change", updateModalResults);
        }

        if (sortField) {
            sortField.addEventListener("change", updateModalResults);
        }

        if (sortOrder) {
            sortOrder.addEventListener("change", updateModalResults);
        }

        // Event delegation for dynamic attribute filters
        if (attributeFiltersContainer) {
            attributeFiltersContainer.addEventListener("change", (e) => {
                const target = e.target as HTMLElement;
                if (target.classList.contains("modal-attr-filter")) {
                    updateModalResults();
                }
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
                document.querySelectorAll("#attribute-filters .attribute-filter").forEach((select) => {
                    (select as HTMLSelectElement).value = "";
                });
                applyFilters();
            });
        }
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
            // Load deck details
            currentDeck = await loadDeck(deckId);
            renderDeck(currentDeck);

            // Load deck cards and collection cards in parallel
            const [cards, collCards] = await Promise.all([
                loadDeckCards(deckId),
                loadCollectionCards(currentDeck.collection_id),
            ]);

            deckCards = cards;
            collectionCards = collCards;

            // Setup UI
            populateFilterDropdowns();
            renderCardsGrid(sortCards(deckCards));
            setupFilterEventHandlers();
            setupModalEventHandlers();
        } catch (error) {
            console.error("Error loading deck:", error);
            showError();
        }
    }

    document.addEventListener("DOMContentLoaded", init);
})();
