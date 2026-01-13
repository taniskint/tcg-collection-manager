use sha2::{Digest, Sha256};
use crate::deck::DeckCard;

#[derive(Debug)]
pub enum ImageError {
    LoadError(String),
    NetworkError(String),
    DecodeError(String),
    TaskJoinError,
}

#[derive(Debug)]
pub enum S3Error {
    UploadFailed(String),
    ConfigError(String),
}

#[derive(Debug)]
pub enum AtlasError {
    ImageError(ImageError),
    S3Error(S3Error),
    NoValidImages,
    EncodingError,
}

impl From<ImageError> for AtlasError {
    fn from(err: ImageError) -> Self {
        AtlasError::ImageError(err)
    }
}

impl From<S3Error> for AtlasError {
    fn from(err: S3Error) -> Self {
        AtlasError::S3Error(err)
    }
}

/// Calculate a content hash for a deck based on card IDs and quantities
/// This is used as a cache key for S3 storage
pub fn calculate_deck_hash(deck_cards: &[DeckCard]) -> String {
    let mut hasher = Sha256::new();

    // Sort cards by ID for consistency
    let mut sorted_cards: Vec<_> = deck_cards.iter().collect();
    sorted_cards.sort_by_key(|card| card.id);

    // Hash card IDs and quantities
    for card in sorted_cards {
        hasher.update(card.id.to_le_bytes());
        hasher.update(card.quantity.to_le_bytes());
    }

    // Return hex string
    format!("{:x}", hasher.finalize())
}

/// Generate S3 key for an atlas
/// Format: decks/{deck_id}/atlas-{hash}-{index}.png
pub fn get_s3_key(deck_id: i64, hash: &str, index: usize) -> String {
    format!("decks/{}/atlas-{}-{}.png", deck_id, hash, index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_card(id: i64, quantity: i64) -> DeckCard {
        crate::deck::DeckCard {
            id,
            name: "Test Card".to_string(),
            collector_number: "001".to_string(),
            image_url: Some("/test.png".to_string()),
            attributes: HashMap::new(),
            set_id: 1,
            set_name: "Test Set".to_string(),
            quantity,
        }
    }

    #[test]
    fn test_calculate_deck_hash_consistent() {
        let cards = vec![
            create_test_card(1, 4),
            create_test_card(2, 3),
            create_test_card(3, 2),
        ];

        let hash1 = calculate_deck_hash(&cards);
        let hash2 = calculate_deck_hash(&cards);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_calculate_deck_hash_order_independent() {
        let cards1 = vec![
            create_test_card(1, 4),
            create_test_card(2, 3),
        ];

        let cards2 = vec![
            create_test_card(2, 3),
            create_test_card(1, 4),
        ];

        let hash1 = calculate_deck_hash(&cards1);
        let hash2 = calculate_deck_hash(&cards2);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_calculate_deck_hash_changes_with_quantity() {
        let cards1 = vec![create_test_card(1, 4)];
        let cards2 = vec![create_test_card(1, 3)];

        let hash1 = calculate_deck_hash(&cards1);
        let hash2 = calculate_deck_hash(&cards2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_get_s3_key_format() {
        let key = get_s3_key(42, "abc123", 0);
        assert_eq!(key, "decks/42/atlas-abc123-0.png");

        let key2 = get_s3_key(100, "def456", 2);
        assert_eq!(key2, "decks/100/atlas-def456-2.png");
    }
}
