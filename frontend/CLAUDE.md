# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
npm install      # Install dependencies
npm run build    # Compile TypeScript to dist/
npm run watch    # Watch mode for development
```

## Architecture

This is the frontend for a TCG (Trading Card Game) Collection Manager - a multi-page app using vanilla HTML5, CSS3, and TypeScript (no frameworks like React/Vue/Angular).

### Key Design Decisions

- **Multi-page app**: Each page is a separate HTML file with its own CSS and TypeScript
- **Dual view pattern**: Pages show different content based on login state (e.g., index.html has both landing page and dashboard views)
- **Session-based auth**: Login state determined by `session_id` cookie
- **API proxy**: Frontend expects API at `/api/*` on the same origin (backend serves both static files and API)

### File Structure

- `src/*.ts` - TypeScript source files, compiled to `dist/*.js`
- `*.html` - Page files at root level
- `*.css` - Stylesheets (styles.css is shared, others are page-specific)
- `images/` - Static assets (logos, placeholders)

### CSS Design System

Uses "Penumbra" color scheme with CSS custom properties:
- Light/dark theme via `prefers-color-scheme` media query
- Semantic variables: `--bg-primary`, `--bg-elevated`, `--bg-recessed`, `--text-primary`, `--text-emphasis`, `--border`

## API Reference

See `../backend/openapi.yaml` for full API spec. Key endpoints:

- `GET /api/games` - List games
- `GET /api/games/{id}/sets` - List sets in a game
- `GET /api/games/{id}/sets/{id}/cards` - List cards in a set
- `POST /api/sessions` - Login (sets session_id cookie)
- `DELETE /api/sessions/{id}` - Logout
- `POST /api/users` - Register

Admin endpoints (require Bearer token): POST games, sets, cards.
