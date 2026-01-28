#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingMode {
    Fast,
    Mix,
    Adaptive,
}

#[derive(Debug, Clone)]
pub struct ModeConfig {
    pub mode: RoutingMode,
    pub hops: u8,
    pub cover_traffic: bool,
    pub jitter_ms: Option<u64>,
    pub padding_size: usize,
}

impl ModeConfig {
    pub fn fast() -> Self {
        Self {
            mode: RoutingMode::Fast,
            hops: 1,
            cover_traffic: false,
            jitter_ms: None,
            padding_size: 64,
        }
    }

    pub fn mix() -> Self {
        Self {
            mode: RoutingMode::Mix,
            hops: 4,
            cover_traffic: true,
            jitter_ms: Some(200),
            padding_size: 512,
        }
    }

    pub fn adaptive() -> Self {
        Self {
            mode: RoutingMode::Adaptive,
            hops: 2,
            cover_traffic: false,
            jitter_ms: Some(50),
            padding_size: 256,
        }
    }

    pub fn custom(mode: RoutingMode, hops: u8) -> Self {
        match mode {
            RoutingMode::Fast => Self::fast(),
            RoutingMode::Mix => Self::mix(),
            RoutingMode::Adaptive => Self::adaptive(),
        }
        .with_hops(hops)
    }

    pub fn with_hops(mut self, hops: u8) -> Self {
        self.hops = hops;
        self
    }

    pub fn with_cover_traffic(mut self, enabled: bool) -> Self {
        self.cover_traffic = enabled;
        self
    }

    pub fn with_jitter(mut self, ms: Option<u64>) -> Self {
        self.jitter_ms = ms;
        self
    }
}
