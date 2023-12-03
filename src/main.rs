use anyhow;
use anyhow::{bail, Context};
use clap::Parser;
use clio::{Input, Output};
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::marker::PhantomData;

#[derive(Parser)]
struct Args {
    /// Input file, use '-' for stdin
    #[clap(value_parser, default_value = "-")]
    covidence_ris: Input,

    /// Input file, use '-' for stdin
    #[clap(value_parser, default_value = "-")]
    covidence_csv: Input,

    /// Output file '-' for stdout
    #[clap(long, short, value_parser, default_value = "-")]
    output: Output,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CovidenceRecord {
    title: String,
    _authors: String,
    #[serde(rename = "Abstract")]
    _abstract_: String,
    #[serde(rename = "Published Year")]
    _published_year: String,
    #[serde(rename = "Published Month")]
    _published_month: String,
    _journal: String,
    _volume: String,
    _issue: String,
    _pages: String,
    #[serde(rename = "Accession Number")]
    _accession_number: String,
    #[serde(rename = "DOI")]
    _doi: String,
    #[serde(rename = "Ref")]
    _ref_: String,
    #[serde(rename = "Covidence #")]
    _covidence_number: String,
    _study: String,
    _notes: String,
    #[serde(deserialize_with = "deserialize_tags")]
    tags: Vec<String>,
}

fn deserialize_tags<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    struct Tags(PhantomData<Vec<String>>);

    impl<'de> de::Visitor<'de> for Tags {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value
                .split(";")
                .map(str::trim)
                .map(str::to_string)
                .collect())
        }
    }

    d.deserialize_string(Tags(PhantomData))
}

#[derive(Default)]
enum Parsing {
    #[default]
    StartParsing,
    WaitingForNextRecord,
    LookingForTitle,
    FoundTitle(String),
}

#[derive(Default)]
struct ParsingContext {
    line_number: u32,
    state: Parsing,
}

impl ParsingContext {
    fn next_line(mut self) -> Self {
        self.line_number += 1;
        self
    }

    fn next_line_state(mut self, state: Parsing) -> Self {
        self.line_number += 1;
        self.state = state;
        self
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let covidence_records: HashMap<String, CovidenceRecord> =
        csv::Reader::from_reader(args.covidence_csv)
            .into_deserialize()
            .map(|r| {
                let record: anyhow::Result<CovidenceRecord> =
                    r.context("Failed to deserialize record into struct");
                record.map(|r| (r.title.clone(), r))
            })
            .collect::<anyhow::Result<HashMap<String, CovidenceRecord>>>()?;
    let covidence_ris = BufReader::new(args.covidence_ris);
    let mut merged_ris = BufWriter::new(args.output);
    let final_state = covidence_ris
        .lines()
        .try_fold(ParsingContext::default(), |context, line| {
            let line = line?;
            let state = &context.state;
            if line.is_empty() {
                return Ok(context.next_line());
            }

            let Some((tag_type, tag_info)) = line
                .split_once("-") else {
                merged_ris.write_all(line.as_bytes())?;
                merged_ris.write_all(b"\n")?;
                return Ok(context.next_line())
            };

            let tag_info = tag_info.trim();
            match (state, tag_type.trim()) {
                (Parsing::StartParsing, "TY") | (Parsing::WaitingForNextRecord, "TY") => {
                    merged_ris.write_all(line.as_bytes())?;
                    merged_ris.write_all(b"\n")?;
                    Ok(context.next_line_state(Parsing::LookingForTitle))
                },
                (Parsing::StartParsing, _) => {
                    bail!("File started with invalid tag {}", tag_type)
                },
                (Parsing::WaitingForNextRecord, _) => {
                    bail!("Record started with invalid tag {} while attempting to read the next record on line {}", tag_type, context.line_number)
                }
                (Parsing::LookingForTitle, "TI") => {
                    merged_ris.write_all(line.as_bytes())?;
                    merged_ris.write_all(b"\n")?;
                    Ok(context.next_line_state(Parsing::FoundTitle(tag_info.to_string())))
                },
                (Parsing::LookingForTitle, "ER") => {
                    bail!("Unable to find title in record before reaching the end of the record on line {}", context.line_number)
                }
                (Parsing::FoundTitle(title), "ER") => {
                    for record_tag in covidence_records.get(title).with_context(|| format!("Failed to retrieve covidence record for document \"{}\" on line {}", title, context.line_number))?.tags.iter() {
                        merged_ris.write_all(b"KW  - ")?;
                        merged_ris.write_all(record_tag.as_bytes())?;
                        merged_ris.write_all(b"\n")?;
                    }

                    merged_ris.write_all(line.as_bytes())?;
                    merged_ris.write_all(b"\n")?;
                    Ok(context.next_line_state(Parsing::WaitingForNextRecord))
                },
                _ => {
                    merged_ris.write_all(line.as_bytes())?;
                    merged_ris.write_all(b"\n")?;
                    Ok(context.next_line())
                }
            }
        })?.state;

    match final_state {
        Parsing::StartParsing => {
            bail!("No records in file")
        }
        Parsing::LookingForTitle => {
            bail!("Last record in file did not have an end")
        }
        Parsing::FoundTitle(_) | Parsing::WaitingForNextRecord => Ok(()),
    }
}
