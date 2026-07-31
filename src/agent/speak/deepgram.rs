//! Deepgram Speak provider settings for the Voice Agent.
//!
//! Mirrors `asyncapi/schemas/agent/speak-providers/deepgram.yml`.
//!
//! Note: this provider's `model` enum is independent from the SDK's
//! top-level `crate::speak::options::Model` used by the Speak REST API.
//! Phase 8 of the spec-coverage rollout reshapes the top-level model list;
//! this enum will be kept in sync at that time.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Deepgram TTS as the Voice Agent's Speak provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DeepgramSpeakProvider {
    /// REST API version of Deepgram TTS.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<DeepgramSpeakVersion>,

    /// TTS voice model.
    pub model: DeepgramSpeakModel,

    /// Speaking rate multiplier (0.7 – 1.5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
}

impl DeepgramSpeakProvider {
    /// Construct with the given model and no speed override.
    pub fn new(model: DeepgramSpeakModel) -> Self {
        Self {
            version: None,
            model,
            speed: None,
        }
    }
}

/// Version of the Deepgram TTS REST API used by the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DeepgramSpeakVersion {
    /// REST API v1.
    #[serde(rename = "v1")]
    V1,
}

/// Deepgram TTS voice model.
///
/// Includes the Aura-1 English voices and the Aura-2 English/Spanish voices
/// listed in the AsyncAPI spec at the time this SDK was built. Use
/// [`DeepgramSpeakModel::Other`] to pass any value not yet enumerated.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum DeepgramSpeakModel {
    AuraAsteriaEn,
    AuraLunaEn,
    AuraStellaEn,
    AuraAthenaEn,
    AuraHeraEn,
    AuraOrionEn,
    AuraArcasEn,
    AuraPerseusEn,
    AuraAngusEn,
    AuraOrpheusEn,
    AuraHeliosEn,
    AuraZeusEn,

    Aura2AmaltheaEn,
    Aura2AndromedaEn,
    Aura2ApolloEn,
    Aura2ArcasEn,
    Aura2AriesEn,
    Aura2AsteriaEn,
    Aura2AthenaEn,
    Aura2AtlasEn,
    Aura2AuroraEn,
    Aura2CallistaEn,
    Aura2CoraEn,
    Aura2CordeliaEn,
    Aura2DeliaEn,
    Aura2DracoEn,
    Aura2ElectraEn,
    Aura2HarmoniaEn,
    Aura2HelenaEn,
    Aura2HeraEn,
    Aura2HermesEn,
    Aura2HyperionEn,
    Aura2IrisEn,
    Aura2JanusEn,
    Aura2JunoEn,
    Aura2JupiterEn,
    Aura2LunaEn,
    Aura2MarsEn,
    Aura2MinervaEn,
    Aura2NeptuneEn,
    Aura2OdysseusEn,
    Aura2OpheliaEn,
    Aura2OrionEn,
    Aura2OrpheusEn,
    Aura2PandoraEn,
    Aura2PhoebeEn,
    Aura2PlutoEn,
    Aura2SaturnEn,
    Aura2SeleneEn,
    Aura2ThaliaEn,
    Aura2TheiaEn,
    Aura2VestaEn,
    Aura2ZeusEn,

    Aura2SirioEs,
    Aura2NestorEs,
    Aura2CarinaEs,
    Aura2CelesteEs,
    Aura2AlvaroEs,
    Aura2DianaEs,
    Aura2AquilaEs,
    Aura2SelenaEs,
    Aura2EstrellaEs,
    Aura2JavierEs,

    /// Forward-compatibility escape — pass any unrecognized model identifier.
    Other(String),
}

impl DeepgramSpeakModel {
    /// Wire string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::AuraAsteriaEn => "aura-asteria-en",
            Self::AuraLunaEn => "aura-luna-en",
            Self::AuraStellaEn => "aura-stella-en",
            Self::AuraAthenaEn => "aura-athena-en",
            Self::AuraHeraEn => "aura-hera-en",
            Self::AuraOrionEn => "aura-orion-en",
            Self::AuraArcasEn => "aura-arcas-en",
            Self::AuraPerseusEn => "aura-perseus-en",
            Self::AuraAngusEn => "aura-angus-en",
            Self::AuraOrpheusEn => "aura-orpheus-en",
            Self::AuraHeliosEn => "aura-helios-en",
            Self::AuraZeusEn => "aura-zeus-en",

            Self::Aura2AmaltheaEn => "aura-2-amalthea-en",
            Self::Aura2AndromedaEn => "aura-2-andromeda-en",
            Self::Aura2ApolloEn => "aura-2-apollo-en",
            Self::Aura2ArcasEn => "aura-2-arcas-en",
            Self::Aura2AriesEn => "aura-2-aries-en",
            Self::Aura2AsteriaEn => "aura-2-asteria-en",
            Self::Aura2AthenaEn => "aura-2-athena-en",
            Self::Aura2AtlasEn => "aura-2-atlas-en",
            Self::Aura2AuroraEn => "aura-2-aurora-en",
            Self::Aura2CallistaEn => "aura-2-callista-en",
            Self::Aura2CoraEn => "aura-2-cora-en",
            Self::Aura2CordeliaEn => "aura-2-cordelia-en",
            Self::Aura2DeliaEn => "aura-2-delia-en",
            Self::Aura2DracoEn => "aura-2-draco-en",
            Self::Aura2ElectraEn => "aura-2-electra-en",
            Self::Aura2HarmoniaEn => "aura-2-harmonia-en",
            Self::Aura2HelenaEn => "aura-2-helena-en",
            Self::Aura2HeraEn => "aura-2-hera-en",
            Self::Aura2HermesEn => "aura-2-hermes-en",
            Self::Aura2HyperionEn => "aura-2-hyperion-en",
            Self::Aura2IrisEn => "aura-2-iris-en",
            Self::Aura2JanusEn => "aura-2-janus-en",
            Self::Aura2JunoEn => "aura-2-juno-en",
            Self::Aura2JupiterEn => "aura-2-jupiter-en",
            Self::Aura2LunaEn => "aura-2-luna-en",
            Self::Aura2MarsEn => "aura-2-mars-en",
            Self::Aura2MinervaEn => "aura-2-minerva-en",
            Self::Aura2NeptuneEn => "aura-2-neptune-en",
            Self::Aura2OdysseusEn => "aura-2-odysseus-en",
            Self::Aura2OpheliaEn => "aura-2-ophelia-en",
            Self::Aura2OrionEn => "aura-2-orion-en",
            Self::Aura2OrpheusEn => "aura-2-orpheus-en",
            Self::Aura2PandoraEn => "aura-2-pandora-en",
            Self::Aura2PhoebeEn => "aura-2-phoebe-en",
            Self::Aura2PlutoEn => "aura-2-pluto-en",
            Self::Aura2SaturnEn => "aura-2-saturn-en",
            Self::Aura2SeleneEn => "aura-2-selene-en",
            Self::Aura2ThaliaEn => "aura-2-thalia-en",
            Self::Aura2TheiaEn => "aura-2-theia-en",
            Self::Aura2VestaEn => "aura-2-vesta-en",
            Self::Aura2ZeusEn => "aura-2-zeus-en",

            Self::Aura2SirioEs => "aura-2-sirio-es",
            Self::Aura2NestorEs => "aura-2-nestor-es",
            Self::Aura2CarinaEs => "aura-2-carina-es",
            Self::Aura2CelesteEs => "aura-2-celeste-es",
            Self::Aura2AlvaroEs => "aura-2-alvaro-es",
            Self::Aura2DianaEs => "aura-2-diana-es",
            Self::Aura2AquilaEs => "aura-2-aquila-es",
            Self::Aura2SelenaEs => "aura-2-selena-es",
            Self::Aura2EstrellaEs => "aura-2-estrella-es",
            Self::Aura2JavierEs => "aura-2-javier-es",

            Self::Other(s) => s,
        }
    }
}

impl From<String> for DeepgramSpeakModel {
    fn from(value: String) -> Self {
        match value.as_str() {
            "aura-asteria-en" => Self::AuraAsteriaEn,
            "aura-luna-en" => Self::AuraLunaEn,
            "aura-stella-en" => Self::AuraStellaEn,
            "aura-athena-en" => Self::AuraAthenaEn,
            "aura-hera-en" => Self::AuraHeraEn,
            "aura-orion-en" => Self::AuraOrionEn,
            "aura-arcas-en" => Self::AuraArcasEn,
            "aura-perseus-en" => Self::AuraPerseusEn,
            "aura-angus-en" => Self::AuraAngusEn,
            "aura-orpheus-en" => Self::AuraOrpheusEn,
            "aura-helios-en" => Self::AuraHeliosEn,
            "aura-zeus-en" => Self::AuraZeusEn,

            "aura-2-amalthea-en" => Self::Aura2AmaltheaEn,
            "aura-2-andromeda-en" => Self::Aura2AndromedaEn,
            "aura-2-apollo-en" => Self::Aura2ApolloEn,
            "aura-2-arcas-en" => Self::Aura2ArcasEn,
            "aura-2-aries-en" => Self::Aura2AriesEn,
            "aura-2-asteria-en" => Self::Aura2AsteriaEn,
            "aura-2-athena-en" => Self::Aura2AthenaEn,
            "aura-2-atlas-en" => Self::Aura2AtlasEn,
            "aura-2-aurora-en" => Self::Aura2AuroraEn,
            "aura-2-callista-en" => Self::Aura2CallistaEn,
            "aura-2-cora-en" => Self::Aura2CoraEn,
            "aura-2-cordelia-en" => Self::Aura2CordeliaEn,
            "aura-2-delia-en" => Self::Aura2DeliaEn,
            "aura-2-draco-en" => Self::Aura2DracoEn,
            "aura-2-electra-en" => Self::Aura2ElectraEn,
            "aura-2-harmonia-en" => Self::Aura2HarmoniaEn,
            "aura-2-helena-en" => Self::Aura2HelenaEn,
            "aura-2-hera-en" => Self::Aura2HeraEn,
            "aura-2-hermes-en" => Self::Aura2HermesEn,
            "aura-2-hyperion-en" => Self::Aura2HyperionEn,
            "aura-2-iris-en" => Self::Aura2IrisEn,
            "aura-2-janus-en" => Self::Aura2JanusEn,
            "aura-2-juno-en" => Self::Aura2JunoEn,
            "aura-2-jupiter-en" => Self::Aura2JupiterEn,
            "aura-2-luna-en" => Self::Aura2LunaEn,
            "aura-2-mars-en" => Self::Aura2MarsEn,
            "aura-2-minerva-en" => Self::Aura2MinervaEn,
            "aura-2-neptune-en" => Self::Aura2NeptuneEn,
            "aura-2-odysseus-en" => Self::Aura2OdysseusEn,
            "aura-2-ophelia-en" => Self::Aura2OpheliaEn,
            "aura-2-orion-en" => Self::Aura2OrionEn,
            "aura-2-orpheus-en" => Self::Aura2OrpheusEn,
            "aura-2-pandora-en" => Self::Aura2PandoraEn,
            "aura-2-phoebe-en" => Self::Aura2PhoebeEn,
            "aura-2-pluto-en" => Self::Aura2PlutoEn,
            "aura-2-saturn-en" => Self::Aura2SaturnEn,
            "aura-2-selene-en" => Self::Aura2SeleneEn,
            "aura-2-thalia-en" => Self::Aura2ThaliaEn,
            "aura-2-theia-en" => Self::Aura2TheiaEn,
            "aura-2-vesta-en" => Self::Aura2VestaEn,
            "aura-2-zeus-en" => Self::Aura2ZeusEn,

            "aura-2-sirio-es" => Self::Aura2SirioEs,
            "aura-2-nestor-es" => Self::Aura2NestorEs,
            "aura-2-carina-es" => Self::Aura2CarinaEs,
            "aura-2-celeste-es" => Self::Aura2CelesteEs,
            "aura-2-alvaro-es" => Self::Aura2AlvaroEs,
            "aura-2-diana-es" => Self::Aura2DianaEs,
            "aura-2-aquila-es" => Self::Aura2AquilaEs,
            "aura-2-selena-es" => Self::Aura2SelenaEs,
            "aura-2-estrella-es" => Self::Aura2EstrellaEs,
            "aura-2-javier-es" => Self::Aura2JavierEs,

            _ => Self::Other(value),
        }
    }
}

impl Serialize for DeepgramSpeakModel {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DeepgramSpeakModel {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(Self::from(String::deserialize(de)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn round_trip_aura1() {
        let raw = json!({
            "version": "v1",
            "model": "aura-asteria-en",
            "speed": 1.1
        });
        let p: DeepgramSpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.model, DeepgramSpeakModel::AuraAsteriaEn);
        assert_eq!(p.speed, Some(1.1));
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }

    #[test]
    fn round_trip_aura2_spanish() {
        let raw = json!({ "model": "aura-2-javier-es" });
        let p: DeepgramSpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.model, DeepgramSpeakModel::Aura2JavierEs);
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }

    #[test]
    fn unknown_model_falls_back_to_other() {
        let raw = json!({ "model": "aura-3-future-en" });
        let p: DeepgramSpeakProvider = serde_json::from_value(raw).unwrap();
        assert_eq!(
            p.model,
            DeepgramSpeakModel::Other("aura-3-future-en".into())
        );
    }
}
