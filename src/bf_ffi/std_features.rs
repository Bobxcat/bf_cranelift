//! Standard features for `bf_ffi`

use enum_map::{Enum, EnumMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdFeatures {
    features: EnumMap<StdFeature, bool>,
}

impl StdFeatures {
    /// Creates a new feature set with only the `Core` feature enabled
    pub fn new() -> Self {
        Self {
            features: EnumMap::from_fn(|f| match f {
                StdFeature::Core => true,
                _ => false,
            }),
        }
    }

    pub fn with_feature(&mut self, f: StdFeature) -> &mut Self {
        self.features[f] = true;
        self
    }

    pub fn has_feature(&self, f: StdFeature) -> bool {
        self.features[f]
    }

    pub fn iter_features(&self) -> impl Iterator<Item = StdFeature> {
        self.features
            .into_iter()
            .filter_map(|(f, enabled)| enabled.then_some(f))
    }
}

#[derive(Debug, Clone, Copy, Enum)]
pub enum StdFeature {
    Core,
}
