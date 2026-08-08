//! The trace file: one array, one header, and what a reader may assume (#35).
//!
//! A trace is two documents. The values are a two-dimensional array of doubles
//! in an NPY file, read by `npy`. Everything needed to say what those values
//! are is a JSON header, written in the canonical order `json` fixes, so that
//! identical content is identical bytes and a hash over a trace means
//! something.
//!
//! The part worth reading before using one of these is what a reader may
//! assume, because every clause of it is a mistake somebody would otherwise
//! make quietly.
//!
//! Both axes are strictly increasing, and this refuses a file where either is
//! not. A repeated value is refused with them, because two cells at one axis
//! position are two measurements of one thing and nothing in the format says
//! which is meant.
//!
//! The delay axis is not necessarily uniform. Sampling completeness is one of
//! the axes of the study, so a gap in the delays is a case rather than a
//! defect, and a reader that computes a step from the first two entries and
//! uses it for the rest is wrong about every case with a gap in it. A method
//! that needs a uniform delay axis says so as a declared requirement and is
//! scored on the cases that meet it.
//!
//! Cells are counts, or a stated normalisation of counts, and never an
//! arbitrary scale. A normalised trace carries the sentence saying what it was
//! divided by, and a trace of raw counts carries no such sentence, so the two
//! cannot be confused by a reader looking at the numbers.
//!
//! Missing samples are absent from the axis rather than written as a value, so
//! there is no sentinel here to mistake for data. That is enforced from the
//! other end: a cell that is not finite is refused, in a reader and in a
//! writer, so a file cannot carry a missing sample at all.
//!
//! What is not here. The path a trace is written to, and what else sits beside
//! it in a case directory, is the case declaration and the case index, #37 and
//! #39. This module reads and writes bytes and names the two files by
//! convention only, in [`HEADER_FILE`] and [`ARRAY_FILE`].

use std::fmt;

use crate::json::{self, Json, Object};
use crate::npy::Array;

/// What the header calls this format. A document that does not carry it is not
/// a trace header, however much of one it looks like.
pub const FORMAT: &str = "messlatte-trace";

/// The version this module writes.
pub const VERSION: Version = Version { major: 1, minor: 0 };

/// The conventional name of the header inside a case directory.
pub const HEADER_FILE: &str = "trace.json";

/// The conventional name of the array beside it.
pub const ARRAY_FILE: &str = "trace.npy";

/// A format version.
///
/// The rule it exists for is #41 and is applied here in its narrow form: a
/// major version this reader does not know is refused rather than guessed at,
/// and a higher minor version is accepted with the fields this reader does not
/// recognise ignored. Ignored rather than preserved: a trace read from a later
/// minor version and written again by this writer is a version 1.0 trace and
/// says so, because a writer that re-emitted fields it does not understand
/// would be claiming to have kept their meaning.
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

/// Which quantity the electron axis measures.
///
/// Both are in use and the difference is not a presentation choice: a
/// spectrometer that reports momentum and one that reports energy disagree
/// about where the samples sit, and converting between them moves every sample.
/// The case says which it is and nothing infers it from the numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Electron {
    Momentum,
    Energy,
}

impl Electron {
    fn name(self) -> &'static str {
        match self {
            Electron::Momentum => "momentum",
            Electron::Energy => "energy",
        }
    }

    /// The units the format admits for it.
    ///
    /// The set is small and closed rather than open, so that the conversion
    /// layer #13 asks for has a finite thing to round-trip. Every entry is SI,
    /// with the electronvolt the one exception, which is the exception SI
    /// itself makes: it is on the table of units accepted for use with the SI.
    /// Atomic units are not admitted in a file, because they belong inside the
    /// numerics and a file is what an operator reads.
    pub fn units(self) -> &'static [&'static str] {
        match self {
            Electron::Momentum => &["kg m/s"],
            Electron::Energy => &["J", "eV"],
        }
    }
}

/// The units the delay axis admits, on the same reasoning.
pub const DELAY_UNITS: &[&str] = &["s", "fs", "as"];

/// One axis: what it is measured in and where its samples are.
#[derive(Debug, Clone, PartialEq)]
pub struct Axis {
    pub unit: String,
    pub values: Vec<f64>,
}

/// What a cell holds.
///
/// The two are separate variants rather than one string with a convention,
/// because the difference decides whether a photon count can be read out of the
/// file, and a reader that had to recognise the word "counts" inside a free
/// sentence would sometimes get it wrong.
#[derive(Debug, Clone, PartialEq)]
pub enum Cells {
    /// Counts, on no scale but their own.
    Counts,
    /// Counts divided by something, and the sentence saying what.
    Normalised { normalisation: String },
}

/// One trace.
#[derive(Debug, Clone, PartialEq)]
pub struct Trace {
    /// The case this trace belongs to. The identifier is the case directory's
    /// name and the join key of the index in #39.
    pub case: String,
    pub electron_quantity: Electron,
    pub electron: Axis,
    pub delay: Axis,
    pub cells: Cells,
    /// The values, with the electron axis along the rows and the delay axis
    /// along the columns.
    pub values: Array,
}

/// One reason a trace is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The header or the array could not be read at all.
    Unreadable { what: String, detail: String },
    /// The header names a different format.
    NotATraceHeader { found: String },
    /// A major version this reader does not know.
    UnknownMajorVersion { found: Version },
    /// A field the format requires is absent, or is of the wrong shape.
    Field { path: String, wanted: String },
    /// An axis with no samples on it.
    EmptyAxis { axis: String },
    /// An axis that does not strictly increase.
    AxisNotIncreasing { axis: String, index: usize },
    /// An axis carrying a value that is not a number.
    AxisNotFinite { axis: String, index: usize },
    /// A unit outside the set the format admits.
    UnknownUnit {
        axis: String,
        unit: String,
        admitted: String,
    },
    /// A normalisation stated for a trace of raw counts, or absent from a
    /// normalised one.
    Normalisation { detail: String },
    /// The array's shape and the axes disagree about how big the trace is.
    ShapeDisagrees {
        rows: usize,
        columns: usize,
        electron: usize,
        delay: usize,
    },
    /// A cell that is not a number, which is how a missing sample would be
    /// smuggled in.
    CellNotFinite { row: usize, column: usize },
    /// An identifier that is not one.
    EmptyCase,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::Unreadable { what, detail } => {
                write!(f, "the {what} could not be read: {detail}")
            }
            Refusal::NotATraceHeader { found } => write!(
                f,
                "the header declares the format {found:?} and a trace header declares {FORMAT:?}"
            ),
            Refusal::UnknownMajorVersion { found } => write!(
                f,
                "the header declares version {found} and this reads major version {}. A major \
                 version is a change of meaning, so this refuses it rather than guessing which \
                 fields still mean what they did",
                VERSION.major
            ),
            Refusal::Field { path, wanted } => {
                write!(f, "the header's {path} is not {wanted}")
            }
            Refusal::EmptyAxis { axis } => write!(
                f,
                "the {axis} axis carries no samples, so the trace has no cells to be about"
            ),
            Refusal::AxisNotIncreasing { axis, index } => write!(
                f,
                "the {axis} axis does not increase at sample {index}. Axes are strictly \
                 increasing, and two samples at one position are two measurements of one thing \
                 with nothing to say which is meant"
            ),
            Refusal::AxisNotFinite { axis, index } => write!(
                f,
                "the {axis} axis carries a value at sample {index} that is not finite"
            ),
            Refusal::UnknownUnit {
                axis,
                unit,
                admitted,
            } => write!(
                f,
                "the {axis} axis is in {unit:?}, and this format admits {admitted}"
            ),
            Refusal::Normalisation { detail } => write!(f, "{detail}"),
            Refusal::ShapeDisagrees {
                rows,
                columns,
                electron,
                delay,
            } => write!(
                f,
                "the array is {rows} by {columns} and the axes are {electron} electron samples \
                 by {delay} delay samples"
            ),
            Refusal::CellNotFinite { row, column } => write!(
                f,
                "the cell at row {row} and column {column} is not finite. A missing sample is \
                 absent from the axis rather than written as a value, so there is no sentinel \
                 in this format for a reader to mistake for data"
            ),
            Refusal::EmptyCase => {
                write!(f, "the header carries no case identifier")
            }
        }
    }
}

impl Trace {
    /// The two files, header first.
    ///
    /// The writer applies the same refusals as the reader. A writer that could
    /// emit a file its own reader rejects would put the check somewhere the
    /// mistake has already been committed, and the file would exist by then.
    pub fn to_bytes(&self) -> Result<(Vec<u8>, Vec<u8>), Vec<Refusal>> {
        let refusals = self.refusals();
        if !refusals.is_empty() {
            return Err(refusals);
        }

        let mut format = Object::new();
        format.insert("name".to_string(), Json::String(FORMAT.to_string()));
        format.insert("version".to_string(), Json::String(VERSION.to_string()));

        let mut electron = Object::new();
        electron.insert(
            "quantity".to_string(),
            Json::String(self.electron_quantity.name().to_string()),
        );
        electron.insert("unit".to_string(), Json::String(self.electron.unit.clone()));
        electron.insert("values".to_string(), numbers(&self.electron.values));

        let mut delay = Object::new();
        delay.insert("unit".to_string(), Json::String(self.delay.unit.clone()));
        delay.insert("values".to_string(), numbers(&self.delay.values));

        let mut cells = Object::new();
        match &self.cells {
            Cells::Counts => {
                cells.insert("quantity".to_string(), Json::String("counts".to_string()));
            }
            Cells::Normalised { normalisation } => {
                cells.insert(
                    "quantity".to_string(),
                    Json::String("normalised-counts".to_string()),
                );
                cells.insert(
                    "normalisation".to_string(),
                    Json::String(normalisation.clone()),
                );
            }
        }

        let mut header = Object::new();
        header.insert("case".to_string(), Json::String(self.case.clone()));
        header.insert("cells".to_string(), Json::Object(cells));
        header.insert("delay".to_string(), Json::Object(delay));
        header.insert("electron".to_string(), Json::Object(electron));
        header.insert("format".to_string(), Json::Object(format));

        let header = Json::Object(header).to_bytes().map_err(|detail| {
            vec![Refusal::Unreadable {
                what: "header".to_string(),
                detail,
            }]
        })?;
        let array = self.values.to_bytes().map_err(|detail| {
            vec![Refusal::Unreadable {
                what: "array".to_string(),
                detail,
            }]
        })?;
        Ok((header, array))
    }

    /// A trace from the two files.
    pub fn from_bytes(header: &[u8], array: &[u8]) -> Result<Trace, Vec<Refusal>> {
        let document = json::parse(header).map_err(|detail| {
            vec![Refusal::Unreadable {
                what: "header".to_string(),
                detail,
            }]
        })?;
        let values = Array::from_bytes(array).map_err(|detail| {
            vec![Refusal::Unreadable {
                what: "array".to_string(),
                detail,
            }]
        })?;

        // The format and the version are read before anything else, because
        // every field below means what this version says it means and a
        // complaint about a field in a document from another format would send
        // a reader after the wrong thing.
        let name = field(&document, "format", "name")?;
        let name = string(&name, "format.name")?;
        if name != FORMAT {
            return Err(vec![Refusal::NotATraceHeader { found: name }]);
        }
        let version = version_of(&document)?;
        if version.major != VERSION.major {
            return Err(vec![Refusal::UnknownMajorVersion { found: version }]);
        }

        let case = string(&field(&document, "case", "")?, "case")?;
        let quantity = string(
            &field(&document, "electron", "quantity")?,
            "electron.quantity",
        )?;
        let electron_quantity = match quantity.as_str() {
            "momentum" => Electron::Momentum,
            "energy" => Electron::Energy,
            _ => {
                return Err(vec![Refusal::Field {
                    path: "electron.quantity".to_string(),
                    wanted: "\"momentum\" or \"energy\"".to_string(),
                }])
            }
        };
        let electron = axis(&document, "electron")?;
        let delay = axis(&document, "delay")?;
        let cells = cells_of(&document)?;

        let trace = Trace {
            case,
            electron_quantity,
            electron,
            delay,
            cells,
            values,
        };
        let refusals = trace.refusals();
        if refusals.is_empty() {
            Ok(trace)
        } else {
            Err(refusals)
        }
    }

    /// Everything wrong with this trace, in one pass.
    ///
    /// All of them rather than the first, because a trace with a bad axis and a
    /// bad shape is one file somebody has to fix once, and a reader that
    /// reported one refusal per run would make them fix it twice.
    fn refusals(&self) -> Vec<Refusal> {
        let mut found = Vec::new();
        if self.case.trim().is_empty() {
            found.push(Refusal::EmptyCase);
        }
        check_axis(
            "electron",
            &self.electron,
            self.electron_quantity.units(),
            &mut found,
        );
        check_axis("delay", &self.delay, DELAY_UNITS, &mut found);

        if let Cells::Normalised { normalisation } = &self.cells {
            if normalisation.trim().is_empty() {
                found.push(Refusal::Normalisation {
                    detail: "the cells are normalised counts and the header does not say what \
                             they were divided by. A normalisation nobody stated is an \
                             arbitrary scale, which this format does not carry"
                        .to_string(),
                });
            }
        }

        if self.values.rows != self.electron.values.len()
            || self.values.columns != self.delay.values.len()
        {
            found.push(Refusal::ShapeDisagrees {
                rows: self.values.rows,
                columns: self.values.columns,
                electron: self.electron.values.len(),
                delay: self.delay.values.len(),
            });
        }

        for (index, value) in self.values.values.iter().enumerate() {
            if !value.is_finite() {
                let columns = self.values.columns.max(1);
                found.push(Refusal::CellNotFinite {
                    row: index / columns,
                    column: index % columns,
                });
            }
        }
        found
    }
}

fn check_axis(name: &str, axis: &Axis, admitted: &[&str], found: &mut Vec<Refusal>) {
    if !admitted.contains(&axis.unit.as_str()) {
        found.push(Refusal::UnknownUnit {
            axis: name.to_string(),
            unit: axis.unit.clone(),
            admitted: admitted.join(", "),
        });
    }
    if axis.values.is_empty() {
        found.push(Refusal::EmptyAxis {
            axis: name.to_string(),
        });
        return;
    }
    for (index, value) in axis.values.iter().enumerate() {
        if !value.is_finite() {
            found.push(Refusal::AxisNotFinite {
                axis: name.to_string(),
                index,
            });
        }
    }
    for index in 1..axis.values.len() {
        // Written through the ordering rather than as a negated comparison so
        // that a value which compares to nothing is refused here too, and not
        // only by the finiteness check above.
        let increases = matches!(
            axis.values[index].partial_cmp(&axis.values[index - 1]),
            Some(std::cmp::Ordering::Greater)
        );
        if !increases {
            found.push(Refusal::AxisNotIncreasing {
                axis: name.to_string(),
                index,
            });
        }
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

fn axis(document: &Json, name: &str) -> Result<Axis, Vec<Refusal>> {
    let unit = string(&field(document, name, "unit")?, &format!("{name}.unit"))?;
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
    Ok(Axis {
        unit,
        values: samples,
    })
}

fn cells_of(document: &Json) -> Result<Cells, Vec<Refusal>> {
    let quantity = string(&field(document, "cells", "quantity")?, "cells.quantity")?;
    let stated = document
        .get("cells")
        .and_then(|cells| cells.get("normalisation"))
        .map(|value| string(value, "cells.normalisation"))
        .transpose()?;
    match (quantity.as_str(), stated) {
        ("counts", None) => Ok(Cells::Counts),
        ("counts", Some(_)) => Err(vec![Refusal::Normalisation {
            detail: "the cells are counts and the header also states a normalisation. One of \
                     the two is wrong, and this format does not choose which"
                .to_string(),
        }]),
        ("normalised-counts", Some(normalisation)) => Ok(Cells::Normalised { normalisation }),
        ("normalised-counts", None) => Err(vec![Refusal::Normalisation {
            detail: "the cells are normalised counts and the header does not say what they \
                     were divided by. A normalisation nobody stated is an arbitrary scale, \
                     which this format does not carry"
                .to_string(),
        }]),
        _ => Err(vec![Refusal::Field {
            path: "cells.quantity".to_string(),
            wanted: "\"counts\" or \"normalised-counts\"".to_string(),
        }]),
    }
}
