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

    pub fn set_pixel_rgb8(&mut self, x: u32, y: u32, color: [u8; 3]) {
        assert!(x < self.width, "pixel x is out of bounds");
        assert!(y < self.height, "pixel y is out of bounds");

        let offset = ((y as usize) * (self.width as usize) + (x as usize)) * 3;
        self.pixels_rgb8[offset] = color[0];
        self.pixels_rgb8[offset + 1] = color[1];
        self.pixels_rgb8[offset + 2] = color[2];
    }

    pub fn pixel_rgb8(&self, x: u32, y: u32) -> [u8; 3] {
        assert!(x < self.width, "pixel x is out of bounds");
        assert!(y < self.height, "pixel y is out of bounds");

        let offset = ((y as usize) * (self.width as usize) + (x as usize)) * 3;
        [
            self.pixels_rgb8[offset],
            self.pixels_rgb8[offset + 1],
            self.pixels_rgb8[offset + 2],
        ]
    }
}
