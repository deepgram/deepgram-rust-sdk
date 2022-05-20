use super::*;

fn check_serialization(options: &OptionsBuilder, expected: &str) {
    let actual = {
        let mut serializer = form_urlencoded::Serializer::new(String::new());

        options
            .serialize(serde_urlencoded::Serializer::new(&mut serializer))
            .unwrap();

        serializer.finish()
    };

    assert_eq!(actual, expected);
}

#[test]
fn all_options() {
    let options = OptionsBuilder::new()
        .model(Model::General)
        .version("1.2.3")
        .language(Language::en_US)
        .punctuate(true)
        .profanity_filter(true)
        .redact([Redact::Pci, Redact::Ssn])
        .diarize(true)
        .ner(true)
        .multichannel(true)
        .alternatives(4)
        .numerals(true)
        .search(["Rust", "Deepgram"])
        .callback("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
        .keywords(["Ferris", "Cargo"])
        .utterances(Utterances::Enabled {
            utt_split: Some(0.9),
        })
        .tag("SDK Test");

    check_serialization(&options, "model=general&version=1.2.3&language=en-US&punctuate=true&profanity_filter=true&redact=pci&redact=ssn&diarize=true&ner=true&multichannel=true&alternatives=4&numerals=true&search=Rust&search=Deepgram&callback=https%3A%2F%2Fwww.youtube.com%2Fwatch%3Fv%3DdQw4w9WgXcQ&keywords=Ferris&keywords=Cargo&utterances=true&utt_split=0.9&tag=SDK+Test");
}

#[test]
fn model() {
    check_serialization(
        &OptionsBuilder::new().model(Model::General),
        "model=general",
    );

    check_serialization(
        &OptionsBuilder::new().model(Model::CustomId("extra_crispy")),
        "model=extra_crispy",
    );
}

#[test]
fn version() {
    check_serialization(&OptionsBuilder::new().version("1.2.3"), "version=1.2.3");
}

#[test]
fn language() {
    check_serialization(
        &OptionsBuilder::new().language(Language::en_US),
        "language=en-US",
    );

    check_serialization(&OptionsBuilder::new().language(Language::ja), "language=ja");
}

#[test]
fn punctuate() {
    check_serialization(&OptionsBuilder::new().punctuate(true), "punctuate=true");

    check_serialization(&OptionsBuilder::new().punctuate(false), "punctuate=false");
}

#[test]
fn profanity_filter() {
    check_serialization(
        &OptionsBuilder::new().profanity_filter(true),
        "profanity_filter=true",
    );

    check_serialization(
        &OptionsBuilder::new().profanity_filter(false),
        "profanity_filter=false",
    );
}

#[test]
fn redact() {
    check_serialization(&OptionsBuilder::new().redact([]), "");

    check_serialization(
        &OptionsBuilder::new().redact([Redact::Numbers]),
        "redact=numbers",
    );

    check_serialization(
        &OptionsBuilder::new().redact([Redact::Ssn, Redact::Pci]),
        "redact=ssn&redact=pci",
    );
}

#[test]
fn diarize() {
    check_serialization(&OptionsBuilder::new().diarize(true), "diarize=true");

    check_serialization(&OptionsBuilder::new().diarize(false), "diarize=false");
}

#[test]
fn ner() {
    check_serialization(&OptionsBuilder::new().ner(true), "ner=true");

    check_serialization(&OptionsBuilder::new().ner(false), "ner=false");
}

#[test]
fn multichannel() {
    check_serialization(
        &OptionsBuilder::new().multichannel(true),
        "multichannel=true",
    );

    check_serialization(
        &OptionsBuilder::new().multichannel(false),
        "multichannel=false",
    );
}

#[test]
fn alternatives() {
    check_serialization(&OptionsBuilder::new().alternatives(4), "alternatives=4");
}

#[test]
fn numerals() {
    check_serialization(&OptionsBuilder::new().numerals(true), "numerals=true");

    check_serialization(&OptionsBuilder::new().numerals(false), "numerals=false");
}

#[test]
fn search() {
    check_serialization(&OptionsBuilder::new().search([]), "");

    check_serialization(&OptionsBuilder::new().search(["Rust"]), "search=Rust");

    check_serialization(
        &OptionsBuilder::new().search(["Rust", "Deepgram"]),
        "search=Rust&search=Deepgram",
    );
}

#[test]
fn callback() {
    check_serialization(
        &OptionsBuilder::new().callback("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
        "callback=https%3A%2F%2Fwww.youtube.com%2Fwatch%3Fv%3DdQw4w9WgXcQ",
    );
}

#[test]
fn keywords() {
    check_serialization(&OptionsBuilder::new().keywords([]), "");

    check_serialization(
        &OptionsBuilder::new().keywords(["Ferris"]),
        "keywords=Ferris",
    );

    check_serialization(
        &OptionsBuilder::new().keywords(["Ferris", "Cargo"]),
        "keywords=Ferris&keywords=Cargo",
    );
}

#[test]
fn utterances() {
    check_serialization(
        &OptionsBuilder::new().utterances(Utterances::Disabled),
        "utterances=false",
    );

    check_serialization(
        &OptionsBuilder::new().utterances(Utterances::Enabled { utt_split: None }),
        "utterances=true",
    );

    check_serialization(
        &OptionsBuilder::new().utterances(Utterances::Enabled {
            utt_split: Some(0.9),
        }),
        "utterances=true&utt_split=0.9",
    );
}

#[test]
fn tag() {
    check_serialization(&OptionsBuilder::new().tag("SDK Test"), "tag=SDK+Test");
}
