//! File storage operations for images
//!
//! This module handles saving and loading images to/from the filesystem,
//! storing file paths in the database rather than BLOBs to avoid memory issues.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Storage service for managing image files
pub struct StorageService {
    assets_dir: PathBuf,
    images_dir: PathBuf,
}

impl StorageService {
    /// Create a new storage service with the specified assets directory
    /// Creates the images subdirectory if it doesn't exist
    pub fn new(assets_dir: PathBuf) -> Result<Self> {
        let images_dir = assets_dir.join("images");

        // Create images directory if it doesn't exist
        if !images_dir.exists() {
            fs::create_dir_all(&images_dir)
                .with_context(|| format!("Failed to create images directory: {:?}", images_dir))?;
        }

        Ok(Self {
            assets_dir,
            images_dir,
        })
    }

    /// Save image data to filesystem
    ///
    /// # Arguments
    /// * `data` - Raw image bytes
    /// * `prefix` - Prefix for the filename (e.g., "cover", "ocr")
    ///
    /// # Returns
    /// Relative path to the saved file (e.g., "images/cover_abc123.jpg")
    pub fn save_image(&self, data: &[u8], prefix: &str) -> Result<String> {
        // Generate UUID-based filename
        let filename = format!("{}_{}.bin", prefix, uuid::Uuid::new_v4());
        let file_path = self.images_dir.join(&filename);

        // Write file
        let mut file = fs::File::create(&file_path)
            .with_context(|| format!("Failed to create file: {:?}", file_path))?;

        file.write_all(data)
            .with_context(|| format!("Failed to write image data: {:?}", file_path))?;

        // Return relative path (not absolute)
        let relative_path = format!("images/{}", filename);
        Ok(relative_path)
    }

    /// Load image data from filesystem
    ///
    /// # Arguments
    /// * `path` - Relative path to the image (e.g., "images/cover_abc123.jpg")
    ///
    /// # Returns
    /// Raw image bytes
    pub fn get_image(&self, path: &str) -> Result<Vec<u8>> {
        let file_path = self.assets_dir.join(path);

        if !file_path.exists() {
            anyhow::bail!("Image file not found: {:?}", file_path);
        }

        let mut file = fs::File::open(&file_path)
            .with_context(|| format!("Failed to open file: {:?}", file_path))?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .with_context(|| format!("Failed to read file: {:?}", file_path))?;

        Ok(data)
    }

    /// Delete image from filesystem
    ///
    /// # Arguments
    /// * `path` - Relative path to the image
    pub fn delete_image(&self, path: &str) -> Result<()> {
        let file_path = self.assets_dir.join(path);

        if file_path.exists() {
            fs::remove_file(&file_path)
                .with_context(|| format!("Failed to delete file: {:?}", file_path))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_storage() -> (StorageService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageService::new(temp_dir.path().to_path_buf()).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_save_image_writes_file_to_correct_directory() {
        let (storage, _temp) = setup_storage();
        let image_data = b"fake image data";

        let result = storage.save_image(image_data, "test");
        assert!(result.is_ok());

        let relative_path = result.unwrap();
        assert!(relative_path.starts_with("images/"));

        // Verify file exists
        let full_path = storage.assets_dir.join(&relative_path);
        assert!(full_path.exists());
    }

    #[test]
    fn test_save_image_returns_relative_path() {
        let (storage, _temp) = setup_storage();
        let image_data = b"fake image data";

        let result = storage.save_image(image_data, "test").unwrap();

        // Should be relative, not absolute
        assert!(!result.starts_with('/'));
        assert!(!result.starts_with("C:"));
        assert!(result.starts_with("images/"));
    }

    #[test]
    fn test_get_image_reads_file_content_back() {
        let (storage, _temp) = setup_storage();
        let original_data = b"original image data";

        let path = storage.save_image(original_data, "test").unwrap();
        let loaded_data = storage.get_image(&path).unwrap();

        assert_eq!(original_data, loaded_data.as_slice());
    }

    #[test]
    fn test_delete_image_removes_file() {
        let (storage, _temp) = setup_storage();
        let image_data = b"image to delete";

        let path = storage.save_image(image_data, "test").unwrap();
        let full_path = storage.assets_dir.join(&path);
        assert!(full_path.exists());

        storage.delete_image(&path).unwrap();
        assert!(!full_path.exists());
    }

    #[test]
    fn test_get_image_returns_error_for_non_existent_file() {
        let (storage, _temp) = setup_storage();

        let result = storage.get_image("images/nonexistent.bin");
        assert!(result.is_err());
    }

    #[test]
    fn test_images_directory_created_if_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let assets_dir = temp_dir.path().to_path_buf();

        // Images dir should not exist yet
        let images_dir = assets_dir.join("images");
        assert!(!images_dir.exists());

        // Create storage - should create images dir
        let _storage = StorageService::new(assets_dir).unwrap();

        assert!(images_dir.exists());
    }
}
