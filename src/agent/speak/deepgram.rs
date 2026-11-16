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

use core::fmt;

use serde::de::{self, Visitor};
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
/// Serializes as the JSON integer `-2`…`2`, which is the form the live
/// Voice Agent service accepts under `agent.speak.provider.expressivity`;
/// the string form the published AsyncAPI declares
/// (`DeepgramSpeakProviderExpressivity`, `type: string`, `enum: ['-2'…'2']`)
/// is rejected by the service with `UNPARSABLE_CLIENT_MESSAGE`. Deserialization
/// accepts both forms, so a `Settings` written against either reading of the
/// contract parses.
///
/// [`DeepgramSpeakExpressivity::Zero`] (the default) is the voice's tuned
/// delivery and the only value validated for production; negative values are
/// calmer, positive values more animated. Flux TTS (`v2`) only, and fixed for
/// the session. Beta — behavior may change in future model versions.
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
    /// Always the JSON integer: the string form fails the session.
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_i8(self.as_i8())
    }
}

impl<'de> Deserialize<'de> for DeepgramSpeakExpressivity {
    /// Accepts the JSON integer the service uses and the JSON string the
    /// AsyncAPI declares, so neither reading of the contract fails to parse.
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct Wire;

        impl Visitor<'_> for Wire {
            type Value = DeepgramSpeakExpressivity;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a whole number from -2 to 2, or a string holding one")
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                i8::try_from(value)
                    .ok()
                    .and_then(DeepgramSpeakExpressivity::from_i8)
                    .ok_or_else(|| out_of_range::<E>(&value.to_string()))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                i8::try_from(value)
                    .ok()
                    .and_then(DeepgramSpeakExpressivity::from_i8)
                    .ok_or_else(|| out_of_range::<E>(&value.to_string()))
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                value
                    .trim()
                    .parse::<i8>()
                    .ok()
                    .and_then(DeepgramSpeakExpressivity::from_i8)
                    .ok_or_else(|| out_of_range::<E>(value))
            }
        }

        fn out_of_range<E: de::Error>(value: &str) -> E {
            E::custom(format!(
                "expressivity must be a whole number from -2 to 2, got {value}"
            ))
        }

        de.deserialize_any(Wire)
    }
}

/// Deepgram TTS voice model.
///
/// Includes the Aura-1 English voices, Aura-2 English/Spanish voices, and
/// every Flux TTS voice listed in the AsyncAPI spec at the time this SDK was
/// built. Aura voices require [`DeepgramSpeakVersion::V1`] (or no `version`);
/// Flux TTS voices require [`DeepgramSpeakVersion::V2`]. Use
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

    // Flux TTS voices (require `version: "v2"`).
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

    FluxAnoukNl,
    FluxBramNl,
    FluxFleurNl,
    FluxIreneNl,
    FluxLuukNl,
    FluxMoniqueNl,
    FluxRubenNl,
    FluxSanneNl,
    FluxYvonneNl,
    FluxDirkNl,
    FluxLotteNl,
    FluxNielsNl,
    FluxVeerleNl,
    FluxArnaudFr,
    FluxBenoitFr,
    FluxBrigitteFr,
    FluxCamilleFr,
    FluxChloeFr,
    FluxFlorianFr,
    FluxJulienFr,
    FluxMathieuFr,
    FluxPierreFr,
    FluxThierryFr,
    FluxVeroniqueFr,
    FluxYasmineFr,
    FluxBenjaminDe,
    FluxChristianDe,
    FluxJanineDe,
    FluxMagdalenaDe,
    FluxMoritzDe,
    FluxNadineDe,
    FluxRolfDe,
    FluxSonjaDe,
    FluxSvenDe,
    FluxTobiasDe,
    FluxVerenaDe,
    FluxLeonieDe,
    FluxYannickDe,
    FluxAjayHi,
    FluxAnjaliHi,
    FluxArjunHi,
    FluxDivyaHi,
    FluxJyotiHi,
    FluxNikhilHi,
    FluxPallaviHi,
    FluxPrachiHi,
    FluxRahulHi,
    FluxRajeshHi,
    FluxRituHi,
    FluxShreyaHi,
    FluxSunitaHi,
    FluxVikramHi,
    FluxAngelaIt,
    FluxChiaraIt,
    FluxFedericoIt,
    FluxGiorgioIt,
    FluxMatteoIt,
    FluxPaolaIt,
    FluxRoccoIt,
    FluxSofiaIt,
    FluxStefanoIt,
    FluxVincenzoIt,
    FluxVitoIt,
    FluxAyumiJa,
    FluxHayatoJa,
    FluxNozomiJa,
    FluxSayakaJa,
    FluxBiancaPt,
    FluxBrunoPt,
    FluxClaudiaPt,
    FluxLeandroPt,
    FluxMarciaPt,
    FluxMarianaPt,
    FluxMartaPt,
    FluxMateusPt,
    FluxMiguelPt,
    FluxRenatoPt,
    FluxRobertoPt,
    FluxRodrigoPt,
    FluxArturPt,
    FluxBeneditaPt,
    FluxCatarinaPt,
    FluxHugoPt,
    FluxPedroPt,
    FluxRicardoPt,
    FluxAndreiRu,
    FluxKirillRu,
    FluxMaximRu,
    FluxMikhailRu,
    FluxNataliaRu,
    FluxNikolaiRu,
    FluxOlgaRu,
    FluxPolinaRu,
    FluxVladimirRu,
    FluxAinaEs,
    FluxAlfredoEs,
    FluxCarmenEs,
    FluxCeciliaEs,
    FluxDuvanEs,
    FluxEduardoEs,
    FluxErnestoEs,
    FluxFernandaEs,
    FluxFiorellaEs,
    FluxGustavoEs,
    FluxJaimeEs,
    FluxMartinEs,
    FluxMauricioEs,
    FluxPatriciaEs,
    FluxPilarEs,
    FluxRafaelEs,
    FluxTomasEs,
    FluxValentinaEs,
    FluxYsabelEs,
    FluxAfsanaBn,
    FluxIshratBn,
    FluxMamunBn,
    FluxSabbirBn,
    FluxShuvoBn,
    FluxSumaiyaBn,
    FluxDesislavaBg,
    FluxGerganaBg,
    FluxJordiCa,
    FluxJuditCa,
    FluxBoZh,
    FluxHaoranZh,
    FluxJianZh,
    FluxJieZh,
    FluxLijuanZh,
    FluxShufenZh,
    FluxWenboZh,
    FluxWenjingZh,
    FluxZhenZh,
    FluxChiayingZh,
    FluxKuanyuZh,
    FluxYawenZh,
    FluxYutingZh,
    FluxManlokZh,
    FluxDaliborHr,
    FluxDanijelaHr,
    FluxDomagojHr,
    FluxJasnaHr,
    FluxJosipaHr,
    FluxMihaelaHr,
    FluxVedranHr,
    FluxJakubCs,
    FluxPernilleDa,
    FluxRasmusDa,
    FluxRikkeDa,
    FluxTriinEt,
    FluxKirsiFi,
    FluxTeemuFi,
    FluxKostasEl,
    FluxBhavnaGu,
    FluxHetalGu,
    FluxJigneshGu,
    FluxBenceHu,
    FluxEszterHu,
    FluxKingaHu,
    FluxTiborHu,
    FluxDewiId,
    FluxFajarId,
    FluxFitriId,
    FluxRizkyId,
    FluxTeguhId,
    FluxManjunathKn,
    FluxEunjiKo,
    FluxHyunwooKo,
    FluxMikyungKo,
    FluxMinjunKo,
    FluxSoojinKo,
    FluxSungminKo,
    FluxAgneseLv,
    FluxEdgarsLv,
    FluxGodaLt,
    FluxNeringaLt,
    FluxHaziqMs,
    FluxIzzahMs,
    FluxPoojaMr,
    FluxPratikMr,
    FluxSnehaMr,
    FluxErlendNo,
    FluxEspenNo,
    FluxIdaNo,
    FluxSigneNo,
    FluxSiljeNo,
    FluxVegardNo,
    FluxBartoszPl,
    FluxEwaPl,
    FluxJustynaPl,
    FluxKacperPl,
    FluxKonradPl,
    FluxZuzannaPl,
    FluxCosminRo,
    FluxIoanaRo,
    FluxDominikaSk,
    FluxJurajSk,
    FluxGregorSl,
    FluxKatjaSl,
    FluxAxelSv,
    FluxCharlottaSv,
    FluxFridaSv,
    FluxLudvigSv,
    FluxOskarSv,
    FluxPontusSv,
    FluxWilmaSv,
    FluxJeromeTl,
    FluxJomarTl,
    FluxLiezelTl,
    FluxNoelTl,
    FluxTrishaTl,
    FluxGokulTa,
    FluxKavyaTa,
    FluxSowmyaTa,
    FluxVinothTa,
    FluxPavanTe,
    FluxRaviTe,
    FluxSravaniTe,
    FluxDilekTr,
    FluxEcemTr,
    FluxEmreTr,
    FluxGizemTr,
    FluxMehmetTr,
    FluxSerkanTr,
    FluxZeynepTr,
    FluxDenysUk,
    FluxSolomiyaUk,
    FluxAmbreenUr,
    FluxUsmanUr,
    FluxHoaVi,
    FluxMinhVi,

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

            Self::FluxAnoukNl => "flux-anouk-nl",
            Self::FluxBramNl => "flux-bram-nl",
            Self::FluxFleurNl => "flux-fleur-nl",
            Self::FluxIreneNl => "flux-irene-nl",
            Self::FluxLuukNl => "flux-luuk-nl",
            Self::FluxMoniqueNl => "flux-monique-nl",
            Self::FluxRubenNl => "flux-ruben-nl",
            Self::FluxSanneNl => "flux-sanne-nl",
            Self::FluxYvonneNl => "flux-yvonne-nl",
            Self::FluxDirkNl => "flux-dirk-nl",
            Self::FluxLotteNl => "flux-lotte-nl",
            Self::FluxNielsNl => "flux-niels-nl",
            Self::FluxVeerleNl => "flux-veerle-nl",
            Self::FluxArnaudFr => "flux-arnaud-fr",
            Self::FluxBenoitFr => "flux-benoit-fr",
            Self::FluxBrigitteFr => "flux-brigitte-fr",
            Self::FluxCamilleFr => "flux-camille-fr",
            Self::FluxChloeFr => "flux-chloe-fr",
            Self::FluxFlorianFr => "flux-florian-fr",
            Self::FluxJulienFr => "flux-julien-fr",
            Self::FluxMathieuFr => "flux-mathieu-fr",
            Self::FluxPierreFr => "flux-pierre-fr",
            Self::FluxThierryFr => "flux-thierry-fr",
            Self::FluxVeroniqueFr => "flux-veronique-fr",
            Self::FluxYasmineFr => "flux-yasmine-fr",
            Self::FluxBenjaminDe => "flux-benjamin-de",
            Self::FluxChristianDe => "flux-christian-de",
            Self::FluxJanineDe => "flux-janine-de",
            Self::FluxMagdalenaDe => "flux-magdalena-de",
            Self::FluxMoritzDe => "flux-moritz-de",
            Self::FluxNadineDe => "flux-nadine-de",
            Self::FluxRolfDe => "flux-rolf-de",
            Self::FluxSonjaDe => "flux-sonja-de",
            Self::FluxSvenDe => "flux-sven-de",
            Self::FluxTobiasDe => "flux-tobias-de",
            Self::FluxVerenaDe => "flux-verena-de",
            Self::FluxLeonieDe => "flux-leonie-de",
            Self::FluxYannickDe => "flux-yannick-de",
            Self::FluxAjayHi => "flux-ajay-hi",
            Self::FluxAnjaliHi => "flux-anjali-hi",
            Self::FluxArjunHi => "flux-arjun-hi",
            Self::FluxDivyaHi => "flux-divya-hi",
            Self::FluxJyotiHi => "flux-jyoti-hi",
            Self::FluxNikhilHi => "flux-nikhil-hi",
            Self::FluxPallaviHi => "flux-pallavi-hi",
            Self::FluxPrachiHi => "flux-prachi-hi",
            Self::FluxRahulHi => "flux-rahul-hi",
            Self::FluxRajeshHi => "flux-rajesh-hi",
            Self::FluxRituHi => "flux-ritu-hi",
            Self::FluxShreyaHi => "flux-shreya-hi",
            Self::FluxSunitaHi => "flux-sunita-hi",
            Self::FluxVikramHi => "flux-vikram-hi",
            Self::FluxAngelaIt => "flux-angela-it",
            Self::FluxChiaraIt => "flux-chiara-it",
            Self::FluxFedericoIt => "flux-federico-it",
            Self::FluxGiorgioIt => "flux-giorgio-it",
            Self::FluxMatteoIt => "flux-matteo-it",
            Self::FluxPaolaIt => "flux-paola-it",
            Self::FluxRoccoIt => "flux-rocco-it",
            Self::FluxSofiaIt => "flux-sofia-it",
            Self::FluxStefanoIt => "flux-stefano-it",
            Self::FluxVincenzoIt => "flux-vincenzo-it",
            Self::FluxVitoIt => "flux-vito-it",
            Self::FluxAyumiJa => "flux-ayumi-ja",
            Self::FluxHayatoJa => "flux-hayato-ja",
            Self::FluxNozomiJa => "flux-nozomi-ja",
            Self::FluxSayakaJa => "flux-sayaka-ja",
            Self::FluxBiancaPt => "flux-bianca-pt",
            Self::FluxBrunoPt => "flux-bruno-pt",
            Self::FluxClaudiaPt => "flux-claudia-pt",
            Self::FluxLeandroPt => "flux-leandro-pt",
            Self::FluxMarciaPt => "flux-marcia-pt",
            Self::FluxMarianaPt => "flux-mariana-pt",
            Self::FluxMartaPt => "flux-marta-pt",
            Self::FluxMateusPt => "flux-mateus-pt",
            Self::FluxMiguelPt => "flux-miguel-pt",
            Self::FluxRenatoPt => "flux-renato-pt",
            Self::FluxRobertoPt => "flux-roberto-pt",
            Self::FluxRodrigoPt => "flux-rodrigo-pt",
            Self::FluxArturPt => "flux-artur-pt",
            Self::FluxBeneditaPt => "flux-benedita-pt",
            Self::FluxCatarinaPt => "flux-catarina-pt",
            Self::FluxHugoPt => "flux-hugo-pt",
            Self::FluxPedroPt => "flux-pedro-pt",
            Self::FluxRicardoPt => "flux-ricardo-pt",
            Self::FluxAndreiRu => "flux-andrei-ru",
            Self::FluxKirillRu => "flux-kirill-ru",
            Self::FluxMaximRu => "flux-maxim-ru",
            Self::FluxMikhailRu => "flux-mikhail-ru",
            Self::FluxNataliaRu => "flux-natalia-ru",
            Self::FluxNikolaiRu => "flux-nikolai-ru",
            Self::FluxOlgaRu => "flux-olga-ru",
            Self::FluxPolinaRu => "flux-polina-ru",
            Self::FluxVladimirRu => "flux-vladimir-ru",
            Self::FluxAinaEs => "flux-aina-es",
            Self::FluxAlfredoEs => "flux-alfredo-es",
            Self::FluxCarmenEs => "flux-carmen-es",
            Self::FluxCeciliaEs => "flux-cecilia-es",
            Self::FluxDuvanEs => "flux-duvan-es",
            Self::FluxEduardoEs => "flux-eduardo-es",
            Self::FluxErnestoEs => "flux-ernesto-es",
            Self::FluxFernandaEs => "flux-fernanda-es",
            Self::FluxFiorellaEs => "flux-fiorella-es",
            Self::FluxGustavoEs => "flux-gustavo-es",
            Self::FluxJaimeEs => "flux-jaime-es",
            Self::FluxMartinEs => "flux-martin-es",
            Self::FluxMauricioEs => "flux-mauricio-es",
            Self::FluxPatriciaEs => "flux-patricia-es",
            Self::FluxPilarEs => "flux-pilar-es",
            Self::FluxRafaelEs => "flux-rafael-es",
            Self::FluxTomasEs => "flux-tomas-es",
            Self::FluxValentinaEs => "flux-valentina-es",
            Self::FluxYsabelEs => "flux-ysabel-es",
            Self::FluxAfsanaBn => "flux-afsana-bn",
            Self::FluxIshratBn => "flux-ishrat-bn",
            Self::FluxMamunBn => "flux-mamun-bn",
            Self::FluxSabbirBn => "flux-sabbir-bn",
            Self::FluxShuvoBn => "flux-shuvo-bn",
            Self::FluxSumaiyaBn => "flux-sumaiya-bn",
            Self::FluxDesislavaBg => "flux-desislava-bg",
            Self::FluxGerganaBg => "flux-gergana-bg",
            Self::FluxJordiCa => "flux-jordi-ca",
            Self::FluxJuditCa => "flux-judit-ca",
            Self::FluxBoZh => "flux-bo-zh",
            Self::FluxHaoranZh => "flux-haoran-zh",
            Self::FluxJianZh => "flux-jian-zh",
            Self::FluxJieZh => "flux-jie-zh",
            Self::FluxLijuanZh => "flux-lijuan-zh",
            Self::FluxShufenZh => "flux-shufen-zh",
            Self::FluxWenboZh => "flux-wenbo-zh",
            Self::FluxWenjingZh => "flux-wenjing-zh",
            Self::FluxZhenZh => "flux-zhen-zh",
            Self::FluxChiayingZh => "flux-chiaying-zh",
            Self::FluxKuanyuZh => "flux-kuanyu-zh",
            Self::FluxYawenZh => "flux-yawen-zh",
            Self::FluxYutingZh => "flux-yuting-zh",
            Self::FluxManlokZh => "flux-manlok-zh",
            Self::FluxDaliborHr => "flux-dalibor-hr",
            Self::FluxDanijelaHr => "flux-danijela-hr",
            Self::FluxDomagojHr => "flux-domagoj-hr",
            Self::FluxJasnaHr => "flux-jasna-hr",
            Self::FluxJosipaHr => "flux-josipa-hr",
            Self::FluxMihaelaHr => "flux-mihaela-hr",
            Self::FluxVedranHr => "flux-vedran-hr",
            Self::FluxJakubCs => "flux-jakub-cs",
            Self::FluxPernilleDa => "flux-pernille-da",
            Self::FluxRasmusDa => "flux-rasmus-da",
            Self::FluxRikkeDa => "flux-rikke-da",
            Self::FluxTriinEt => "flux-triin-et",
            Self::FluxKirsiFi => "flux-kirsi-fi",
            Self::FluxTeemuFi => "flux-teemu-fi",
            Self::FluxKostasEl => "flux-kostas-el",
            Self::FluxBhavnaGu => "flux-bhavna-gu",
            Self::FluxHetalGu => "flux-hetal-gu",
            Self::FluxJigneshGu => "flux-jignesh-gu",
            Self::FluxBenceHu => "flux-bence-hu",
            Self::FluxEszterHu => "flux-eszter-hu",
            Self::FluxKingaHu => "flux-kinga-hu",
            Self::FluxTiborHu => "flux-tibor-hu",
            Self::FluxDewiId => "flux-dewi-id",
            Self::FluxFajarId => "flux-fajar-id",
            Self::FluxFitriId => "flux-fitri-id",
            Self::FluxRizkyId => "flux-rizky-id",
            Self::FluxTeguhId => "flux-teguh-id",
            Self::FluxManjunathKn => "flux-manjunath-kn",
            Self::FluxEunjiKo => "flux-eunji-ko",
            Self::FluxHyunwooKo => "flux-hyunwoo-ko",
            Self::FluxMikyungKo => "flux-mikyung-ko",
            Self::FluxMinjunKo => "flux-minjun-ko",
            Self::FluxSoojinKo => "flux-soojin-ko",
            Self::FluxSungminKo => "flux-sungmin-ko",
            Self::FluxAgneseLv => "flux-agnese-lv",
            Self::FluxEdgarsLv => "flux-edgars-lv",
            Self::FluxGodaLt => "flux-goda-lt",
            Self::FluxNeringaLt => "flux-neringa-lt",
            Self::FluxHaziqMs => "flux-haziq-ms",
            Self::FluxIzzahMs => "flux-izzah-ms",
            Self::FluxPoojaMr => "flux-pooja-mr",
            Self::FluxPratikMr => "flux-pratik-mr",
            Self::FluxSnehaMr => "flux-sneha-mr",
            Self::FluxErlendNo => "flux-erlend-no",
            Self::FluxEspenNo => "flux-espen-no",
            Self::FluxIdaNo => "flux-ida-no",
            Self::FluxSigneNo => "flux-signe-no",
            Self::FluxSiljeNo => "flux-silje-no",
            Self::FluxVegardNo => "flux-vegard-no",
            Self::FluxBartoszPl => "flux-bartosz-pl",
            Self::FluxEwaPl => "flux-ewa-pl",
            Self::FluxJustynaPl => "flux-justyna-pl",
            Self::FluxKacperPl => "flux-kacper-pl",
            Self::FluxKonradPl => "flux-konrad-pl",
            Self::FluxZuzannaPl => "flux-zuzanna-pl",
            Self::FluxCosminRo => "flux-cosmin-ro",
            Self::FluxIoanaRo => "flux-ioana-ro",
            Self::FluxDominikaSk => "flux-dominika-sk",
            Self::FluxJurajSk => "flux-juraj-sk",
            Self::FluxGregorSl => "flux-gregor-sl",
            Self::FluxKatjaSl => "flux-katja-sl",
            Self::FluxAxelSv => "flux-axel-sv",
            Self::FluxCharlottaSv => "flux-charlotta-sv",
            Self::FluxFridaSv => "flux-frida-sv",
            Self::FluxLudvigSv => "flux-ludvig-sv",
            Self::FluxOskarSv => "flux-oskar-sv",
            Self::FluxPontusSv => "flux-pontus-sv",
            Self::FluxWilmaSv => "flux-wilma-sv",
            Self::FluxJeromeTl => "flux-jerome-tl",
            Self::FluxJomarTl => "flux-jomar-tl",
            Self::FluxLiezelTl => "flux-liezel-tl",
            Self::FluxNoelTl => "flux-noel-tl",
            Self::FluxTrishaTl => "flux-trisha-tl",
            Self::FluxGokulTa => "flux-gokul-ta",
            Self::FluxKavyaTa => "flux-kavya-ta",
            Self::FluxSowmyaTa => "flux-sowmya-ta",
            Self::FluxVinothTa => "flux-vinoth-ta",
            Self::FluxPavanTe => "flux-pavan-te",
            Self::FluxRaviTe => "flux-ravi-te",
            Self::FluxSravaniTe => "flux-sravani-te",
            Self::FluxDilekTr => "flux-dilek-tr",
            Self::FluxEcemTr => "flux-ecem-tr",
            Self::FluxEmreTr => "flux-emre-tr",
            Self::FluxGizemTr => "flux-gizem-tr",
            Self::FluxMehmetTr => "flux-mehmet-tr",
            Self::FluxSerkanTr => "flux-serkan-tr",
            Self::FluxZeynepTr => "flux-zeynep-tr",
            Self::FluxDenysUk => "flux-denys-uk",
            Self::FluxSolomiyaUk => "flux-solomiya-uk",
            Self::FluxAmbreenUr => "flux-ambreen-ur",
            Self::FluxUsmanUr => "flux-usman-ur",
            Self::FluxHoaVi => "flux-hoa-vi",
            Self::FluxMinhVi => "flux-minh-vi",

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

            "flux-anouk-nl" => Self::FluxAnoukNl,
            "flux-bram-nl" => Self::FluxBramNl,
            "flux-fleur-nl" => Self::FluxFleurNl,
            "flux-irene-nl" => Self::FluxIreneNl,
            "flux-luuk-nl" => Self::FluxLuukNl,
            "flux-monique-nl" => Self::FluxMoniqueNl,
            "flux-ruben-nl" => Self::FluxRubenNl,
            "flux-sanne-nl" => Self::FluxSanneNl,
            "flux-yvonne-nl" => Self::FluxYvonneNl,
            "flux-dirk-nl" => Self::FluxDirkNl,
            "flux-lotte-nl" => Self::FluxLotteNl,
            "flux-niels-nl" => Self::FluxNielsNl,
            "flux-veerle-nl" => Self::FluxVeerleNl,
            "flux-arnaud-fr" => Self::FluxArnaudFr,
            "flux-benoit-fr" => Self::FluxBenoitFr,
            "flux-brigitte-fr" => Self::FluxBrigitteFr,
            "flux-camille-fr" => Self::FluxCamilleFr,
            "flux-chloe-fr" => Self::FluxChloeFr,
            "flux-florian-fr" => Self::FluxFlorianFr,
            "flux-julien-fr" => Self::FluxJulienFr,
            "flux-mathieu-fr" => Self::FluxMathieuFr,
            "flux-pierre-fr" => Self::FluxPierreFr,
            "flux-thierry-fr" => Self::FluxThierryFr,
            "flux-veronique-fr" => Self::FluxVeroniqueFr,
            "flux-yasmine-fr" => Self::FluxYasmineFr,
            "flux-benjamin-de" => Self::FluxBenjaminDe,
            "flux-christian-de" => Self::FluxChristianDe,
            "flux-janine-de" => Self::FluxJanineDe,
            "flux-magdalena-de" => Self::FluxMagdalenaDe,
            "flux-moritz-de" => Self::FluxMoritzDe,
            "flux-nadine-de" => Self::FluxNadineDe,
            "flux-rolf-de" => Self::FluxRolfDe,
            "flux-sonja-de" => Self::FluxSonjaDe,
            "flux-sven-de" => Self::FluxSvenDe,
            "flux-tobias-de" => Self::FluxTobiasDe,
            "flux-verena-de" => Self::FluxVerenaDe,
            "flux-leonie-de" => Self::FluxLeonieDe,
            "flux-yannick-de" => Self::FluxYannickDe,
            "flux-ajay-hi" => Self::FluxAjayHi,
            "flux-anjali-hi" => Self::FluxAnjaliHi,
            "flux-arjun-hi" => Self::FluxArjunHi,
            "flux-divya-hi" => Self::FluxDivyaHi,
            "flux-jyoti-hi" => Self::FluxJyotiHi,
            "flux-nikhil-hi" => Self::FluxNikhilHi,
            "flux-pallavi-hi" => Self::FluxPallaviHi,
            "flux-prachi-hi" => Self::FluxPrachiHi,
            "flux-rahul-hi" => Self::FluxRahulHi,
            "flux-rajesh-hi" => Self::FluxRajeshHi,
            "flux-ritu-hi" => Self::FluxRituHi,
            "flux-shreya-hi" => Self::FluxShreyaHi,
            "flux-sunita-hi" => Self::FluxSunitaHi,
            "flux-vikram-hi" => Self::FluxVikramHi,
            "flux-angela-it" => Self::FluxAngelaIt,
            "flux-chiara-it" => Self::FluxChiaraIt,
            "flux-federico-it" => Self::FluxFedericoIt,
            "flux-giorgio-it" => Self::FluxGiorgioIt,
            "flux-matteo-it" => Self::FluxMatteoIt,
            "flux-paola-it" => Self::FluxPaolaIt,
            "flux-rocco-it" => Self::FluxRoccoIt,
            "flux-sofia-it" => Self::FluxSofiaIt,
            "flux-stefano-it" => Self::FluxStefanoIt,
            "flux-vincenzo-it" => Self::FluxVincenzoIt,
            "flux-vito-it" => Self::FluxVitoIt,
            "flux-ayumi-ja" => Self::FluxAyumiJa,
            "flux-hayato-ja" => Self::FluxHayatoJa,
            "flux-nozomi-ja" => Self::FluxNozomiJa,
            "flux-sayaka-ja" => Self::FluxSayakaJa,
            "flux-bianca-pt" => Self::FluxBiancaPt,
            "flux-bruno-pt" => Self::FluxBrunoPt,
            "flux-claudia-pt" => Self::FluxClaudiaPt,
            "flux-leandro-pt" => Self::FluxLeandroPt,
            "flux-marcia-pt" => Self::FluxMarciaPt,
            "flux-mariana-pt" => Self::FluxMarianaPt,
            "flux-marta-pt" => Self::FluxMartaPt,
            "flux-mateus-pt" => Self::FluxMateusPt,
            "flux-miguel-pt" => Self::FluxMiguelPt,
            "flux-renato-pt" => Self::FluxRenatoPt,
            "flux-roberto-pt" => Self::FluxRobertoPt,
            "flux-rodrigo-pt" => Self::FluxRodrigoPt,
            "flux-artur-pt" => Self::FluxArturPt,
            "flux-benedita-pt" => Self::FluxBeneditaPt,
            "flux-catarina-pt" => Self::FluxCatarinaPt,
            "flux-hugo-pt" => Self::FluxHugoPt,
            "flux-pedro-pt" => Self::FluxPedroPt,
            "flux-ricardo-pt" => Self::FluxRicardoPt,
            "flux-andrei-ru" => Self::FluxAndreiRu,
            "flux-kirill-ru" => Self::FluxKirillRu,
            "flux-maxim-ru" => Self::FluxMaximRu,
            "flux-mikhail-ru" => Self::FluxMikhailRu,
            "flux-natalia-ru" => Self::FluxNataliaRu,
            "flux-nikolai-ru" => Self::FluxNikolaiRu,
            "flux-olga-ru" => Self::FluxOlgaRu,
            "flux-polina-ru" => Self::FluxPolinaRu,
            "flux-vladimir-ru" => Self::FluxVladimirRu,
            "flux-aina-es" => Self::FluxAinaEs,
            "flux-alfredo-es" => Self::FluxAlfredoEs,
            "flux-carmen-es" => Self::FluxCarmenEs,
            "flux-cecilia-es" => Self::FluxCeciliaEs,
            "flux-duvan-es" => Self::FluxDuvanEs,
            "flux-eduardo-es" => Self::FluxEduardoEs,
            "flux-ernesto-es" => Self::FluxErnestoEs,
            "flux-fernanda-es" => Self::FluxFernandaEs,
            "flux-fiorella-es" => Self::FluxFiorellaEs,
            "flux-gustavo-es" => Self::FluxGustavoEs,
            "flux-jaime-es" => Self::FluxJaimeEs,
            "flux-martin-es" => Self::FluxMartinEs,
            "flux-mauricio-es" => Self::FluxMauricioEs,
            "flux-patricia-es" => Self::FluxPatriciaEs,
            "flux-pilar-es" => Self::FluxPilarEs,
            "flux-rafael-es" => Self::FluxRafaelEs,
            "flux-tomas-es" => Self::FluxTomasEs,
            "flux-valentina-es" => Self::FluxValentinaEs,
            "flux-ysabel-es" => Self::FluxYsabelEs,
            "flux-afsana-bn" => Self::FluxAfsanaBn,
            "flux-ishrat-bn" => Self::FluxIshratBn,
            "flux-mamun-bn" => Self::FluxMamunBn,
            "flux-sabbir-bn" => Self::FluxSabbirBn,
            "flux-shuvo-bn" => Self::FluxShuvoBn,
            "flux-sumaiya-bn" => Self::FluxSumaiyaBn,
            "flux-desislava-bg" => Self::FluxDesislavaBg,
            "flux-gergana-bg" => Self::FluxGerganaBg,
            "flux-jordi-ca" => Self::FluxJordiCa,
            "flux-judit-ca" => Self::FluxJuditCa,
            "flux-bo-zh" => Self::FluxBoZh,
            "flux-haoran-zh" => Self::FluxHaoranZh,
            "flux-jian-zh" => Self::FluxJianZh,
            "flux-jie-zh" => Self::FluxJieZh,
            "flux-lijuan-zh" => Self::FluxLijuanZh,
            "flux-shufen-zh" => Self::FluxShufenZh,
            "flux-wenbo-zh" => Self::FluxWenboZh,
            "flux-wenjing-zh" => Self::FluxWenjingZh,
            "flux-zhen-zh" => Self::FluxZhenZh,
            "flux-chiaying-zh" => Self::FluxChiayingZh,
            "flux-kuanyu-zh" => Self::FluxKuanyuZh,
            "flux-yawen-zh" => Self::FluxYawenZh,
            "flux-yuting-zh" => Self::FluxYutingZh,
            "flux-manlok-zh" => Self::FluxManlokZh,
            "flux-dalibor-hr" => Self::FluxDaliborHr,
            "flux-danijela-hr" => Self::FluxDanijelaHr,
            "flux-domagoj-hr" => Self::FluxDomagojHr,
            "flux-jasna-hr" => Self::FluxJasnaHr,
            "flux-josipa-hr" => Self::FluxJosipaHr,
            "flux-mihaela-hr" => Self::FluxMihaelaHr,
            "flux-vedran-hr" => Self::FluxVedranHr,
            "flux-jakub-cs" => Self::FluxJakubCs,
            "flux-pernille-da" => Self::FluxPernilleDa,
            "flux-rasmus-da" => Self::FluxRasmusDa,
            "flux-rikke-da" => Self::FluxRikkeDa,
            "flux-triin-et" => Self::FluxTriinEt,
            "flux-kirsi-fi" => Self::FluxKirsiFi,
            "flux-teemu-fi" => Self::FluxTeemuFi,
            "flux-kostas-el" => Self::FluxKostasEl,
            "flux-bhavna-gu" => Self::FluxBhavnaGu,
            "flux-hetal-gu" => Self::FluxHetalGu,
            "flux-jignesh-gu" => Self::FluxJigneshGu,
            "flux-bence-hu" => Self::FluxBenceHu,
            "flux-eszter-hu" => Self::FluxEszterHu,
            "flux-kinga-hu" => Self::FluxKingaHu,
            "flux-tibor-hu" => Self::FluxTiborHu,
            "flux-dewi-id" => Self::FluxDewiId,
            "flux-fajar-id" => Self::FluxFajarId,
            "flux-fitri-id" => Self::FluxFitriId,
            "flux-rizky-id" => Self::FluxRizkyId,
            "flux-teguh-id" => Self::FluxTeguhId,
            "flux-manjunath-kn" => Self::FluxManjunathKn,
            "flux-eunji-ko" => Self::FluxEunjiKo,
            "flux-hyunwoo-ko" => Self::FluxHyunwooKo,
            "flux-mikyung-ko" => Self::FluxMikyungKo,
            "flux-minjun-ko" => Self::FluxMinjunKo,
            "flux-soojin-ko" => Self::FluxSoojinKo,
            "flux-sungmin-ko" => Self::FluxSungminKo,
            "flux-agnese-lv" => Self::FluxAgneseLv,
            "flux-edgars-lv" => Self::FluxEdgarsLv,
            "flux-goda-lt" => Self::FluxGodaLt,
            "flux-neringa-lt" => Self::FluxNeringaLt,
            "flux-haziq-ms" => Self::FluxHaziqMs,
            "flux-izzah-ms" => Self::FluxIzzahMs,
            "flux-pooja-mr" => Self::FluxPoojaMr,
            "flux-pratik-mr" => Self::FluxPratikMr,
            "flux-sneha-mr" => Self::FluxSnehaMr,
            "flux-erlend-no" => Self::FluxErlendNo,
            "flux-espen-no" => Self::FluxEspenNo,
            "flux-ida-no" => Self::FluxIdaNo,
            "flux-signe-no" => Self::FluxSigneNo,
            "flux-silje-no" => Self::FluxSiljeNo,
            "flux-vegard-no" => Self::FluxVegardNo,
            "flux-bartosz-pl" => Self::FluxBartoszPl,
            "flux-ewa-pl" => Self::FluxEwaPl,
            "flux-justyna-pl" => Self::FluxJustynaPl,
            "flux-kacper-pl" => Self::FluxKacperPl,
            "flux-konrad-pl" => Self::FluxKonradPl,
            "flux-zuzanna-pl" => Self::FluxZuzannaPl,
            "flux-cosmin-ro" => Self::FluxCosminRo,
            "flux-ioana-ro" => Self::FluxIoanaRo,
            "flux-dominika-sk" => Self::FluxDominikaSk,
            "flux-juraj-sk" => Self::FluxJurajSk,
            "flux-gregor-sl" => Self::FluxGregorSl,
            "flux-katja-sl" => Self::FluxKatjaSl,
            "flux-axel-sv" => Self::FluxAxelSv,
            "flux-charlotta-sv" => Self::FluxCharlottaSv,
            "flux-frida-sv" => Self::FluxFridaSv,
            "flux-ludvig-sv" => Self::FluxLudvigSv,
            "flux-oskar-sv" => Self::FluxOskarSv,
            "flux-pontus-sv" => Self::FluxPontusSv,
            "flux-wilma-sv" => Self::FluxWilmaSv,
            "flux-jerome-tl" => Self::FluxJeromeTl,
            "flux-jomar-tl" => Self::FluxJomarTl,
            "flux-liezel-tl" => Self::FluxLiezelTl,
            "flux-noel-tl" => Self::FluxNoelTl,
            "flux-trisha-tl" => Self::FluxTrishaTl,
            "flux-gokul-ta" => Self::FluxGokulTa,
            "flux-kavya-ta" => Self::FluxKavyaTa,
            "flux-sowmya-ta" => Self::FluxSowmyaTa,
            "flux-vinoth-ta" => Self::FluxVinothTa,
            "flux-pavan-te" => Self::FluxPavanTe,
            "flux-ravi-te" => Self::FluxRaviTe,
            "flux-sravani-te" => Self::FluxSravaniTe,
            "flux-dilek-tr" => Self::FluxDilekTr,
            "flux-ecem-tr" => Self::FluxEcemTr,
            "flux-emre-tr" => Self::FluxEmreTr,
            "flux-gizem-tr" => Self::FluxGizemTr,
            "flux-mehmet-tr" => Self::FluxMehmetTr,
            "flux-serkan-tr" => Self::FluxSerkanTr,
            "flux-zeynep-tr" => Self::FluxZeynepTr,
            "flux-denys-uk" => Self::FluxDenysUk,
            "flux-solomiya-uk" => Self::FluxSolomiyaUk,
            "flux-ambreen-ur" => Self::FluxAmbreenUr,
            "flux-usman-ur" => Self::FluxUsmanUr,
            "flux-hoa-vi" => Self::FluxHoaVi,
            "flux-minh-vi" => Self::FluxMinhVi,

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
        for bad in [
            json!(3),
            json!(-3),
            json!(1.5),
            // The string form is accepted, but only for the five valid
            // values — a quoted out-of-range or non-numeric value is not a
            // way in.
            json!("3"),
            json!("1.5"),
            json!("high"),
            json!(""),
            json!(null),
        ] {
            let err = serde_json::from_value::<DeepgramSpeakExpressivity>(bad.clone());
            assert!(err.is_err(), "expected {bad} to be rejected");
        }
    }

    #[test]
    fn expressivity_serializes_as_a_json_integer_and_accepts_both_forms() {
        // The published AsyncAPI declares `DeepgramSpeakProviderExpressivity`
        // as `type: string` with `enum: ['-2','-1','0','1','2']`, but the
        // live Voice Agent service rejects the quoted form under
        // `agent.speak` with `UNPARSABLE_CLIENT_MESSAGE` and accepts the
        // integer. So the SDK writes the integer, and reads either.
        for (variant, number, string) in [
            (DeepgramSpeakExpressivity::NegativeTwo, -2, "-2"),
            (DeepgramSpeakExpressivity::NegativeOne, -1, "-1"),
            (DeepgramSpeakExpressivity::Zero, 0, "0"),
            (DeepgramSpeakExpressivity::One, 1, "1"),
            (DeepgramSpeakExpressivity::Two, 2, "2"),
        ] {
            let written = serde_json::to_value(variant).unwrap();
            assert!(written.is_number(), "expected an integer, got {written}");
            assert_eq!(written, json!(number));
            assert_eq!(
                serde_json::from_value::<DeepgramSpeakExpressivity>(json!(string)).unwrap(),
                variant
            );
            assert_eq!(
                serde_json::from_value::<DeepgramSpeakExpressivity>(json!(number)).unwrap(),
                variant
            );
        }
    }

    #[test]
    fn speak_settings_expressivity_exact_wire_json_for_every_value() {
        for (variant, expected) in [
            (DeepgramSpeakExpressivity::NegativeTwo, "-2"),
            (DeepgramSpeakExpressivity::NegativeOne, "-1"),
            (DeepgramSpeakExpressivity::Zero, "0"),
            (DeepgramSpeakExpressivity::One, "1"),
            (DeepgramSpeakExpressivity::Two, "2"),
        ] {
            let settings = SpeakSettings::new(SpeakProvider::Deepgram(
                DeepgramSpeakProvider::v2(DeepgramSpeakModel::FluxAlexisEn)
                    .with_expressivity(variant),
            ));
            assert_eq!(
                serde_json::to_string(&settings).unwrap(),
                format!(
                    concat!(
                        r#"{{"provider":{{"type":"deepgram","version":"v2","#,
                        r#""model":"flux-alexis-en","expressivity":{}}}}}"#,
                    ),
                    expected
                )
            );
        }
    }

    #[test]
    fn every_documented_speak_model_round_trips() {
        for name in [
            "aura-asteria-en",
            "aura-luna-en",
            "aura-stella-en",
            "aura-athena-en",
            "aura-hera-en",
            "aura-orion-en",
            "aura-arcas-en",
            "aura-perseus-en",
            "aura-angus-en",
            "aura-orpheus-en",
            "aura-helios-en",
            "aura-zeus-en",
            "aura-2-amalthea-en",
            "aura-2-andromeda-en",
            "aura-2-apollo-en",
            "aura-2-arcas-en",
            "aura-2-aries-en",
            "aura-2-asteria-en",
            "aura-2-athena-en",
            "aura-2-atlas-en",
            "aura-2-aurora-en",
            "aura-2-callista-en",
            "aura-2-cora-en",
            "aura-2-cordelia-en",
            "aura-2-delia-en",
            "aura-2-draco-en",
            "aura-2-electra-en",
            "aura-2-harmonia-en",
            "aura-2-helena-en",
            "aura-2-hera-en",
            "aura-2-hermes-en",
            "aura-2-hyperion-en",
            "aura-2-iris-en",
            "aura-2-janus-en",
            "aura-2-juno-en",
            "aura-2-jupiter-en",
            "aura-2-luna-en",
            "aura-2-mars-en",
            "aura-2-minerva-en",
            "aura-2-neptune-en",
            "aura-2-odysseus-en",
            "aura-2-ophelia-en",
            "aura-2-orion-en",
            "aura-2-orpheus-en",
            "aura-2-pandora-en",
            "aura-2-phoebe-en",
            "aura-2-pluto-en",
            "aura-2-saturn-en",
            "aura-2-selene-en",
            "aura-2-thalia-en",
            "aura-2-theia-en",
            "aura-2-vesta-en",
            "aura-2-zeus-en",
            "aura-2-sirio-es",
            "aura-2-nestor-es",
            "aura-2-carina-es",
            "aura-2-celeste-es",
            "aura-2-alvaro-es",
            "aura-2-diana-es",
            "aura-2-aquila-es",
            "aura-2-selena-es",
            "aura-2-estrella-es",
            "aura-2-javier-es",
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
            "flux-anouk-nl",
            "flux-bram-nl",
            "flux-fleur-nl",
            "flux-irene-nl",
            "flux-luuk-nl",
            "flux-monique-nl",
            "flux-ruben-nl",
            "flux-sanne-nl",
            "flux-yvonne-nl",
            "flux-dirk-nl",
            "flux-lotte-nl",
            "flux-niels-nl",
            "flux-veerle-nl",
            "flux-arnaud-fr",
            "flux-benoit-fr",
            "flux-brigitte-fr",
            "flux-camille-fr",
            "flux-chloe-fr",
            "flux-florian-fr",
            "flux-julien-fr",
            "flux-mathieu-fr",
            "flux-pierre-fr",
            "flux-thierry-fr",
            "flux-veronique-fr",
            "flux-yasmine-fr",
            "flux-benjamin-de",
            "flux-christian-de",
            "flux-janine-de",
            "flux-magdalena-de",
            "flux-moritz-de",
            "flux-nadine-de",
            "flux-rolf-de",
            "flux-sonja-de",
            "flux-sven-de",
            "flux-tobias-de",
            "flux-verena-de",
            "flux-leonie-de",
            "flux-yannick-de",
            "flux-ajay-hi",
            "flux-anjali-hi",
            "flux-arjun-hi",
            "flux-divya-hi",
            "flux-jyoti-hi",
            "flux-nikhil-hi",
            "flux-pallavi-hi",
            "flux-prachi-hi",
            "flux-rahul-hi",
            "flux-rajesh-hi",
            "flux-ritu-hi",
            "flux-shreya-hi",
            "flux-sunita-hi",
            "flux-vikram-hi",
            "flux-angela-it",
            "flux-chiara-it",
            "flux-federico-it",
            "flux-giorgio-it",
            "flux-matteo-it",
            "flux-paola-it",
            "flux-rocco-it",
            "flux-sofia-it",
            "flux-stefano-it",
            "flux-vincenzo-it",
            "flux-vito-it",
            "flux-ayumi-ja",
            "flux-hayato-ja",
            "flux-nozomi-ja",
            "flux-sayaka-ja",
            "flux-bianca-pt",
            "flux-bruno-pt",
            "flux-claudia-pt",
            "flux-leandro-pt",
            "flux-marcia-pt",
            "flux-mariana-pt",
            "flux-marta-pt",
            "flux-mateus-pt",
            "flux-miguel-pt",
            "flux-renato-pt",
            "flux-roberto-pt",
            "flux-rodrigo-pt",
            "flux-artur-pt",
            "flux-benedita-pt",
            "flux-catarina-pt",
            "flux-hugo-pt",
            "flux-pedro-pt",
            "flux-ricardo-pt",
            "flux-andrei-ru",
            "flux-kirill-ru",
            "flux-maxim-ru",
            "flux-mikhail-ru",
            "flux-natalia-ru",
            "flux-nikolai-ru",
            "flux-olga-ru",
            "flux-polina-ru",
            "flux-vladimir-ru",
            "flux-aina-es",
            "flux-alfredo-es",
            "flux-carmen-es",
            "flux-cecilia-es",
            "flux-duvan-es",
            "flux-eduardo-es",
            "flux-ernesto-es",
            "flux-fernanda-es",
            "flux-fiorella-es",
            "flux-gustavo-es",
            "flux-jaime-es",
            "flux-martin-es",
            "flux-mauricio-es",
            "flux-patricia-es",
            "flux-pilar-es",
            "flux-rafael-es",
            "flux-tomas-es",
            "flux-valentina-es",
            "flux-ysabel-es",
            "flux-afsana-bn",
            "flux-ishrat-bn",
            "flux-mamun-bn",
            "flux-sabbir-bn",
            "flux-shuvo-bn",
            "flux-sumaiya-bn",
            "flux-desislava-bg",
            "flux-gergana-bg",
            "flux-jordi-ca",
            "flux-judit-ca",
            "flux-bo-zh",
            "flux-haoran-zh",
            "flux-jian-zh",
            "flux-jie-zh",
            "flux-lijuan-zh",
            "flux-shufen-zh",
            "flux-wenbo-zh",
            "flux-wenjing-zh",
            "flux-zhen-zh",
            "flux-chiaying-zh",
            "flux-kuanyu-zh",
            "flux-yawen-zh",
            "flux-yuting-zh",
            "flux-manlok-zh",
            "flux-dalibor-hr",
            "flux-danijela-hr",
            "flux-domagoj-hr",
            "flux-jasna-hr",
            "flux-josipa-hr",
            "flux-mihaela-hr",
            "flux-vedran-hr",
            "flux-jakub-cs",
            "flux-pernille-da",
            "flux-rasmus-da",
            "flux-rikke-da",
            "flux-triin-et",
            "flux-kirsi-fi",
            "flux-teemu-fi",
            "flux-kostas-el",
            "flux-bhavna-gu",
            "flux-hetal-gu",
            "flux-jignesh-gu",
            "flux-bence-hu",
            "flux-eszter-hu",
            "flux-kinga-hu",
            "flux-tibor-hu",
            "flux-dewi-id",
            "flux-fajar-id",
            "flux-fitri-id",
            "flux-rizky-id",
            "flux-teguh-id",
            "flux-manjunath-kn",
            "flux-eunji-ko",
            "flux-hyunwoo-ko",
            "flux-mikyung-ko",
            "flux-minjun-ko",
            "flux-soojin-ko",
            "flux-sungmin-ko",
            "flux-agnese-lv",
            "flux-edgars-lv",
            "flux-goda-lt",
            "flux-neringa-lt",
            "flux-haziq-ms",
            "flux-izzah-ms",
            "flux-pooja-mr",
            "flux-pratik-mr",
            "flux-sneha-mr",
            "flux-erlend-no",
            "flux-espen-no",
            "flux-ida-no",
            "flux-signe-no",
            "flux-silje-no",
            "flux-vegard-no",
            "flux-bartosz-pl",
            "flux-ewa-pl",
            "flux-justyna-pl",
            "flux-kacper-pl",
            "flux-konrad-pl",
            "flux-zuzanna-pl",
            "flux-cosmin-ro",
            "flux-ioana-ro",
            "flux-dominika-sk",
            "flux-juraj-sk",
            "flux-gregor-sl",
            "flux-katja-sl",
            "flux-axel-sv",
            "flux-charlotta-sv",
            "flux-frida-sv",
            "flux-ludvig-sv",
            "flux-oskar-sv",
            "flux-pontus-sv",
            "flux-wilma-sv",
            "flux-jerome-tl",
            "flux-jomar-tl",
            "flux-liezel-tl",
            "flux-noel-tl",
            "flux-trisha-tl",
            "flux-gokul-ta",
            "flux-kavya-ta",
            "flux-sowmya-ta",
            "flux-vinoth-ta",
            "flux-pavan-te",
            "flux-ravi-te",
            "flux-sravani-te",
            "flux-dilek-tr",
            "flux-ecem-tr",
            "flux-emre-tr",
            "flux-gizem-tr",
            "flux-mehmet-tr",
            "flux-serkan-tr",
            "flux-zeynep-tr",
            "flux-denys-uk",
            "flux-solomiya-uk",
            "flux-ambreen-ur",
            "flux-usman-ur",
            "flux-hoa-vi",
            "flux-minh-vi",
        ] {
            let model = DeepgramSpeakModel::from(name.to_string());
            assert!(
                !matches!(model, DeepgramSpeakModel::Other(_)),
                "{name} should be a named variant"
            );
            assert_eq!(model.is_flux(), name.starts_with("flux-"));
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
