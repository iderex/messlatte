//! The submission a method returns, and what its validator refuses (#38).
//!
//! A submission is one JSON document, written in the canonical order `json`
//! fixes, so that two submissions with the same content are the same bytes and
//! a hash over one means something. One start writes one file. The ensemble
//! file that sits over a run's starts is #71, and nothing here assumes it
//! exists: a start that failed leaves the other starts readable, and an author
//! outside this workspace can submit starts as they finish.
//!
//! What a method hands back is the retrieved object and the record of how it
//! got there, and nothing about how well it did. How well it did is the
//! scorer's output. A submission carrying its own error would be a claim the
//! scorer would then have to either believe or contradict, and the format
//! avoids the question by having no field for it.
//!
//! The retrieved object is the complex field on a time grid, or the spectral
//! amplitude and phase on a photon-energy grid, because the methods argue in
//! different domains and converting on the way out would make one of them
//! carry a transform it did not choose. The two forms are alternatives rather
//! than halves: a document carrying both would have two answers with nothing to
//! say which was meant. `docs/format/submission.md` is where the conversion
//! between them is defined, in one place, so that a method may submit either
//! and be read the same way.
//!
//! What the validator does with a fault is refuse it. It never repairs one: a
//! repaired submission is a submission whose author does not know what was
//! scored, and the difference between what they ran and what was scored is
//! exactly the thing a benchmark cannot afford to invent.
//!
//! Two of the refusals need something the document cannot carry, because they
//! are about the fit between a submission and the case it claims to be about.
//! A known this case did not offer, and a case identifier that is not the one
//! being scored, are both refused by [`Submission::against`], which takes the
//! case's own side of the comparison from its caller. The case declaration
//! that will supply it is #37 and does not exist; until it does, a caller
//! states the two directly and the refusals are proved against that.
//!
//! What is not here. The path a submission is written to is a convention of the
//! case directory, in [`FILE`]. The version rule is #41 and is applied here in
//! the same narrow form the trace uses; the three modules in this crate each
//! carry their own [`Version`] today, and unifying them is that issue's rather
//! than this one's.

use std::fmt;

use messlatte_units::{Energy, Momentum, Time};

use crate::json::{self, Json, Object};
use crate::trace::Axis;

/// What the document calls this format.
pub const FORMAT: &str = "messlatte-submission";

/// The version this module writes.
pub const VERSION: Version = Version { major: 1, minor: 0 };

/// The conventional name of a submission inside a case directory.
///
/// One start writes one file, so a run over several starts writes several of
/// them and the start identifier is what tells them apart. Where they sit
/// relative to each other is #39 and is not decided here.
pub const FILE: &str = "submission.json";

/// The largest whole number a double holds exactly, which is the ceiling on
/// every count in this format.
///
/// The header's numbers are doubles, so a count beyond this reads back as a
/// neighbour of itself. Nothing legitimate here comes near it: it is nine
/// thousand million million iterations.
pub const LARGEST_COUNT: f64 = 9_007_199_254_740_992.0;

/// A format version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// The form a method's stopping rule takes.
///
/// The three forms are the ones issue #12 admits and the enumeration is closed,
/// so a run cannot be stopped by a rule nobody can describe. That decision is
/// the reason this is an enum rather than a sentence: the practice this board
/// is a response to is a method stopped by somebody watching a number, whose
/// patience is then part of the published result and is recorded nowhere.
///
/// The fixed count is the default for the matrix, because it is the only form
/// that makes the cost comparison mean anything and the only one that cannot be
/// tuned per case without leaving a trace in this file.
#[derive(Debug, Clone, PartialEq)]
pub enum Rule {
    /// Stop after this many iterations, whatever the figure of merit does.
    FixedCount { iterations: f64 },
    /// Stop when the relative change in the figure of merit stays below
    /// `threshold` for `window` consecutive iterations.
    RelativeChange { threshold: f64, window: f64 },
    /// Stop after this many seconds. Admitted for the cost comparison and
    /// never for a scored run, because the same rule on two machines is two
    /// different methods.
    WallClock { seconds: f64 },
}

impl Rule {
    fn form(&self) -> &'static str {
        match self {
            Rule::FixedCount { .. } => "fixed-count",
            Rule::RelativeChange { .. } => "relative-change",
            Rule::WallClock { .. } => "wall-clock",
        }
    }
}

/// Where a method stopped and what it was looking at when it did.
#[derive(Debug, Clone, PartialEq)]
pub struct Stopping {
    /// The rule the run was given, as a parameter of the run rather than as a
    /// property of the method.
    pub rule: Rule,
    /// The iteration the method stopped at.
    pub stopped_at: f64,
    /// The method's own figure of merit at that iteration. It is the method's
    /// number, on the method's own definition, and it is not comparable across
    /// methods. The comparable one is the scorer's and is not in this file.
    pub merit: f64,
    /// Why it stopped, in the author's own words, for the case where the rule
    /// alone does not say.
    pub why: String,
}

/// The object a method retrieved, in one domain or the other.
#[derive(Debug, Clone, PartialEq)]
pub enum Retrieved {
    /// The complex field on a time grid, as two real columns.
    ///
    /// Two columns rather than one of pairs, because a column of pairs has an
    /// order somebody has to state and a reader can get backwards while every
    /// number still looks reasonable.
    Field {
        time: Axis,
        real: Vec<f64>,
        imaginary: Vec<f64>,
    },
    /// The spectral amplitude and phase on a photon-energy grid.
    Spectrum {
        energy: Axis,
        amplitude: Vec<f64>,
        phase: Vec<f64>,
    },
}

impl Retrieved {
    fn quantity(&self) -> &'static str {
        match self {
            Retrieved::Field { .. } => "field",
            Retrieved::Spectrum { .. } => "spectrum",
        }
    }
}

/// The streaking field a method retrieved, where it retrieves one.
///
/// Written as the momentum the field imparts rather than as a vector potential
/// in atomic units, because atomic units belong inside the numerics and a file
/// is what an operator reads, which is #13. In atomic units the vector
/// potential of `docs/format/streaking-field.md` and that momentum are the same
/// number, so this is a spelling of that quantity in the unit an operator can
/// check, and not a different quantity.
#[derive(Debug, Clone, PartialEq)]
pub struct Streaking {
    pub time: Axis,
    /// The unit of the values, from the momentum units the conversion layer
    /// admits.
    pub unit: String,
    pub values: Vec<f64>,
}

/// One submission, from one start.
#[derive(Debug, Clone, PartialEq)]
pub struct Submission {
    /// The case this claims to be about. Checked against the case being scored
    /// by [`Submission::against`] and not by the reader, which has no way to
    /// know.
    pub case: String,
    /// The start within the run. Two submissions of one case differ here.
    pub start: String,
    /// The seed that reproduces this start on its own, without rerunning the
    /// starts before it.
    pub seed: u64,
    /// The declared knowns the method read. What it was offered is a property
    /// of the case, and reading one it was not offered is refused rather than
    /// scored.
    pub knowns: Vec<String>,
    pub stopping: Stopping,
    pub retrieved: Retrieved,
    pub streaking: Option<Streaking>,
}

/// The case's side of the two refusals a document cannot carry.
///
/// Supplied by whoever is scoring, because a submission cannot be trusted to
/// state what the case offered: that is exactly the field a method that read
/// too much would fill in wrongly.
#[derive(Debug, Clone, Copy)]
pub struct Scoring<'a> {
    /// The identifier of the case directory being scored.
    pub case: &'a str,
    /// The knowns that case declares.
    pub offered: &'a [String],
}

/// One reason a submission is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The document could not be read at all.
    Unreadable { detail: String },
    /// The document names a different format.
    NotASubmission { found: String },
    /// A major version this reader does not know.
    UnknownMajorVersion { found: Version },
    /// A field the format requires is absent, or is of the wrong shape.
    Field { path: String, wanted: String },
    /// A field that carries nothing where it has to carry a name or a
    /// sentence.
    Blank { field: String },
    /// A unit outside the set the conversion layer admits.
    UnknownUnit {
        grid: String,
        unit: String,
        admitted: String,
    },
    /// A grid with no samples on it.
    EmptyGrid { grid: String },
    /// A grid that does not strictly increase.
    GridNotIncreasing { grid: String, index: usize },
    /// A column whose length disagrees with the grid it is on.
    LengthDisagrees {
        column: String,
        grid: String,
        samples: usize,
        values: usize,
    },
    /// A value that is not a number.
    NotFinite { column: String, index: usize },
    /// A spectral amplitude below zero.
    AmplitudeNegative { index: usize },
    /// No record of where the method stopped.
    NoStoppingRecord,
    /// A declared known the case did not offer.
    KnownNotOffered { known: String, offered: String },
    /// A case identifier that is not the case being scored.
    CaseMismatch { found: String, scoring: String },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::Unreadable { detail } => {
                write!(f, "the submission could not be read: {detail}")
            }
            Refusal::NotASubmission { found } => write!(
                f,
                "the document declares the format {found:?} and a submission declares {FORMAT:?}"
            ),
            Refusal::UnknownMajorVersion { found } => write!(
                f,
                "the document declares version {found} and this reads major version {}. A major \
                 version is a change of meaning, so this refuses it rather than guessing which \
                 fields still mean what they did",
                VERSION.major
            ),
            Refusal::Field { path, wanted } => write!(f, "the document's {path} is not {wanted}"),
            Refusal::Blank { field } => write!(
                f,
                "the document's {field} carries nothing. A submission whose {field} was never \
                 filled in is one nobody can place"
            ),
            Refusal::UnknownUnit {
                grid,
                unit,
                admitted,
            } => write!(
                f,
                "the {grid} grid is in {unit:?}, and a file states this quantity in {admitted}"
            ),
            Refusal::EmptyGrid { grid } => write!(
                f,
                "the {grid} grid carries no samples, so the columns on it are about nothing"
            ),
            Refusal::GridNotIncreasing { grid, index } => write!(
                f,
                "the {grid} grid does not increase at sample {index}. A grid is strictly \
                 increasing, and two samples at one position are two values of one thing with \
                 nothing to say which is meant"
            ),
            Refusal::LengthDisagrees {
                column,
                grid,
                samples,
                values,
            } => write!(
                f,
                "the {column} column carries {values} values and the {grid} grid carries \
                 {samples} samples, and a sample is both"
            ),
            Refusal::NotFinite { column, index } => write!(
                f,
                "the {column} column carries a value at sample {index} that is not finite. A \
                 sample a method did not retrieve is absent from the grid rather than written \
                 as a value, so there is no sentinel here for a scorer to mistake for a number"
            ),
            Refusal::AmplitudeNegative { index } => write!(
                f,
                "the spectral amplitude at sample {index} is below zero. An amplitude is a \
                 modulus and the sign belongs in the phase, so this is a phase written into the \
                 wrong column"
            ),
            Refusal::NoStoppingRecord => write!(
                f,
                "the submission carries no stopping record. Where a method stopped is what \
                 separates the algorithm from whoever was watching it, so a run that does not \
                 say is not scored"
            ),
            Refusal::KnownNotOffered { known, offered } => write!(
                f,
                "the method declares it read {known:?}, and this case offers {offered}. A \
                 method that read something the case did not declare was not solving this case"
            ),
            Refusal::CaseMismatch { found, scoring } => write!(
                f,
                "the submission is for case {found:?} and the case being scored is {scoring:?}"
            ),
        }
    }
}

impl Submission {
    /// The document.
    ///
    /// The writer applies the same refusals the reader does, minus the two that
    /// need a case to compare against. A writer that could emit a document its
    /// own reader rejects would put the check after the file exists.
    pub fn to_bytes(&self) -> Result<Vec<u8>, Vec<Refusal>> {
        let refusals = self.refusals();
        if !refusals.is_empty() {
            return Err(refusals);
        }

        let mut format = Object::new();
        format.insert("name".to_string(), Json::String(FORMAT.to_string()));
        format.insert("version".to_string(), Json::String(VERSION.to_string()));

        let mut rule = Object::new();
        rule.insert(
            "form".to_string(),
            Json::String(self.stopping.rule.form().to_string()),
        );
        match &self.stopping.rule {
            Rule::FixedCount { iterations } => {
                rule.insert("iterations".to_string(), Json::Number(*iterations));
            }
            Rule::RelativeChange { threshold, window } => {
                rule.insert("threshold".to_string(), Json::Number(*threshold));
                rule.insert("window".to_string(), Json::Number(*window));
            }
            Rule::WallClock { seconds } => {
                rule.insert("seconds".to_string(), Json::Number(*seconds));
            }
        }

        let mut stopping = Object::new();
        stopping.insert("merit".to_string(), Json::Number(self.stopping.merit));
        stopping.insert("rule".to_string(), Json::Object(rule));
        stopping.insert(
            "stopped-at".to_string(),
            Json::Number(self.stopping.stopped_at),
        );
        stopping.insert("why".to_string(), Json::String(self.stopping.why.clone()));

        let mut retrieved = Object::new();
        retrieved.insert(
            "quantity".to_string(),
            Json::String(self.retrieved.quantity().to_string()),
        );
        match &self.retrieved {
            Retrieved::Field {
                time,
                real,
                imaginary,
            } => {
                retrieved.insert("imaginary".to_string(), numbers(imaginary));
                retrieved.insert("real".to_string(), numbers(real));
                retrieved.insert("time".to_string(), grid(time));
            }
            Retrieved::Spectrum {
                energy,
                amplitude,
                phase,
            } => {
                retrieved.insert("amplitude".to_string(), numbers(amplitude));
                retrieved.insert("energy".to_string(), grid(energy));
                retrieved.insert("phase".to_string(), numbers(phase));
            }
        }

        let mut document = Object::new();
        document.insert("case".to_string(), Json::String(self.case.clone()));
        document.insert("format".to_string(), Json::Object(format));
        document.insert(
            "knowns".to_string(),
            Json::Array(
                self.knowns
                    .iter()
                    .map(|known| Json::String(known.clone()))
                    .collect(),
            ),
        );
        document.insert("retrieved".to_string(), Json::Object(retrieved));
        document.insert(
            "seed".to_string(),
            Json::String(format!("{:016x}", self.seed)),
        );
        document.insert("start".to_string(), Json::String(self.start.clone()));
        document.insert("stopping".to_string(), Json::Object(stopping));
        if let Some(streaking) = &self.streaking {
            let mut object = Object::new();
            object.insert("time".to_string(), grid(&streaking.time));
            object.insert("unit".to_string(), Json::String(streaking.unit.clone()));
            object.insert("values".to_string(), numbers(&streaking.values));
            document.insert("streaking".to_string(), Json::Object(object));
        }

        Json::Object(document)
            .to_bytes()
            .map_err(|detail| vec![Refusal::Unreadable { detail }])
    }

    /// A submission from its bytes, with everything the document alone decides.
    ///
    /// The two refusals that need the case are [`Submission::against`], and a
    /// scorer runs both. They are separate calls rather than one so that a
    /// reader outside a scoring run, checking a file it is about to send, gets
    /// the answer it can get rather than an argument about a case it does not
    /// have.
    pub fn from_bytes(bytes: &[u8]) -> Result<Submission, Vec<Refusal>> {
        let document = json::parse(bytes).map_err(|detail| vec![Refusal::Unreadable { detail }])?;

        // The format and the version come first, because every field below
        // means what this version says it means, and a complaint about a field
        // in a document from another format sends a reader after the wrong
        // thing.
        let name = string(&field(&document, "format", "name")?, "format.name")?;
        if name != FORMAT {
            return Err(vec![Refusal::NotASubmission { found: name }]);
        }
        let version = version_of(&document)?;
        if version.major != VERSION.major {
            return Err(vec![Refusal::UnknownMajorVersion { found: version }]);
        }

        let case = string(&field(&document, "case", "")?, "case")?;
        let start = string(&field(&document, "start", "")?, "start")?;
        let seed = seed_of(&document)?;
        let knowns = knowns_of(&document)?;
        let stopping = stopping_of(&document)?;
        let retrieved = retrieved_of(&document)?;
        let streaking = streaking_of(&document)?;

        let submission = Submission {
            case,
            start,
            seed,
            knowns,
            stopping,
            retrieved,
            streaking,
        };
        let refusals = submission.refusals();
        if refusals.is_empty() {
            Ok(submission)
        } else {
            Err(refusals)
        }
    }

    /// The two refusals that are about the fit between this submission and the
    /// case being scored.
    pub fn against(&self, scoring: &Scoring<'_>) -> Vec<Refusal> {
        let mut found = Vec::new();
        if self.case != scoring.case {
            found.push(Refusal::CaseMismatch {
                found: self.case.clone(),
                scoring: scoring.case.to_string(),
            });
        }
        let offered = if scoring.offered.is_empty() {
            "nothing".to_string()
        } else {
            scoring.offered.join(", ")
        };
        for known in &self.knowns {
            if !scoring.offered.iter().any(|entry| entry == known) {
                found.push(Refusal::KnownNotOffered {
                    known: known.clone(),
                    offered: offered.clone(),
                });
            }
        }
        found
    }

    /// Everything wrong with this submission that the document alone decides,
    /// in one pass.
    ///
    /// All of them rather than the first, because a submission with a bad grid
    /// and a bad column is one file somebody has to fix once, and a validator
    /// reporting one refusal per run would make them fix it twice.
    fn refusals(&self) -> Vec<Refusal> {
        let mut found = Vec::new();
        blank("case", &self.case, &mut found);
        blank("start", &self.start, &mut found);
        blank("stopping.why", &self.stopping.why, &mut found);
        for (index, known) in self.knowns.iter().enumerate() {
            blank(&format!("knowns[{index}]"), known, &mut found);
        }

        count("stopping.stopped-at", self.stopping.stopped_at, &mut found);
        if !self.stopping.merit.is_finite() {
            found.push(Refusal::Field {
                path: "stopping.merit".to_string(),
                wanted: "a finite number".to_string(),
            });
        }
        match &self.stopping.rule {
            Rule::FixedCount { iterations } => {
                count("stopping.rule.iterations", *iterations, &mut found);
            }
            Rule::RelativeChange { threshold, window } => {
                if !threshold.is_finite() {
                    found.push(Refusal::Field {
                        path: "stopping.rule.threshold".to_string(),
                        wanted: "a finite number".to_string(),
                    });
                }
                count("stopping.rule.window", *window, &mut found);
            }
            Rule::WallClock { seconds } => {
                if !seconds.is_finite() {
                    found.push(Refusal::Field {
                        path: "stopping.rule.seconds".to_string(),
                        wanted: "a finite number".to_string(),
                    });
                }
            }
        }

        match &self.retrieved {
            Retrieved::Field {
                time,
                real,
                imaginary,
            } => {
                check_grid("retrieved.time", time, Time::UNITS, &mut found);
                check_column("retrieved.real", "retrieved.time", real, time, &mut found);
                check_column(
                    "retrieved.imaginary",
                    "retrieved.time",
                    imaginary,
                    time,
                    &mut found,
                );
            }
            Retrieved::Spectrum {
                energy,
                amplitude,
                phase,
            } => {
                check_grid("retrieved.energy", energy, Energy::UNITS, &mut found);
                check_column(
                    "retrieved.amplitude",
                    "retrieved.energy",
                    amplitude,
                    energy,
                    &mut found,
                );
                check_column(
                    "retrieved.phase",
                    "retrieved.energy",
                    phase,
                    energy,
                    &mut found,
                );
                for (index, value) in amplitude.iter().enumerate() {
                    // Written through the ordering so that a value which
                    // compares to nothing is left to the finiteness check above
                    // rather than reported twice.
                    if matches!(value.partial_cmp(&0.0), Some(std::cmp::Ordering::Less)) {
                        found.push(Refusal::AmplitudeNegative { index });
                    }
                }
            }
        }

        if let Some(streaking) = &self.streaking {
            check_grid("streaking.time", &streaking.time, Time::UNITS, &mut found);
            if !Momentum::UNITS.contains(&streaking.unit.as_str()) {
                found.push(Refusal::UnknownUnit {
                    grid: "streaking".to_string(),
                    unit: streaking.unit.clone(),
                    admitted: Momentum::UNITS.join(", "),
                });
            }
            check_column(
                "streaking.values",
                "streaking.time",
                &streaking.values,
                &streaking.time,
                &mut found,
            );
        }
        found
    }
}

fn blank(field: &str, value: &str, found: &mut Vec<Refusal>) {
    if value.trim().is_empty() {
        found.push(Refusal::Blank {
            field: field.to_string(),
        });
    }
}

/// A count is a whole number a double holds exactly, and never negative.
fn count(path: &str, value: f64, found: &mut Vec<Refusal>) {
    let whole = value.is_finite()
        && matches!(
            value.partial_cmp(&0.0),
            Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
        )
        && matches!(
            value.partial_cmp(&LARGEST_COUNT),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
        )
        && matches!(
            value.fract().partial_cmp(&0.0),
            Some(std::cmp::Ordering::Equal)
        );
    if !whole {
        found.push(Refusal::Field {
            path: path.to_string(),
            wanted: "a whole number of iterations that a double holds exactly".to_string(),
        });
    }
}

fn check_grid(name: &str, axis: &Axis, admitted: &[&str], found: &mut Vec<Refusal>) {
    if !admitted.contains(&axis.unit.as_str()) {
        found.push(Refusal::UnknownUnit {
            grid: name.to_string(),
            unit: axis.unit.clone(),
            admitted: admitted.join(", "),
        });
    }
    if axis.values.is_empty() {
        found.push(Refusal::EmptyGrid {
            grid: name.to_string(),
        });
        return;
    }
    finite(&format!("{name}.values"), &axis.values, found);
    for index in 1..axis.values.len() {
        // Written through the ordering rather than as a negated comparison, so
        // that a value which compares to nothing is refused by the finiteness
        // check above and not silently accepted here.
        let increases = matches!(
            axis.values[index].partial_cmp(&axis.values[index - 1]),
            Some(std::cmp::Ordering::Greater)
        );
        if !increases {
            found.push(Refusal::GridNotIncreasing {
                grid: name.to_string(),
                index,
            });
        }
    }
}

fn check_column(name: &str, grid: &str, values: &[f64], axis: &Axis, found: &mut Vec<Refusal>) {
    if values.len() != axis.values.len() {
        found.push(Refusal::LengthDisagrees {
            column: name.to_string(),
            grid: grid.to_string(),
            samples: axis.values.len(),
            values: values.len(),
        });
    }
    finite(name, values, found);
}

fn finite(name: &str, values: &[f64], found: &mut Vec<Refusal>) {
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            found.push(Refusal::NotFinite {
                column: name.to_string(),
                index,
            });
        }
    }
}

fn numbers(values: &[f64]) -> Json {
    Json::Array(values.iter().map(|value| Json::Number(*value)).collect())
}

fn grid(axis: &Axis) -> Json {
    let mut object = Object::new();
    object.insert("unit".to_string(), Json::String(axis.unit.clone()));
    object.insert("values".to_string(), numbers(&axis.values));
    Json::Object(object)
}

/// One field, named by its object and its member, where an empty member name
/// asks for a member of the document itself.
fn field(document: &Json, object: &str, member: &str) -> Result<Json, Vec<Refusal>> {
    let path = if member.is_empty() {
        object.to_string()
    } else {
        format!("{object}.{member}")
    };
    let value = if member.is_empty() {
        document.get(object)
    } else {
        document.get(object).and_then(|value| value.get(member))
    };
    value.cloned().ok_or_else(|| {
        vec![Refusal::Field {
            path,
            wanted: "present".to_string(),
        }]
    })
}

fn string(value: &Json, path: &str) -> Result<String, Vec<Refusal>> {
    value.as_str().map(str::to_string).ok_or_else(|| {
        vec![Refusal::Field {
            path: path.to_string(),
            wanted: "a string".to_string(),
        }]
    })
}

fn number(value: &Json, path: &str) -> Result<f64, Vec<Refusal>> {
    value.as_number().ok_or_else(|| {
        vec![Refusal::Field {
            path: path.to_string(),
            wanted: "a number".to_string(),
        }]
    })
}

fn version_of(document: &Json) -> Result<Version, Vec<Refusal>> {
    let text = string(&field(document, "format", "version")?, "format.version")?;
    let wrong = || {
        vec![Refusal::Field {
            path: "format.version".to_string(),
            wanted: "a version written as major.minor".to_string(),
        }]
    };
    let (major, minor) = text.split_once('.').ok_or_else(wrong)?;
    let major = major.parse::<u32>().map_err(|_| wrong())?;
    let minor = minor.parse::<u32>().map_err(|_| wrong())?;
    Ok(Version { major, minor })
}

/// The seed, as sixteen lowercase hexadecimal digits.
///
/// A string rather than a number because a seed is sixty-four bits and this
/// document's numbers are doubles, so a seed written as a number would read
/// back as a neighbour of itself and reproduce a different start. Fixed width
/// and one case, so that two documents with the same seed are the same bytes.
fn seed_of(document: &Json) -> Result<u64, Vec<Refusal>> {
    let text = string(&field(document, "seed", "")?, "seed")?;
    let wrong = || {
        vec![Refusal::Field {
            path: "seed".to_string(),
            wanted: "sixteen lowercase hexadecimal digits".to_string(),
        }]
    };
    if text.len() != 16
        || text
            .chars()
            .any(|c| !c.is_ascii_hexdigit() || c.is_ascii_uppercase())
    {
        return Err(wrong());
    }
    u64::from_str_radix(&text, 16).map_err(|_| wrong())
}

fn knowns_of(document: &Json) -> Result<Vec<String>, Vec<Refusal>> {
    let items = field(document, "knowns", "")?;
    let items = items.as_array().ok_or_else(|| {
        vec![Refusal::Field {
            path: "knowns".to_string(),
            wanted: "an array".to_string(),
        }]
    })?;
    let mut knowns = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        knowns.push(string(item, &format!("knowns[{index}]"))?);
    }
    Ok(knowns)
}

fn stopping_of(document: &Json) -> Result<Stopping, Vec<Refusal>> {
    // The whole record is looked for first, so that a document that carries
    // none of it is told that rather than being told about its first missing
    // member. A method that never recorded where it stopped and one that
    // mistyped a field name are different mistakes.
    if document.get("stopping").is_none() {
        return Err(vec![Refusal::NoStoppingRecord]);
    }
    let rule = field(document, "stopping", "rule")?;
    let rule = rule_of(&rule)?;
    let stopped_at = number(
        &field(document, "stopping", "stopped-at")?,
        "stopping.stopped-at",
    )?;
    let merit = number(&field(document, "stopping", "merit")?, "stopping.merit")?;
    let why = string(&field(document, "stopping", "why")?, "stopping.why")?;
    Ok(Stopping {
        rule,
        stopped_at,
        merit,
        why,
    })
}

fn rule_of(rule: &Json) -> Result<Rule, Vec<Refusal>> {
    let form = string(
        rule.get("form").ok_or_else(|| {
            vec![Refusal::Field {
                path: "stopping.rule.form".to_string(),
                wanted: "present".to_string(),
            }]
        })?,
        "stopping.rule.form",
    )?;
    let member = |name: &str| -> Result<f64, Vec<Refusal>> {
        let path = format!("stopping.rule.{name}");
        let value = rule.get(name).ok_or_else(|| {
            vec![Refusal::Field {
                path: path.clone(),
                wanted: "present".to_string(),
            }]
        })?;
        number(value, &path)
    };
    match form.as_str() {
        "fixed-count" => Ok(Rule::FixedCount {
            iterations: member("iterations")?,
        }),
        "relative-change" => Ok(Rule::RelativeChange {
            threshold: member("threshold")?,
            window: member("window")?,
        }),
        "wall-clock" => Ok(Rule::WallClock {
            seconds: member("seconds")?,
        }),
        _ => Err(vec![Refusal::Field {
            path: "stopping.rule.form".to_string(),
            wanted: "\"fixed-count\", \"relative-change\" or \"wall-clock\"".to_string(),
        }]),
    }
}

fn retrieved_of(document: &Json) -> Result<Retrieved, Vec<Refusal>> {
    let quantity = string(
        &field(document, "retrieved", "quantity")?,
        "retrieved.quantity",
    )?;
    let object = field(document, "retrieved", "")?;
    match quantity.as_str() {
        "field" => Ok(Retrieved::Field {
            time: axis_of(&object, "retrieved", "time")?,
            real: column(&object, "retrieved", "real")?,
            imaginary: column(&object, "retrieved", "imaginary")?,
        }),
        "spectrum" => Ok(Retrieved::Spectrum {
            energy: axis_of(&object, "retrieved", "energy")?,
            amplitude: column(&object, "retrieved", "amplitude")?,
            phase: column(&object, "retrieved", "phase")?,
        }),
        _ => Err(vec![Refusal::Field {
            path: "retrieved.quantity".to_string(),
            wanted: "\"field\" or \"spectrum\"".to_string(),
        }]),
    }
}

fn streaking_of(document: &Json) -> Result<Option<Streaking>, Vec<Refusal>> {
    let Some(object) = document.get("streaking").cloned() else {
        return Ok(None);
    };
    Ok(Some(Streaking {
        time: axis_of(&object, "streaking", "time")?,
        unit: string(
            object.get("unit").ok_or_else(|| {
                vec![Refusal::Field {
                    path: "streaking.unit".to_string(),
                    wanted: "present".to_string(),
                }]
            })?,
            "streaking.unit",
        )?,
        values: column(&object, "streaking", "values")?,
    }))
}

fn axis_of(object: &Json, owner: &str, name: &str) -> Result<Axis, Vec<Refusal>> {
    let grid = object.get(name).ok_or_else(|| {
        vec![Refusal::Field {
            path: format!("{owner}.{name}"),
            wanted: "present".to_string(),
        }]
    })?;
    let unit = string(
        grid.get("unit").ok_or_else(|| {
            vec![Refusal::Field {
                path: format!("{owner}.{name}.unit"),
                wanted: "present".to_string(),
            }]
        })?,
        &format!("{owner}.{name}.unit"),
    )?;
    Ok(Axis {
        unit,
        values: numbers_of(grid, &format!("{owner}.{name}.values"), "values")?,
    })
}

fn column(object: &Json, owner: &str, name: &str) -> Result<Vec<f64>, Vec<Refusal>> {
    numbers_of(object, &format!("{owner}.{name}"), name)
}

fn numbers_of(object: &Json, path: &str, name: &str) -> Result<Vec<f64>, Vec<Refusal>> {
    let value = object.get(name).ok_or_else(|| {
        vec![Refusal::Field {
            path: path.to_string(),
            wanted: "present".to_string(),
        }]
    })?;
    let items = value.as_array().ok_or_else(|| {
        vec![Refusal::Field {
            path: path.to_string(),
            wanted: "an array".to_string(),
        }]
    })?;
    let mut values = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        values.push(number(item, &format!("{path}[{index}]"))?);
    }
    Ok(values)
}
