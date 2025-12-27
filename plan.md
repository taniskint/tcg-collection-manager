# TCG Collection Manager

## Overview

This will be a website that will enable people to manage collections for various trading card games, such as Pokemon or Magic the Gathering. Users will be able to track multiple collections, even for the same game, and construct decks from their collections.

There'll also be able to "open" booster packs from a set into a collection. Multiple kinds of booster packs for each set will be supported (such as a "draft booster" vs a "collector booster"), but the majority of sets will only have one booster pack.

It's important that everything is data driven, so that new games, sets, and cards can be added after the fact. Database inserts are acceptable, but an "admin" dashboard would also be cool.

## Models

### User

This represents a user account that signs into the website. It must have things like an ID, a username, an email, and a password.

### Session

This represents a currently logged in session. It will be related to users as a many-to-one relation (one user, multiple sessions). All that's necessary is a unique session ID that will be sent to the client as a cookie, a user ID as a foreign key, and an expiration.

### Game

This represents a specific trading card game, such as Magic the Gathering. It should include an ID, a name, and an image name/URL to use for the game's logo.

### Set

Trading card games frequently release in sets (though the specific nomenclature may differ from game to game). "Innistrad" is an example from Magic the Gathering, and "Shrouded Fable" is an example from the Pokemon TCG. Sets are related to games as a many-to-one relation (one game, multiple sets). A set should include an ID, a name, a game ID as a foreign key, and an optional image name/URL for a logo. Most games use images specific to each set to distinguish them (such as Magic the Gathering), but not all do.

### Card

Cards are the main part of trading card games. They're related to sets as a many-to-one relation (one set, many cards). A card should include an ID, a name, a set ID as a foreign key, an image name/URL, and a set of "attributes". Attributes are essentially tags that can be used to construct booster packs. The most common is rarity, but Magic the Gathering could also include a card's color as an attribute so that booster packs contain at least one card of each color. 

### Collection

A collection represents any grouping of cards across multiple sets from a single game. It should include an ID, a name, a user ID as a foreign key, a game ID as a foreign key, and a many-to-many relationship with cards. A collection contains many cards, each with a quantity (i.e. 2 Bulbasaur and 3 Charizard).

### Deck

A deck is a specific subset of cards from a collection. It should include an ID, a name, a user ID as a foreign key, a collection ID as a foreign key, and a many-to-many relationship with cards, similar to a collection. If possible, the database structure should enforce that a card listed in a deck must also be part of its corresponding collection.

### Booster Pack

A booster pack is a common way that trading card games distribute new cards. Almost all booster packs operate with a "slots" paradigm. As an example: in a standard Pokemon TCG booster pack, there are ten slots. The first four slots are common cards. The next three slots are uncommon slots. The following two slots are usually a holographic version of a common, uncommon, or rare card, but also have a small chance to be an alternate-art version of a card in the set. The final card is usually a rare card, but has a chance to be a double-rare card, a small chance to be an ultra-rare card, and a very small chance to be a hyper-rare card.

In this app, a booster pack is a structured way of adding cards from a set into a collection. A booster pack should have an ID, a name, a set ID as a foreign key, and a structure detailing the slots and probabilities of the booster pack, potentially as a JSON blob.

## Technology Stack

### Frontend

I would like this web app to be a multi-page app writting with HTML5, CSS3, and vanilla TypeScript. It should not utilize libraries such as Vue, React, or Angular, and it needn't include server-side rendering.

### Backend

The backend for the web app is two parts:

The first part is a REST API responsible for the functionality of the app. I would like it to be written in Rust with `rocket`, though if that proves infeasible or intractable we can instead write it in TypeScript with Oak, targeting the Deno runtime.

The second part is simply a way to serve static content such as HTML, CSS, JavaScript, and images. It can be an Nginx server, a separate Rust or TypeScript server, or simply rolled into the first part as a separate set of routes.

If possible, I'd like to construct the backend to run as a Docker container or set of Docker containers. If it's a set of Docker containers (such as an Nginx container, a REST API container, and a SQLite container), it should also include a top-level Docker Compose file to easily start them locally.

## Example User Stories

### Recording my Real Life Collection

I collect trading card games physically, but I'd like to create a record of what I have in a digital format so it's easier to get an overview and search.

### Theorycrafting Decks

I play trading card games physically, but buying a bunch of cards in order to build decks is expensive. I'd like a tool to build decks digitally so I can experiment and only buy physical cards when I'm sure of what I want to make.

### Playing Digitally

I'm interested in trading card games, but I don't have anyone near me to play with. I'd like a platform to "collect" cards and use them to build a deck. Then I can use an external platform (like Tabletop Simulator) to play with my friend using the limited set of cards I "own".
