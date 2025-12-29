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

    // For now, if we have a session cookie, assume it's valid
    // A proper implementation would validate with the backend
    // e.g., GET /api/sessions/current or similar
    return true;
}

async function loadGames(): Promise<Game[]> {
    try {
        const response = await fetch("/api/games");
        if (!response.ok) {
            console.error("Failed to load games:", response.status);
            return [];
        }
        return await response.json();
    } catch (error) {
        console.error("Error loading games:", error);
        return [];
    }
}

function renderGameLogos(games: Game[]): void {
    const container = document.getElementById("game-logos");
    if (!container) return;

    if (games.length === 0) {
        container.innerHTML = "<p>No games available yet.</p>";
        return;
    }

    container.innerHTML = games
        .filter((game) => game.image_url)
        .map(
            (game) =>
                `<img src="${game.image_url}" alt="${game.name}" title="${game.name}">`
        )
        .join("");
}

function setupNavToggle(): void {
    const showLinksDiv = document.getElementById("show-links");
    const navLinksUL = document.getElementById("nav-links");

    if (showLinksDiv && navLinksUL) {
        showLinksDiv.addEventListener("click", () => {
            navLinksUL.hidden = !navLinksUL.hidden;
            showLinksDiv.textContent = navLinksUL.hidden ? "+" : "-";
        });
    }
}

async function init(): Promise<void> {
    const loggedOutView = document.getElementById("logged-out-view");
    const loggedInView = document.getElementById("logged-in-view");

    if (!loggedOutView || !loggedInView) {
        console.error("Could not find view containers");
        return;
    }

    const isLoggedIn = await checkSession();

    if (isLoggedIn) {
        loggedOutView.hidden = true;
        loggedInView.hidden = false;
        setupNavToggle();
    } else {
        loggedOutView.hidden = false;
        loggedInView.hidden = true;

        // Load games for the landing page
        const games = await loadGames();
        renderGameLogos(games);
    }
}

document.addEventListener("DOMContentLoaded", init);
