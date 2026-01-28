use aorp::{DecisionConfig, DecisionEngine, EntropySource, MetricView, NeighborSet, PolicyConstraints};
use crate::modes::ModeConfig;

pub struct Router {
    engine: DecisionEngine,
}

impl Router {
    pub fn new() -> Self {
        Self {
            engine: DecisionEngine::new(DecisionConfig::new(Some(10))),
        }
    }

    pub fn decide_next_hop(
        &mut self,
        neighbors: Vec<String>,
        config: &ModeConfig,
    ) -> Option<String> {
        if neighbors.is_empty() {
            return None;
        }

        let neighbor_set = NeighborSet::from_peers(neighbors.iter().map(|s| s.as_str()));
        
        let metrics = MetricView::builder().build();
        
        let diversity = match config.hops {
            1 => aorp::interfaces::types::DiversityLevel::Low,
            2..=3 => aorp::interfaces::types::DiversityLevel::Medium,
            _ => aorp::interfaces::types::DiversityLevel::High,
        };

        let policies = PolicyConstraints::builder()
            .require_diversity(diversity)
            .latency_weight(2)
            .bandwidth_weight(1)
            .avoid_loops(true)
            .max_hops(config.hops)
            .build();

        let hop = self.engine.decide_next_hop(
            neighbor_set,
            metrics,
            EntropySource::secure_random(),
            policies,
        );

        hop.map(|h| h.0.0)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
