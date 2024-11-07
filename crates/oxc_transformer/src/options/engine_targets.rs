use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use browserslist::Version;
use rustc_hash::FxHashMap;
use serde::Deserialize;

use oxc_diagnostics::Error;

use super::{
    babel::BabelTargets,
    es_features::{features, ESFeature},
    BrowserslistQuery,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    Chrome,
    Deno,
    Edge,
    Firefox,
    Hermes,
    Ie,
    Ios,
    Node,
    Opera,
    Rhino,
    Safari,
    Samsung,
    // TODO: electron to chromium
    Electron,
    // TODO: how to handle? There is a `op_mob` key below.
    OperaMobile,
    // TODO:
    Android,
}

impl FromStr for Engine {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "chrome" | "and_chr" => Ok(Self::Chrome),
            "deno" => Ok(Self::Deno),
            "edge" => Ok(Self::Edge),
            "firefox" | "and_ff" => Ok(Self::Firefox),
            "hermes" => Ok(Self::Hermes),
            "ie" | "ie_mob" => Ok(Self::Ie),
            "ios" | "ios_saf" => Ok(Self::Ios),
            "node" => Ok(Self::Node),
            "opera" | "op_mob" => Ok(Self::Opera),
            "rhino" => Ok(Self::Rhino),
            "safari" => Ok(Self::Safari),
            "samsung" => Ok(Self::Samsung),
            "electron" => Ok(Self::Electron),
            "opera_mobile" => Ok(Self::OperaMobile),
            "android" => Ok(Self::Android),
            _ => Err(()),
        }
    }
}

/// A map of engine names to minimum supported versions.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(try_from = "BabelTargets")]
pub struct EngineTargets(FxHashMap<Engine, Version>);

impl Deref for EngineTargets {
    type Target = FxHashMap<Engine, Version>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EngineTargets {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl EngineTargets {
    pub fn new(map: FxHashMap<Engine, Version>) -> Self {
        Self(map)
    }
    /// # Errors
    ///
    /// * Query is invalid.
    pub fn try_from_query(query: &str) -> Result<Self, Error> {
        BrowserslistQuery::Single(query.to_string()).exec()
    }

    /// Returns true if all fields are [None].
    pub fn is_any_target(&self) -> bool {
        self.0.is_empty()
    }

    pub fn has_feature(&self, feature: ESFeature) -> bool {
        self.should_enable(&features()[&feature])
    }

    pub fn should_enable(&self, engine_targets: &EngineTargets) -> bool {
        for (engine, version) in &engine_targets.0 {
            if let Some(v) = self.0.get(engine) {
                if v < version {
                    return true;
                }
            }
        }
        false
    }

    /// Parses the value returned from `browserslist`.
    pub fn parse_versions(versions: Vec<(String, String)>) -> Self {
        let mut engine_targets = Self::default();
        for (engine, version) in versions {
            let Ok(engine) = Engine::from_str(&engine) else {
                continue;
            };
            let Ok(version) = Version::from_str(&version) else {
                continue;
            };
            engine_targets
                .0
                .entry(engine)
                .and_modify(|v| {
                    if version < *v {
                        *v = version;
                    }
                })
                .or_insert(version);
        }
        engine_targets
    }
}
