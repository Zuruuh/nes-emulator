pub struct Bus {
    vram: [u8; 2048],
}

impl Default for Bus {
    fn default() -> Self {
        Self { vram: [0; 2048] }
    }
}
