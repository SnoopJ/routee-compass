use crate::model::cost::cost::Cost;
use crate::model::traversal::state::traversal_state::TraversalState;

pub struct AccessResult {
    pub cost: Cost,
    pub updated_state: Option<TraversalState>,
}

impl AccessResult {
    pub fn no_cost() -> AccessResult {
        return AccessResult {
            cost: Cost::ZERO,
            updated_state: None,
        };
    }
}
