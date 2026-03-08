//! Markdown generation from OCR results
//!
//! This module converts recognized text regions into Markdown format.

use crate::core::error::{OcrError, Result};
use super::TextRegion;

/// Reading order for text regions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingOrder {
    /// Left to right, top to bottom (horizontal)
    Horizontal,
    /// Top to bottom, right to left (vertical - Japanese)
    Vertical,
    /// Mixed (both horizontal and vertical)
    Mixed,
}

impl Default for ReadingOrder {
    fn default() -> Self {
        ReadingOrder::Horizontal
    }
}

/// Generate Markdown from text regions
pub fn generate_markdown(regions: &[TextRegion]) -> Result<String> {
    if regions.is_empty() {
        return Ok(String::new());
    }
    
    // Determine reading order based on detected directions
    let reading_order = determine_reading_order(regions);
    
    // Sort regions according to reading order
    let mut sorted_regions = regions.to_vec();
    sort_by_reading_order(&mut sorted_regions, reading_order);
    
    // Generate markdown
    let mut markdown = String::new();
    let mut last_y = f32::NEG_INFINITY;
    let mut last_direction = 0;
    
    for region in sorted_regions {
        // Add paragraph break if there's a significant vertical gap
        if (region.bbox[1] - last_y).abs() > 50.0 {
            if !markdown.is_empty() {
                markdown.push_str("\n\n");
            }
        } else if last_direction != region.direction && !markdown.is_empty() {
            markdown.push_str("\n");
        }
        
        // Add text content
        if region.is_vertical {
            // For vertical text, we might want special formatting
            markdown.push_str(&format_text_vertical(&region.text));
        } else {
            markdown.push_str(&region.text);
        }
        
        last_y = region.bbox[1];
        last_direction = region.direction;
    }
    
    Ok(markdown)
}

/// Determine the overall reading order from text regions
fn determine_reading_order(regions: &[TextRegion]) -> ReadingOrder {
    let vertical_count = regions.iter().filter(|r| r.is_vertical).count();
    let horizontal_count = regions.len() - vertical_count;
    
    if vertical_count > horizontal_count {
        ReadingOrder::Vertical
    } else if horizontal_count > vertical_count {
        ReadingOrder::Horizontal
    } else if !regions.is_empty() {
        ReadingOrder::Mixed
    } else {
        ReadingOrder::default()
    }
}

/// Sort text regions by reading order
fn sort_by_reading_order(regions: &mut [TextRegion], order: ReadingOrder) {
    match order {
        ReadingOrder::Horizontal => {
            // Sort by y first (top to bottom), then x (left to right)
            regions.sort_by(|a, b| {
                let y_cmp = a.bbox[1].partial_cmp(&b.bbox[1]).unwrap_or(std::cmp::Ordering::Equal);
                if y_cmp != std::cmp::Ordering::Equal {
                    y_cmp
                } else {
                    a.bbox[0].partial_cmp(&b.bbox[0]).unwrap_or(std::cmp::Ordering::Equal)
                }
            });
        }
        ReadingOrder::Vertical => {
            // Sort by x first (right to left), then y (top to bottom)
            regions.sort_by(|a, b| {
                let x_cmp = b.bbox[0].partial_cmp(&a.bbox[0]).unwrap_or(std::cmp::Ordering::Equal);
                if x_cmp != std::cmp::Ordering::Equal {
                    x_cmp
                } else {
                    a.bbox[1].partial_cmp(&b.bbox[1]).unwrap_or(std::cmp::Ordering::Equal)
                }
            });
        }
        ReadingOrder::Mixed => {
            // For mixed, use a more complex algorithm
            // This is a simplified version - a full implementation would use
            // connected component analysis
            regions.sort_by(|a, b| {
                let y_cmp = a.bbox[1].partial_cmp(&b.bbox[1]).unwrap_or(std::cmp::Ordering::Equal);
                if y_cmp != std::cmp::Ordering::Equal {
                    y_cmp
                } else {
                    a.bbox[0].partial_cmp(&b.bbox[0]).unwrap_or(std::cmp::Ordering::Equal)
                }
            });
        }
    }
}

/// Format vertical text with special markers
fn format_text_vertical(text: &str) -> String {
    // For now, just return the text as-is
    // In a full implementation, we might add special formatting
    text.to_string()
}

/// Generate plain text from regions (for FTS indexing)
pub fn generate_plain_text(regions: &[TextRegion]) -> String {
    regions
        .iter()
        .map(|r| r.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Detect paragraph boundaries based on text layout
pub fn detect_paragraphs(regions: &[TextRegion], threshold: f32) -> Vec<Vec<TextRegion>> {
    if regions.is_empty() {
        return Vec::new();
    }
    
    let mut paragraphs = Vec::new();
    let mut current_paragraph = vec![regions[0].clone()];
    let mut last_y = regions[0].bbox[1];
    
    for region in regions.iter().skip(1) {
        let gap = (region.bbox[1] - last_y).abs();
        
        if gap > threshold {
            // Start a new paragraph
            paragraphs.push(current_paragraph);
            current_paragraph = vec![region.clone()];
        } else {
            current_paragraph.push(region.clone());
        }
        
        last_y = region.bbox[1];
    }
    
    // Don't forget the last paragraph
    if !current_paragraph.is_empty() {
        paragraphs.push(current_paragraph);
    }
    
    paragraphs
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_region(x: f32, y: f32, text: &str, is_vertical: bool) -> TextRegion {
        TextRegion {
            bbox: [x, y, x + 100.0, y + 20.0],
            text: text.to_string(),
            confidence: 0.9,
            direction: if is_vertical { 90 } else { 0 },
            is_vertical,
        }
    }
    
    #[test]
    fn test_generate_markdown_empty() {
        let regions: Vec<TextRegion> = Vec::new();
        let markdown = generate_markdown(&regions).unwrap();
        assert!(markdown.is_empty());
    }
    
    #[test]
    fn test_generate_markdown_horizontal() {
        let regions = vec![
            create_test_region(0.0, 0.0, "Hello", false),
            create_test_region(100.0, 0.0, "World", false),
        ];
        
        let markdown = generate_markdown(&regions).unwrap();
        assert!(markdown.contains("Hello"));
        assert!(markdown.contains("World"));
    }
    
    #[test]
    fn test_determine_reading_order() {
        let horizontal = vec![
            create_test_region(0.0, 0.0, "A", false),
            create_test_region(100.0, 0.0, "B", false),
        ];
        assert_eq!(determine_reading_order(&horizontal), ReadingOrder::Horizontal);
        
        let vertical = vec![
            create_test_region(0.0, 0.0, "A", true),
            create_test_region(0.0, 100.0, "B", true),
        ];
        assert_eq!(determine_reading_order(&vertical), ReadingOrder::Vertical);
    }
}