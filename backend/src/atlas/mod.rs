mod image;
mod model;
mod s3;

pub use model::{calculate_deck_hash, get_s3_key, AtlasError};
pub use s3::{check_cache, create_s3_client, upload_atlas};

use crate::deck::DeckCard;

/// Generate texture atlases for a deck of cards
/// Returns a vector of atlases with their images and card counts
pub async fn generate_atlases(
    deck_cards: &[DeckCard],
) -> Result<Vec<image::Atlas>, AtlasError> {
    // 1. Load and deduplicate images
    let images = image::load_and_dedupe_images(deck_cards).await?;

    if images.is_empty() {
        return Err(AtlasError::NoValidImages);
    }

    // 2. Calculate common dimensions (divisible by 10 and 7)
    let (card_width, card_height) = image::calculate_common_dimensions(&images);

    // 3. Scale all images to common size
    let scaled_images = image::scale_images(images, card_width, card_height);

    // 4. Create 10x7 atlases
    let atlases = image::create_atlases(deck_cards, &scaled_images, card_width, card_height);

    Ok(atlases)
}
