//! Which suites exist, how one is asked for, and what the run says about the
//! ones that did not run.
//!
//! Each of the three is named for what it needs rather than called integration,
//! because a reader who sees a name like that learns what the suite requires
//! instead of learning that somebody had a category left over.
//!
//! Every one of them is opt-in, and the default run says so in its own output.
//! A suite that is skipped in silence is how a project comes to believe an
//! untested path works, which is the failure this whole split exists against.

use std::env;
use std::fmt;

/// A suite outside the default one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Suite {
    /// Runs that take minutes rather than seconds: the ensemble and the sweeps.
    Minutes,
    /// Anything that needs a machine with an accelerator (#18).
    Accelerator,
    /// Anything that reads a file from outside this repository.
    OutsideFile,
}

/// Every suite there is. A suite absent from here is a suite the report never
/// mentions, so adding one is adding an entry here and not only a constant.
pub const ALL: [Suite; 3] = [Suite::Minutes, Suite::Accelerator, Suite::OutsideFile];

impl Suite {
    /// The name the report prints.
    pub fn name(self) -> &'static str {
        match self {
            Suite::Minutes => "minutes",
            Suite::Accelerator => "accelerator",
            Suite::OutsideFile => "outside-file",
        }
    }

    /// The environment variable that asks for it.
    pub fn variable(self) -> &'static str {
        match self {
            Suite::Minutes => "MESSLATTE_SUITE_MINUTES",
            Suite::Accelerator => "MESSLATTE_SUITE_ACCELERATOR",
            Suite::OutsideFile => "MESSLATTE_SUITE_OUTSIDE_FILE",
        }
    }

    /// What running it would cost, in the terms somebody deciding whether to
    /// run it would want. A cost nobody states is a cost nobody weighs.
    pub fn cost(self) -> &'static str {
        match self {
            Suite::Minutes => {
                "minutes of wall clock per case, on the same hardware the default suite runs on"
            }
            Suite::Accelerator => {
                "a machine carrying the accelerator, and the run is skipped with a printed reason where there is none"
            }
            Suite::OutsideFile => {
                "a file this repository does not track, which the case names and the operator supplies"
            }
        }
    }
}

impl fmt::Display for Suite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// What happened to one suite in one run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enrolment {
    /// It was asked for, so its cases ran.
    Ran,
    /// Nobody asked for it.
    NotAsked,
    /// Somebody asked for it not to run.
    Declined,
}

impl Enrolment {
    /// The reason the report gives.
    pub fn reason(self, suite: Suite) -> String {
        match self {
            Enrolment::Ran => format!("{} is set to 1", suite.variable()),
            Enrolment::NotAsked => format!("{} is unset", suite.variable()),
            Enrolment::Declined => format!("{} is set to 0", suite.variable()),
        }
    }
}

/// What a value of the environment variable means.
///
/// Two values are understood and everything else is an error. A value this
/// function did not recognise must never read as off: somebody who wrote
/// `true`, or `1 `, or a spelling mistake, asked for a suite, and a run that
/// treated that as a decline would report a suite as not asked for while the
/// person who asked watched it pass. An unset variable is the only silence that
/// means anything here, and it means nobody asked.
pub fn enrol(value: Option<&str>) -> Result<Enrolment, String> {
    match value {
        None => Ok(Enrolment::NotAsked),
        Some("1") => Ok(Enrolment::Ran),
        Some("0") => Ok(Enrolment::Declined),
        Some(other) => Err(format!(
            "the value {other:?} is neither 1 nor 0, so what was asked for cannot be read. \
             Set the variable to 1 to run the suite or to 0 to decline it, or unset it."
        )),
    }
}

/// What the environment says about one suite.
///
/// A variable holding bytes that are not text is an error rather than an
/// absence, for the same reason as above: something was set and this cannot say
/// what.
pub fn from_env(suite: Suite) -> Result<Enrolment, String> {
    match env::var(suite.variable()) {
        Ok(value) => enrol(Some(&value)),
        Err(env::VarError::NotPresent) => Ok(Enrolment::NotAsked),
        Err(env::VarError::NotUnicode(_)) => Err(format!(
            "{} holds bytes that are not text, so what was asked for cannot be read",
            suite.variable()
        )),
    }
}

/// The lines the default run prints about the suites it did and did not run.
///
/// It reports the default suite as having run, because the caller is a case
/// inside it. Everything else is reported with its reason and its cost, and the
/// closing line says in words that what did not run did not pass.
///
/// The default line names no fixture. There is no case in this tree for the
/// default suite to run on, and the line said there was one, which is a claim of
/// coverage in the one block a reader consults to find out how much a run
/// covered. #32 is what puts a case here, and the line moves when it does.
pub fn report(enrolments: &[(Suite, Enrolment)]) -> String {
    let mut out = String::new();
    out.push_str("suites in this run\n");
    out.push_str(
        "  default        RAN      every unit and property case, on the committed fixture\n",
    );

    for (suite, enrolment) in enrolments {
        let verdict = match enrolment {
            Enrolment::Ran => "RAN    ",
            Enrolment::NotAsked | Enrolment::Declined => "NOT RUN",
        };
        out.push_str(&format!(
            "  {:<14} {}  {}\n                          cost: {}\n",
            suite.name(),
            verdict,
            enrolment.reason(*suite),
            suite.cost()
        ));
    }

    out.push_str(
        "\nA suite marked NOT RUN did not pass. It was not run, and this run\n\
         cannot be read as one that covered everything and found nothing.\n",
    );
    out
}

/// The report for the suites as this process's environment leaves them.
pub fn report_from_env() -> Result<String, String> {
    let mut enrolments = Vec::new();
    for suite in ALL {
        enrolments.push((suite, from_env(suite)?));
    }
    Ok(report(&enrolments))
}
