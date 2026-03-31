use image::{imageops, DynamicImage, GenericImageView, RgbaImage};
use std::collections::HashMap;
use std::path::Path;
use tokio::task;

use crate::deck::DeckCard;
use super::model::ImageError;

const ATLAS_COLS: u32 = 10;
const ATLAS_ROWS: u32 = 7;
const CARDS_PER_ATLAS: usize = (ATLAS_COLS * ATLAS_ROWS) as usize;
const MAX_ATLAS_WIDTH: u32 = 4096;
const MAX_CARD_WIDTH: u32 = MAX_ATLAS_WIDTH / ATLAS_COLS; // 409px

pub struct Atlas {
    pub image: RgbaImage,
    pub card_count: usize,
}

/// Load and deduplicate images from deck cards
/// Returns a HashMap of card_id -> image
pub async fn load_and_dedupe_images(
    deck_cards: &[DeckCard],
    frontend_path: &str,
) -> Result<HashMap<i64, DynamicImage>, ImageError> {
    // Deduplicate by image URL
    let mut url_to_id: HashMap<String, i64> = HashMap::new();
    for card in deck_cards {
        if let Some(ref url) = card.image_url
            && !url.is_empty()
        {
            url_to_id.entry(url.clone()).or_insert(card.id);
        }
    }

    // Load images in parallel
    let mut tasks = Vec::new();
    for (url, card_id) in url_to_id.into_iter() {
        let frontend_path_owned = frontend_path.to_string();
        let task = task::spawn(async move {
            let result = load_image(&url, &frontend_path_owned).await;
            (card_id, url, result)
        });
        tasks.push(task);
    }

    // Collect results
    let mut images = HashMap::new();
    for task in tasks {
        let (card_id, url, result) = task
            .await
            .map_err(|_| ImageError::TaskJoin)?;

        match result {
            Ok(img) => {
                images.insert(card_id, img);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load image {}: {:?}", url, e);
            }
        }
    }

    Ok(images)
}

/// Load a single image from local filesystem or remote URL
async fn load_image(url: &str, frontend_path: &str) -> Result<DynamicImage, ImageError> {
    // Check if it's a local path
    let path = format!("{}{}", frontend_path, url);

    if Path::new(&path).exists() {
        // Load from local filesystem
        let path_clone = path.clone();
        task::spawn_blocking(move || {
            image::open(&path_clone)
                .map_err(|_| ImageError::Load)
        })
        .await
        .map_err(|_| ImageError::TaskJoin)?
    } else {
        // Try loading from remote URL
        let url_string = url.to_string();
        let response = reqwest::get(&url_string)
            .await
            .map_err(|_| ImageError::Network)?;

        let bytes = response
            .bytes()
            .await
            .map_err(|_| ImageError::Network)?;

        let bytes_vec = bytes.to_vec();
        task::spawn_blocking(move || {
            image::load_from_memory(&bytes_vec)
                .map_err(|_| ImageError::Decode)
        })
        .await
        .map_err(|_| ImageError::TaskJoin)?
    }
}

/// Calculate common dimensions for all cards
/// Finds minimum dimensions across all images, then makes them divisible by 10 and 7
/// Also enforces maximum card width of 409px to keep atlas width <= 4096px
pub fn calculate_common_dimensions(images: &HashMap<i64, DynamicImage>) -> (u32, u32) {
    let mut min_width = u32::MAX;
    let mut min_height = u32::MAX;

    for img in images.values() {
        let (w, h) = img.dimensions();
        min_width = min_width.min(w);
        min_height = min_height.min(h);
    }

    // Enforce maximum card width
    min_width = min_width.min(MAX_CARD_WIDTH);

    // Make divisible by 10 (round down)
    let card_width = (min_width / ATLAS_COLS) * ATLAS_COLS;

    // Calculate height to maintain aspect ratio, then make divisible by 7
    let aspect_ratio = min_height as f32 / min_width as f32;
    let estimated_height = (card_width as f32 * aspect_ratio) as u32;
    let card_height = (estimated_height / ATLAS_ROWS) * ATLAS_ROWS;

    (card_width, card_height.max(ATLAS_ROWS))
}

/// Scale all images to common dimensions
/// Never upscales, only downscales
pub fn scale_images(
    images: HashMap<i64, DynamicImage>,
    target_width: u32,
    target_height: u32,
) -> HashMap<i64, RgbaImage> {
    images
        .into_iter()
        .map(|(id, img)| {
            let (w, h) = img.dimensions();

            // Only downscale, never upscale
            let scaled = if w > target_width || h > target_height {
                img.resize_exact(target_width, target_height, imageops::FilterType::Lanczos3)
            } else {
                img
            };

            (id, scaled.to_rgba8())
        })
        .collect()
}

/// Create 10x7 texture atlases from deck cards
/// Handles multiple atlases for decks with > 70 cards
pub fn create_atlases(
    deck_cards: &[DeckCard],
    scaled_images: &HashMap<i64, RgbaImage>,
    card_width: u32,
    card_height: u32,
) -> Vec<Atlas> {
    // Expand deck cards into flat list based on quantity
    let mut flat_cards = Vec::new();
    for card in deck_cards {
        for _ in 0..card.quantity {
            flat_cards.push(card.id);
        }
    }

    // Create atlases in chunks of 70 cards
    let mut atlases = Vec::new();
    for chunk in flat_cards.chunks(CARDS_PER_ATLAS) {
        let atlas_width = card_width * ATLAS_COLS;
        let atlas_height = card_height * ATLAS_ROWS;
        let mut atlas_image = RgbaImage::new(atlas_width, atlas_height);

        // Fill atlas grid
        for (i, card_id) in chunk.iter().enumerate() {
            if let Some(card_img) = scaled_images.get(card_id) {
                let col = (i as u32) % ATLAS_COLS;
                let row = (i as u32) / ATLAS_COLS;
                let x = col * card_width;
                let y = row * card_height;

                imageops::overlay(&mut atlas_image, card_img, x.into(), y.into());
            }
        }

        atlases.push(Atlas {
            image: atlas_image,
            card_count: chunk.len(),
        });
    }

    atlases
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_common_dimensions_divisible() {
        let mut images = HashMap::new();

        // Create a mock image with dimensions 800x600
        let img = DynamicImage::new_rgba8(800, 600);
        images.insert(1, img);

        let (width, height) = calculate_common_dimensions(&images);

        // Width should be divisible by 10
        assert_eq!(width % ATLAS_COLS, 0);
        // Height should be divisible by 7
        assert_eq!(height % ATLAS_ROWS, 0);
        // Should not exceed max card width
        assert!(width <= MAX_CARD_WIDTH);
    }

    #[test]
    fn test_calculate_common_dimensions_max_width() {
        let mut images = HashMap::new();

        // Create a very large image
        let img = DynamicImage::new_rgba8(5000, 4000);
        images.insert(1, img);

        let (width, _) = calculate_common_dimensions(&images);

        // Should enforce max card width
        assert!(width <= MAX_CARD_WIDTH);
    }

    #[test]
    fn test_scale_images_only_downscales() {
        let mut images = HashMap::new();

        // Small image that shouldn't be upscaled
        let small_img = DynamicImage::new_rgba8(100, 100);
        images.insert(1, small_img);

        // Large image that should be downscaled
        let large_img = DynamicImage::new_rgba8(1000, 800);
        images.insert(2, large_img);

        let scaled = scale_images(images, 400, 300);

        // Small image dimensions should remain <= target
        let small_scaled = scaled.get(&1).unwrap();
        assert!(small_scaled.width() <= 100 && small_scaled.height() <= 100);

        // Large image should be exactly target size
        let large_scaled = scaled.get(&2).unwrap();
        assert_eq!(large_scaled.width(), 400);
        assert_eq!(large_scaled.height(), 300);
    }
}
