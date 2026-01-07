#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageBuffer, Rgba};

    #[test]
    fn test_collision_polygon_square() {
        let mut img = ImageBuffer::new(10, 10);
        // Draw a 3x3 square at 3,3 (indices 3,4,5)
        for x in 3..=5 {
            for y in 3..=5 {
                img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
        let dynamic_img = DynamicImage::ImageRgba8(img);

        let polygon = crate::sprites::images::generate_collision_polygon(&dynamic_img);
        assert!(polygon.is_some(), "Should generate a polygon");
        let points = polygon.unwrap();

        println!("Generated points: {:?}", points);

        // A 3x3 square should likely have 4 points (corners) or 8 points depending on trace
        // Start point is likely (3,3).
        // Logic check: contours > 1 point.
        assert!(
            points.len() >= 3,
            "Polygon should have at least 3 points, got {}",
            points.len()
        );
    }
}
