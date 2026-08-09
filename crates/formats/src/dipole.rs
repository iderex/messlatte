//! The target's dipole transition matrix element, per target, as data (#44).
//!
//! Energy, amplitude and phase, with a source per table and a version. The
//! document beside this one is `docs/format/dipole.md` and it is the authority
//! for the conventions; what is here is one implementation of it.
//!
//! Three things about this format decide whether two people mean the same
//! target by the same numbers, and each is a choice rather than a consequence.
//!
//! The energy axis is the photoelectron's kinetic energy and not the photon
//! energy, because the forward model evaluates the matrix element at the
//! momentum the electron leaves with. The ionisation potential that separates
//! the two is the truth of a case, #36, and is not in this file.
//!
//! The phase carries the convention it was written in, and this converts it
//! into the one convention once, at load. A table in memory is always in this
//! repository's convention, whatever the file said. See [`Convention`].
//!
//! The energies stay in the unit the file states them in, and the energy a
//! caller asks about is converted into that unit once per lookup. That is the
//! one place this format departs from #13's "converted once at the edge", and
//! the reason is that a table is a file somebody else wrote: converting the
//! column into atomic units and back changes the last bit of a value that has
//! made the trip, so a writer holding the column in atomic units would emit a
//! table that is not the one it read. What the column measures never leaves
//! this module either way, because an amplitude and a phase carry no unit, and
//! linear interpolation gives the same answer on either axis.
//!
//! Outside the tabulated range this refuses rather than extrapolating. A
//! tabulated amplitude stops where the calculation stopped and not where the
//! target does, so a value taken beyond the last sample is a statement about
//! physics that nobody made.
//!
//! What is not here. The path a table sits at is a name in [`FLAT`] and a field
//! of the case declaration, #37, and this module reads and writes bytes rather
//! than resolving a path. Which real targets may be used and whose tables they
//! come from is entry 6 of #34 and is not this module's to answer.

use core::fmt;

use messlatte_units::Energy;

use crate::json::{self, Json, Object};

/// What the document calls this format.
pub const FORMAT: &str = "messlatte-dipole";

/// The version this module writes.
pub const VERSION: Version = Version { major: 1, minor: 0 };

/// The tracked path of the one table that ships.
///
/// A name rather than a route. Nothing here opens it, and what reads it is a
/// case declaration naming it, which is #37.
pub const FLAT: &str = "data/dipole/flat.json";

/// A format version.
///
/// A major version this reader does not know is refused rather than guessed at,
/// and a higher minor version is read with the fields this reader does not
/// recognise ignored. That rule is #41.
///
/// This is the second five-line version type in this crate, beside the trace
/// header's. Whether the two formats share one is #41's to settle, and settling
/// it means editing the trace module, so it is named here rather than done in
/// passing.
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

/// How a table's phase enters the complex matrix element.
///
/// Both spellings are in print, the tabulated numbers look the same in both,
/// and their meanings are opposite. A sign error here looks exactly like a
/// chirp in the pulse: it moves no amplitude, it reddens nothing that reads a
/// trace, and it reverses the direction of the chirp every reconstruction
/// reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Convention {
    /// `d = |d| exp(+i phase)`, which is this repository's own.
    Positive,
    /// `d = |d| exp(-i phase)`.
    Negative,
}

impl Convention {
    /// The convention every table is converted into at load.
    ///
    /// It is the sign that makes the dipole phase enter the photoelectron
    /// amplitude the same way the phase accumulated in the streaking field
    /// does, which is #43 and `docs/format/streaking-field.md`.
    pub const OURS: Convention = Convention::Positive;

    /// The spellings a file may carry.
    pub const ADMITTED: &'static [&'static str] = &["exp(+i phase)", "exp(-i phase)"];

    /// How the file spells it.
    #[must_use]
    pub const fn spelling(self) -> &'static str {
        match self {
            Convention::Positive => "exp(+i phase)",
            Convention::Negative => "exp(-i phase)",
        }
    }

    /// The convention a spelling names, or nothing.
    #[must_use]
    pub fn from_spelling(text: &str) -> Option<Convention> {
        match text {
            "exp(+i phase)" => Some(Convention::Positive),
            "exp(-i phase)" => Some(Convention::Negative),
            _ => None,
        }
    }

    /// What a phase written in this convention is multiplied by to reach
    /// [`Convention::OURS`].
    fn factor(self) -> f64 {
        match self {
            Convention::Positive => 1.0,
            Convention::Negative => -1.0,
        }
    }
}

/// One target's transition amplitude over energy.
///
/// The phase is in this repository's convention whatever the file it was read
/// from declared, and that conversion happens once, in [`Table::from_bytes`].
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    /// What the numbers are the matrix element of.
    pub target: String,
    /// Where they came from, precisely enough to look up.
    pub source: String,
    /// The unit the file states its energies in, and the unit a written file
    /// carries them in.
    pub unit: String,
    /// The tabulated energies in [`Table::unit`], strictly increasing.
    pub energy: Vec<f64>,
    /// The modulus, never negative.
    pub amplitude: Vec<f64>,
    /// What the amplitude is relative to.
    pub normalisation: String,
    /// The phase in radians, in [`Convention::OURS`].
    pub phase: Vec<f64>,
}

/// The matrix element at one energy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dipole {
    pub amplitude: f64,
    /// In radians, in [`Convention::OURS`].
    pub phase: f64,
}

/// Why a table could not answer for an energy.
#[derive(Debug, Clone, PartialEq)]
pub enum Lookup {
    /// The table is one this crate would refuse to read: fewer than two
    /// samples, columns of different lengths, or a unit the conversion layer
    /// does not know.
    ///
    /// A table that came through [`Table::from_bytes`] cannot be in this state.
    /// The variant exists because the fields above are public and a table can
    /// be built without going through the reader.
    Unusable,
    /// The energy is outside the tabulated range, in the table's own unit.
    OutsideRange {
        asked: f64,
        lowest: f64,
        highest: f64,
        unit: String,
    },
}

impl fmt::Display for Lookup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Lookup::Unusable => write!(
                f,
                "the table has no interval to interpolate in, so it can answer for no energy \
                 at all"
            ),
            Lookup::OutsideRange {
                asked,
                lowest,
                highest,
                unit,
            } => write!(
                f,
                "{asked} {unit} is outside the tabulated range {lowest} to {highest} {unit}. A \
                 tabulated amplitude stops where the calculation stopped and not where the \
                 target does, so this refuses rather than extrapolating"
            ),
        }
    }
}

/// One reason a table is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The document could not be read at all.
    Unreadable { detail: String },
    /// The document names a different format.
    NotADipoleTable { found: String },
    /// A major version this reader does not know.
    UnknownMajorVersion { found: Version },
    /// A field the format requires is absent, or is of the wrong shape.
    Field { path: String, wanted: String },
    /// A field that carries nothing where it has to carry a sentence.
    Blank { field: String },
    /// An energy unit outside the set the conversion layer admits.
    UnknownUnit { unit: String, admitted: String },
    /// A phase convention this loader cannot convert.
    UnknownConvention { found: String, admitted: String },
    /// Fewer samples than an interval needs.
    TooFewSamples { found: usize },
    /// The three columns disagree about how many samples there are.
    LengthDisagrees {
        energy: usize,
        amplitude: usize,
        phase: usize,
    },
    /// A column carrying a value that is not a number.
    ColumnNotFinite { column: String, index: usize },
    /// An energy axis that does not strictly increase.
    EnergyNotIncreasing { index: usize },
    /// An amplitude below zero, which is a sign written in the wrong column.
    AmplitudeNegative { index: usize },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::Unreadable { detail } => {
                write!(f, "the table could not be read: {detail}")
            }
            Refusal::NotADipoleTable { found } => write!(
                f,
                "the document declares the format {found:?} and a dipole table declares \
                 {FORMAT:?}"
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
                "the document's {field} carries nothing. A table whose {field} was never filled \
                 in is a table nobody can check"
            ),
            Refusal::UnknownUnit { unit, admitted } => write!(
                f,
                "the energies are in {unit:?}, and a file states an energy in {admitted}"
            ),
            Refusal::UnknownConvention { found, admitted } => write!(
                f,
                "the phase is written in {found:?}, and this converts {admitted}. The two \
                 spellings carry the same numbers and opposite meanings, so this refuses a \
                 third rather than assuming which of them it is"
            ),
            Refusal::TooFewSamples { found } => write!(
                f,
                "the table carries {found} sample(s) and interpolation needs an interval, so a \
                 table this short can answer for no energy but its own"
            ),
            Refusal::LengthDisagrees {
                energy,
                amplitude,
                phase,
            } => write!(
                f,
                "the table carries {energy} energies, {amplitude} amplitudes and {phase} \
                 phases, and a sample is all three"
            ),
            Refusal::ColumnNotFinite { column, index } => write!(
                f,
                "the {column} column carries a value at sample {index} that is not finite"
            ),
            Refusal::EnergyNotIncreasing { index } => write!(
                f,
                "the energy axis does not increase at sample {index}. Two samples at one energy \
                 are two values of one thing with nothing to say which is meant"
            ),
            Refusal::AmplitudeNegative { index } => write!(
                f,
                "the amplitude at sample {index} is below zero. An amplitude is a modulus, and a \
                 real matrix element that changes sign carries that sign as a phase of pi"
            ),
        }
    }
}

impl Table {
    /// The matrix element at an energy, interpolated linearly.
    ///
    /// Written as a weighted sum of the two bracketing samples rather than as
    /// an offset from the lower one, so an energy landing exactly on a sample
    /// returns that sample's own numbers and not a value one rounding away from
    /// them.
    ///
    /// # Errors
    ///
    /// The energy is outside the tabulated range, or the table has no interval
    /// to interpolate in. See [`Lookup`].
    pub fn at(&self, energy: Energy) -> Result<Dipole, Lookup> {
        let samples = self.energy.len();
        if samples < 2 || self.amplitude.len() != samples || self.phase.len() != samples {
            return Err(Lookup::Unusable);
        }
        // The one conversion, and it is the caller's number rather than the
        // table's. See the note at the top of this module for why the column
        // stays in the unit the file wrote it in.
        let Ok(wanted) = energy.in_si(&self.unit) else {
            return Err(Lookup::Unusable);
        };
        let lowest = self.energy[0];
        let highest = self.energy[samples - 1];
        if !(wanted >= lowest && wanted <= highest) {
            return Err(Lookup::OutsideRange {
                asked: wanted,
                lowest,
                highest,
                unit: self.unit.clone(),
            });
        }

        for index in 1..samples {
            let low = self.energy[index - 1];
            let high = self.energy[index];
            if wanted > high {
                continue;
            }
            let span = high - low;
            let upper = if span > 0.0 {
                (wanted - low) / span
            } else {
                0.0
            };
            let lower = 1.0 - upper;
            return Ok(Dipole {
                amplitude: self.amplitude[index - 1] * lower + self.amplitude[index] * upper,
                phase: self.phase[index - 1] * lower + self.phase[index] * upper,
            });
        }
        // Unreachable for an increasing axis, because the last sample is the
        // highest and the range was checked above. An axis that is not
        // increasing is refused by the reader, and one built by hand lands
        // here rather than in an arm that guesses.
        Err(Lookup::Unusable)
    }

    /// The document, in canonical bytes.
    ///
    /// The writer applies the same refusals as the reader, so a table this
    /// crate will not read is not one it can produce. It writes the phase in
    /// [`Convention::OURS`] and says so, because a writer that re-emitted the
    /// other spelling would be claiming to have preserved a meaning it has
    /// already changed.
    ///
    /// # Errors
    ///
    /// The table is one this crate would refuse to read. See [`Refusal`].
    pub fn to_bytes(&self) -> Result<Vec<u8>, Vec<Refusal>> {
        let refusals = self.refusals();
        if !refusals.is_empty() {
            return Err(refusals);
        }

        let mut amplitude = Object::new();
        amplitude.insert(
            "normalisation".to_string(),
            Json::String(self.normalisation.clone()),
        );
        amplitude.insert("values".to_string(), numbers(&self.amplitude));

        let mut energy = Object::new();
        energy.insert("unit".to_string(), Json::String(self.unit.clone()));
        energy.insert("values".to_string(), numbers(&self.energy));

        let mut format = Object::new();
        format.insert("name".to_string(), Json::String(FORMAT.to_string()));
        format.insert("version".to_string(), Json::String(VERSION.to_string()));

        let mut phase = Object::new();
        phase.insert(
            "convention".to_string(),
            Json::String(Convention::OURS.spelling().to_string()),
        );
        phase.insert("values".to_string(), numbers(&self.phase));

        let mut document = Object::new();
        document.insert("amplitude".to_string(), Json::Object(amplitude));
        document.insert("energy".to_string(), Json::Object(energy));
        document.insert("format".to_string(), Json::Object(format));
        document.insert("phase".to_string(), Json::Object(phase));
        document.insert("source".to_string(), Json::String(self.source.clone()));
        document.insert("target".to_string(), Json::String(self.target.clone()));

        Json::Object(document).to_bytes().map_err(|detail| {
            vec![Refusal::Unreadable {
                detail: format!("the document could not be written: {detail}"),
            }]
        })
    }

    /// A table from the bytes of one document.
    ///
    /// The phase is converted into [`Convention::OURS`] here and nowhere else.
    ///
    /// # Errors
    ///
    /// Everything wrong with the document. See [`Refusal`].
    pub fn from_bytes(bytes: &[u8]) -> Result<Table, Vec<Refusal>> {
        let document = json::parse(bytes).map_err(|detail| vec![Refusal::Unreadable { detail }])?;

        // The format and the version are read before anything else, because
        // every field below means what this version says it means and a
        // complaint about a field in a document from another format would send
        // a reader after the wrong thing.
        let name = string(&field(&document, "format", "name")?, "format.name")?;
        if name != FORMAT {
            return Err(vec![Refusal::NotADipoleTable { found: name }]);
        }
        let version = version_of(&document)?;
        if version.major != VERSION.major {
            return Err(vec![Refusal::UnknownMajorVersion { found: version }]);
        }

        let target = string(&field(&document, "target", "")?, "target")?;
        let source = string(&field(&document, "source", "")?, "source")?;
        let normalisation = string(
            &field(&document, "amplitude", "normalisation")?,
            "amplitude.normalisation",
        )?;
        let unit = string(&field(&document, "energy", "unit")?, "energy.unit")?;
        let stated = string(
            &field(&document, "phase", "convention")?,
            "phase.convention",
        )?;
        let Some(convention) = Convention::from_spelling(&stated) else {
            return Err(vec![Refusal::UnknownConvention {
                found: stated,
                admitted: Convention::ADMITTED.join(", "),
            }]);
        };

        let energy = column(&document, "energy")?;
        let amplitude = column(&document, "amplitude")?;
        let stated_phase = column(&document, "phase")?;

        // The one conversion this format owes on the way in, here and nowhere
        // else. Everything after this point is in one convention.
        let factor = convention.factor();
        let phase = stated_phase
            .into_iter()
            .map(|value| value * factor)
            .collect();

        let table = Table {
            target,
            source,
            unit,
            energy,
            amplitude,
            normalisation,
            phase,
        };
        let refusals = table.refusals();
        if refusals.is_empty() {
            Ok(table)
        } else {
            Err(refusals)
        }
    }

    /// Everything wrong with this table, in one pass.
    ///
    /// All of them rather than the first, because a table with a bad unit and a
    /// bad column is one file somebody has to fix once, and a reader reporting
    /// one refusal per run would make them fix it twice.
    fn refusals(&self) -> Vec<Refusal> {
        let mut found = Vec::new();
        for (name, text) in [
            ("target", &self.target),
            ("source", &self.source),
            ("amplitude.normalisation", &self.normalisation),
        ] {
            if text.trim().is_empty() {
                found.push(Refusal::Blank {
                    field: name.to_string(),
                });
            }
        }
        if !Energy::UNITS.contains(&self.unit.as_str()) {
            found.push(Refusal::UnknownUnit {
                unit: self.unit.clone(),
                admitted: Energy::UNITS.join(", "),
            });
        }

        let samples = self.energy.len();
        if self.amplitude.len() != samples || self.phase.len() != samples {
            found.push(Refusal::LengthDisagrees {
                energy: samples,
                amplitude: self.amplitude.len(),
                phase: self.phase.len(),
            });
        } else if samples < 2 {
            // Only where the columns agree. A one-sample table whose columns
            // also disagree has one fault a reader can act on and not two.
            found.push(Refusal::TooFewSamples { found: samples });
        }

        for (index, sample) in self.energy.iter().enumerate() {
            if !sample.is_finite() {
                found.push(Refusal::ColumnNotFinite {
                    column: "energy".to_string(),
                    index,
                });
            }
        }
        for (index, value) in self.amplitude.iter().enumerate() {
            if value.is_finite() {
                if *value < 0.0 {
                    found.push(Refusal::AmplitudeNegative { index });
                }
            } else {
                found.push(Refusal::ColumnNotFinite {
                    column: "amplitude".to_string(),
                    index,
                });
            }
        }
        for (index, value) in self.phase.iter().enumerate() {
            if !value.is_finite() {
                found.push(Refusal::ColumnNotFinite {
                    column: "phase".to_string(),
                    index,
                });
            }
        }

        for index in 1..self.energy.len() {
            // Written through the ordering rather than as a negated comparison
            // so that a value which compares to nothing is refused here too,
            // and not only by the finiteness check above.
            let increases = matches!(
                self.energy[index].partial_cmp(&self.energy[index - 1]),
                Some(core::cmp::Ordering::Greater)
            );
            if !increases {
                found.push(Refusal::EnergyNotIncreasing { index });
            }
        }
        found
    }
}

fn numbers(values: &[f64]) -> Json {
    Json::Array(values.iter().map(|value| Json::Number(*value)).collect())
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

/// One column of numbers, named by the object holding it.
fn column(document: &Json, name: &str) -> Result<Vec<f64>, Vec<Refusal>> {
    let values = field(document, name, "values")?;
    let items = values.as_array().ok_or_else(|| {
        vec![Refusal::Field {
            path: format!("{name}.values"),
            wanted: "an array".to_string(),
        }]
    })?;
    let mut samples = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        samples.push(item.as_number().ok_or_else(|| {
            vec![Refusal::Field {
                path: format!("{name}.values[{index}]"),
                wanted: "a number".to_string(),
            }]
        })?);
    }
    Ok(samples)
}
