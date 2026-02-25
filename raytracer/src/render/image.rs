#[derive(Debug, Clone, PartialEq)]
pub struct RenderImage {
    width: u32,
    height: u32,
    pixels_rgb8: Vec<u8>,
}

impl RenderImage {
    pub fn new_solid(width: u32, height: u32, color: [u8; 3]) -> Self {
        let pixel_count = (width as usize) * (height as usize);
        let mut pixels_rgb8 = Vec::with_capacity(pixel_count * 3);

        for _ in 0..pixel_count {
            pixels_rgb8.extend_from_slice(&color);
        }

        Self {
            width,
            height,
            pixels_rgb8,
        }
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub fn as_rgb8(&self) -> &[u8] {
        &self.pixels_rgb8
    }
}
