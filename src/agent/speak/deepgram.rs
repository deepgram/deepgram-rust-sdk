//! Deepgram Speak provider settings for the Voice Agent.
//!
//! Mirrors `asyncapi/schemas/agent/speak-providers/deepgram.yml`.
//!
//! Two model families are reachable through this one provider, selected
//! by [`DeepgramSpeakProvider::version`]:
//!
//! - **`v1` — Aura** (`aura-*` voices). The default when a provider is
//!   specified without a `version`.
//! - **`v2` — Flux TTS** (`flux-{voice}-{language}` voices). Turn-based,
//!   built for voice agents; supports [`DeepgramSpeakProvider::expressivity`].
//!
//! Switch families by changing `version` and `model` together — a
//! `flux-*` model under `v1`, or an `aura-*` model under `v2`, is invalid.
//! If `agent.speak` is omitted entirely, the server defaults to Flux TTS
//! with `flux-kit-en`.
//!
//! Note: this provider's `model` enum is independent from the SDK's
//! top-level `crate::speak::options::Model` used by the Speak REST API.
//! Phase 8 of the spec-coverage rollout reshapes the top-level model list;
//! this enum will be kept in sync at that time.

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Deepgram TTS as the Voice Agent's Speak provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DeepgramSpeakProvider {
    /// Model family: `v1` (Aura, the server default when omitted) or `v2`
    /// (Flux TTS). Must agree with `model`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<DeepgramSpeakVersion>,

    /// TTS voice model.
    pub model: DeepgramSpeakModel,

    /// Speaking rate multiplier. Aura (`v1`) accepts `0.7`–`1.5`; Flux TTS
    /// (`v2`) accepts `0.5`–`1.5` in `0.05` increments. A value the family
    /// does not accept ends the session with `FAILED_TO_SPEAK`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,

    /// Delivery register on a calm-to-animated axis. **Flux TTS (`v2`)
    /// only.** Fixed for the session. Beta: non-default values increase the
    /// risk of hallucinations and pronunciation errors; audition before
    /// shipping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expressivity: Option<DeepgramSpeakExpressivity>,
}

impl DeepgramSpeakProvider {
    /// Construct with the given model and no other fields set. The server
    /// treats a missing `version` as `v1` (Aura), so pair this with an
    /// `aura-*` model — or use [`Self::v2`] for Flux TTS.
    pub fn new(model: DeepgramSpeakModel) -> Self {
        Self {
            version: None,
            model,
            speed: None,
            expressivity: None,
        }
    }

    /// Construct a Flux TTS (`version: "v2"`) provider. Pair with a
    /// `flux-*` model such as [`DeepgramSpeakModel::FluxAlexisEn`].
    pub fn v2(model: DeepgramSpeakModel) -> Self {
        Self::new(model).with_version(DeepgramSpeakVersion::V2)
    }

    /// Set the model family explicitly.
    pub fn with_version(mut self, version: DeepgramSpeakVersion) -> Self {
        self.version = Some(version);
        self
    }

    /// Set the speaking rate. See [`Self::speed`] for the per-family range.
    pub fn with_speed(mut self, speed: f64) -> Self {
        self.speed = Some(speed);
        self
    }

    /// Set the expressivity (Flux TTS / `v2` only).
    pub fn with_expressivity(mut self, expressivity: DeepgramSpeakExpressivity) -> Self {
        self.expressivity = Some(expressivity);
        self
    }
}

/// Deepgram TTS model family used by the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DeepgramSpeakVersion {
    /// Aura (`aura-*` voices). The server default when omitted.
    #[serde(rename = "v1")]
    V1,
    /// Flux TTS (`flux-{voice}-{language}` voices).
    #[serde(rename = "v2")]
    V2,
}

/// Expressive range of Flux TTS speech, on a calm-to-animated axis.
///
/// Serializes as the integer `-2`…`2`. [`DeepgramSpeakExpressivity::Zero`]
/// (the default) is the voice's tuned delivery and the only value validated
/// for production; negative values are calmer, positive values more
/// animated. Beta — behavior may change in future model versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum DeepgramSpeakExpressivity {
    /// `-2` — the calm end of the range.
    NegativeTwo,
    /// `-1` — calmer and more measured.
    NegativeOne,
    /// `0` — the voice's tuned delivery (default).
    #[default]
    Zero,
    /// `1` — more animated.
    One,
    /// `2` — the animated end of the range.
    Two,
}

impl DeepgramSpeakExpressivity {
    /// Wire integer value.
    pub fn as_i8(self) -> i8 {
        match self {
            Self::NegativeTwo => -2,
            Self::NegativeOne => -1,
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
        }
    }

    /// Construct from the wire integer value; `None` if out of range.
    pub fn from_i8(value: i8) -> Option<Self> {
        Some(match value {
            -2 => Self::NegativeTwo,
            -1 => Self::NegativeOne,
            0 => Self::Zero,
            1 => Self::One,
            2 => Self::Two,
            _ => return None,
        })
    }
}

impl Serialize for DeepgramSpeakExpressivity {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_i8(self.as_i8())
    }
}

impl<'de> Deserialize<'de> for DeepgramSpeakExpressivity {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let value = i64::deserialize(de)?;
        i8::try_from(value)
            .ok()
            .and_then(Self::from_i8)
            .ok_or_else(|| {
                D::Error::custom(format!(
                    "expressivity must be a whole number from -2 to 2, got {value}"
                ))
            })
    }
}

/// Deepgram TTS voice model.
///
/// Includes the Aura-1 English voices, the Aura-2 English/Spanish voices,
/// and the Flux TTS English voices listed in the AsyncAPI spec at the time
/// this SDK was built. Aura voices require [`DeepgramSpeakVersion::V1`]
/// (or no `version`); Flux TTS voices require [`DeepgramSpeakVersion::V2`].
/// Use [`DeepgramSpeakModel::Other`] to pass any value not yet enumerated.
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

    // Flux TTS English voices (require `version: "v2"`).
    FluxAlexisEn,
    FluxBreeEn,
    FluxBrittanyEn,
    FluxBrookeEn,
    FluxBruceEn,
    FluxCliffEn,
    FluxColeEn,
    FluxColinEn,
    FluxConorEn,
    FluxDonovanEn,
    FluxDrewEn,
    FluxEliseEn,
    FluxGemmaEn,
    FluxHaleyEn,
    FluxHannahEn,
    FluxHeatherEn,
    FluxJackEn,
    FluxKaiEn,
    FluxKelseyEn,
    /// The server default voice when `agent.speak` is omitted.
    FluxKitEn,
    FluxMaeveEn,
    FluxMarceloEn,
    FluxMarcusEn,
    FluxMeenaEn,
    FluxMeghanEn,
    FluxMilesEn,
    FluxNaveenEn,
    FluxPaigeEn,
    FluxPriyaEn,
    FluxRufusEn,
    FluxSeanEn,
    FluxSharonEn,
    FluxSiennaEn,
    FluxTannerEn,
    FluxWadeEn,
    FluxWesEn,

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

            Self::FluxAlexisEn => "flux-alexis-en",
            Self::FluxBreeEn => "flux-bree-en",
            Self::FluxBrittanyEn => "flux-brittany-en",
            Self::FluxBrookeEn => "flux-brooke-en",
            Self::FluxBruceEn => "flux-bruce-en",
            Self::FluxCliffEn => "flux-cliff-en",
            Self::FluxColeEn => "flux-cole-en",
            Self::FluxColinEn => "flux-colin-en",
            Self::FluxConorEn => "flux-conor-en",
            Self::FluxDonovanEn => "flux-donovan-en",
            Self::FluxDrewEn => "flux-drew-en",
            Self::FluxEliseEn => "flux-elise-en",
            Self::FluxGemmaEn => "flux-gemma-en",
            Self::FluxHaleyEn => "flux-haley-en",
            Self::FluxHannahEn => "flux-hannah-en",
            Self::FluxHeatherEn => "flux-heather-en",
            Self::FluxJackEn => "flux-jack-en",
            Self::FluxKaiEn => "flux-kai-en",
            Self::FluxKelseyEn => "flux-kelsey-en",
            Self::FluxKitEn => "flux-kit-en",
            Self::FluxMaeveEn => "flux-maeve-en",
            Self::FluxMarceloEn => "flux-marcelo-en",
            Self::FluxMarcusEn => "flux-marcus-en",
            Self::FluxMeenaEn => "flux-meena-en",
            Self::FluxMeghanEn => "flux-meghan-en",
            Self::FluxMilesEn => "flux-miles-en",
            Self::FluxNaveenEn => "flux-naveen-en",
            Self::FluxPaigeEn => "flux-paige-en",
            Self::FluxPriyaEn => "flux-priya-en",
            Self::FluxRufusEn => "flux-rufus-en",
            Self::FluxSeanEn => "flux-sean-en",
            Self::FluxSharonEn => "flux-sharon-en",
            Self::FluxSiennaEn => "flux-sienna-en",
            Self::FluxTannerEn => "flux-tanner-en",
            Self::FluxWadeEn => "flux-wade-en",
            Self::FluxWesEn => "flux-wes-en",

            Self::Other(s) => s,
        }
    }

    /// `true` for the Flux TTS (`flux-*`) voices, which require
    /// [`DeepgramSpeakVersion::V2`]. Also `true` for an
    /// [`Other`](Self::Other) value starting with `flux-`.
    pub fn is_flux(&self) -> bool {
        self.as_str().starts_with("flux-")
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

            "flux-alexis-en" => Self::FluxAlexisEn,
            "flux-bree-en" => Self::FluxBreeEn,
            "flux-brittany-en" => Self::FluxBrittanyEn,
            "flux-brooke-en" => Self::FluxBrookeEn,
            "flux-bruce-en" => Self::FluxBruceEn,
            "flux-cliff-en" => Self::FluxCliffEn,
            "flux-cole-en" => Self::FluxColeEn,
            "flux-colin-en" => Self::FluxColinEn,
            "flux-conor-en" => Self::FluxConorEn,
            "flux-donovan-en" => Self::FluxDonovanEn,
            "flux-drew-en" => Self::FluxDrewEn,
            "flux-elise-en" => Self::FluxEliseEn,
            "flux-gemma-en" => Self::FluxGemmaEn,
            "flux-haley-en" => Self::FluxHaleyEn,
            "flux-hannah-en" => Self::FluxHannahEn,
            "flux-heather-en" => Self::FluxHeatherEn,
            "flux-jack-en" => Self::FluxJackEn,
            "flux-kai-en" => Self::FluxKaiEn,
            "flux-kelsey-en" => Self::FluxKelseyEn,
            "flux-kit-en" => Self::FluxKitEn,
            "flux-maeve-en" => Self::FluxMaeveEn,
            "flux-marcelo-en" => Self::FluxMarceloEn,
            "flux-marcus-en" => Self::FluxMarcusEn,
            "flux-meena-en" => Self::FluxMeenaEn,
            "flux-meghan-en" => Self::FluxMeghanEn,
            "flux-miles-en" => Self::FluxMilesEn,
            "flux-naveen-en" => Self::FluxNaveenEn,
            "flux-paige-en" => Self::FluxPaigeEn,
            "flux-priya-en" => Self::FluxPriyaEn,
            "flux-rufus-en" => Self::FluxRufusEn,
            "flux-sean-en" => Self::FluxSeanEn,
            "flux-sharon-en" => Self::FluxSharonEn,
            "flux-sienna-en" => Self::FluxSiennaEn,
            "flux-tanner-en" => Self::FluxTannerEn,
            "flux-wade-en" => Self::FluxWadeEn,
            "flux-wes-en" => Self::FluxWesEn,

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
    use crate::agent::speak::{SpeakProvider, SpeakSettings};
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
        assert!(p.expressivity.is_none());
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
        assert!(!p.model.is_flux());
        assert!(DeepgramSpeakModel::Other("flux-future-en".into()).is_flux());
    }

    #[test]
    fn v2_flux_fixture_exact_wire_json() {
        // Voice Agent V2 / Flux TTS speak settings, as they appear under
        // `agent.speak` in `Settings` (and in `UpdateSpeak`).
        let settings = SpeakSettings::new(SpeakProvider::Deepgram(
            DeepgramSpeakProvider::v2(DeepgramSpeakModel::FluxAlexisEn)
                .with_speed(1.05)
                .with_expressivity(DeepgramSpeakExpressivity::NegativeOne),
        ));
        assert_eq!(
            serde_json::to_string(&settings).unwrap(),
            r#"{"provider":{"type":"deepgram","version":"v2","model":"flux-alexis-en","speed":1.05,"expressivity":-1}}"#
        );

        // Minimal V2: version + model only.
        let minimal = SpeakSettings::new(SpeakProvider::Deepgram(DeepgramSpeakProvider::v2(
            DeepgramSpeakModel::FluxKitEn,
        )));
        assert_eq!(
            serde_json::to_string(&minimal).unwrap(),
            r#"{"provider":{"type":"deepgram","version":"v2","model":"flux-kit-en"}}"#
        );
    }

    #[test]
    fn v2_flux_fixture_round_trip() {
        let raw = json!({
            "provider": {
                "type": "deepgram",
                "version": "v2",
                "model": "flux-haley-en",
                "speed": 0.95,
                "expressivity": 2
            }
        });
        let settings: SpeakSettings = serde_json::from_value(raw.clone()).unwrap();
        match &settings.provider {
            SpeakProvider::Deepgram(p) => {
                assert_eq!(p.version, Some(DeepgramSpeakVersion::V2));
                assert_eq!(p.model, DeepgramSpeakModel::FluxHaleyEn);
                assert!(p.model.is_flux());
                assert_eq!(p.speed, Some(0.95));
                assert_eq!(p.expressivity, Some(DeepgramSpeakExpressivity::Two));
            }
            other => panic!("expected Deepgram provider, got {other:?}"),
        }
        assert_eq!(serde_json::to_value(&settings).unwrap(), raw);
    }

    #[test]
    fn expressivity_wire_values() {
        for (variant, wire) in [
            (DeepgramSpeakExpressivity::NegativeTwo, -2),
            (DeepgramSpeakExpressivity::NegativeOne, -1),
            (DeepgramSpeakExpressivity::Zero, 0),
            (DeepgramSpeakExpressivity::One, 1),
            (DeepgramSpeakExpressivity::Two, 2),
        ] {
            assert_eq!(serde_json::to_value(variant).unwrap(), json!(wire));
            let back: DeepgramSpeakExpressivity = serde_json::from_value(json!(wire)).unwrap();
            assert_eq!(back, variant);
            assert_eq!(variant.as_i8(), wire);
        }
        assert_eq!(
            DeepgramSpeakExpressivity::default(),
            DeepgramSpeakExpressivity::Zero
        );
    }

    #[test]
    fn expressivity_rejects_out_of_range_and_fractional() {
        for bad in [json!(3), json!(-3), json!(1.5), json!("1")] {
            let err = serde_json::from_value::<DeepgramSpeakExpressivity>(bad.clone());
            assert!(err.is_err(), "expected {bad} to be rejected");
        }
    }

    #[test]
    fn every_flux_voice_round_trips_and_is_flux() {
        for name in [
            "flux-alexis-en",
            "flux-bree-en",
            "flux-brittany-en",
            "flux-brooke-en",
            "flux-bruce-en",
            "flux-cliff-en",
            "flux-cole-en",
            "flux-colin-en",
            "flux-conor-en",
            "flux-donovan-en",
            "flux-drew-en",
            "flux-elise-en",
            "flux-gemma-en",
            "flux-haley-en",
            "flux-hannah-en",
            "flux-heather-en",
            "flux-jack-en",
            "flux-kai-en",
            "flux-kelsey-en",
            "flux-kit-en",
            "flux-maeve-en",
            "flux-marcelo-en",
            "flux-marcus-en",
            "flux-meena-en",
            "flux-meghan-en",
            "flux-miles-en",
            "flux-naveen-en",
            "flux-paige-en",
            "flux-priya-en",
            "flux-rufus-en",
            "flux-sean-en",
            "flux-sharon-en",
            "flux-sienna-en",
            "flux-tanner-en",
            "flux-wade-en",
            "flux-wes-en",
        ] {
            let model = DeepgramSpeakModel::from(name.to_string());
            assert!(
                !matches!(model, DeepgramSpeakModel::Other(_)),
                "{name} should be a named variant"
            );
            assert!(model.is_flux());
            assert_eq!(model.as_str(), name);
        }
    }

    #[test]
    fn version_wire_values() {
        assert_eq!(
            serde_json::to_value(DeepgramSpeakVersion::V1).unwrap(),
            json!("v1")
        );
        assert_eq!(
            serde_json::to_value(DeepgramSpeakVersion::V2).unwrap(),
            json!("v2")
        );
        let v: DeepgramSpeakVersion = serde_json::from_value(json!("v2")).unwrap();
        assert_eq!(v, DeepgramSpeakVersion::V2);
    }
}
