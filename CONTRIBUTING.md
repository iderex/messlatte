# Contributing

## Before anything else

Every change starts as an issue. Planning happens on the tracker first, and a
decision that shapes the architecture is written down there with its reasons
before the code that depends on it exists. An issue says what is wrong, what
the evidence is, and what done means. Where the evidence is a number, it
carries the command that produced it.

The pull request that lands the change names the issue, in its body and in
every commit subject. The hygiene gate reads both and fails a pull request that
names neither.

## Running the checks locally

Each command below is the one the gate of the same name runs, so a local run
gives the same verdict rather than a similar one.

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo build --workspace --all-targets --locked
    cargo test --workspace --locked

Two more run on the server and are worth running before a change that touches
the dependency set or the toolchain floor:

    cargo deny --all-features check
    cargo +1.85.0 build --workspace --all-targets --locked

The floor in the second is the oldest toolchain this workspace is built with.
It is measured rather than chosen, and the measurement and its reasoning are in
the workspace manifest rather than repeated here, so the number moves in one
place.

`cargo test --workspace` runs the default suite only. Three suites sit outside
it, each opt-in through an environment variable, and each named for what it
needs rather than for a category. What they are and what running one costs is
printed by the run itself:

    cargo test -p messlatte-suites --test inventory --locked

That report is the authority for which suites exist. It also states, in words,
that a suite marked NOT RUN did not pass. A green default run is not a run that
covered everything.

## Signing off

Every commit carries a `Signed-off-by:` trailer matching its own author:

    git commit -s

The trailer is the author asserting the Developer Certificate of Origin,
version 1.1. The text asserted is in [DCO](DCO) at the root of this repository,
verbatim, and it is what the sign-off gate checks every non-merge commit in a
pull request for. One commit without a matching trailer reds the check.

The license a sign-off names is the one on the file being changed. For code
that is AGPL-3.0, in [LICENSE](LICENSE); for a document it is CC-BY-4.0. Which
file is which is in [NOTICE.md](NOTICE.md).

## What a submission file is

A method is scored through files. It reads a case directory and writes a
submission file, and the scorer reads that file and nothing else. A method is
not required to be code inside this workspace, and nothing in the scoring path
may assume that it is.

Two things follow from that, and both are the reason the contract exists rather
than a consequence of it.

An author whose method is in Python, in MATLAB, in a notebook or in a trained
network runs it on the same cases and submits the same file the reference
implementations submit. A benchmark that can only score code it contains scores
its own reimplementations, and a reimplementation that scores badly cannot be
told apart from a method that scores badly.

And the scorer sees only what the submission carries. A scorer handed a data
structure by the process that produced it can read the truth by accident. The
truth file sits in the same case directory as the trace, and no method may open
it.

The reference implementations are scored through the same reader as an outside
submission. An in-process interface exists for them, because a subprocess per
start would dominate the cheap methods, and it sits above the file contract
rather than beside it.

What a submission carries, field by field, and the validator that refuses a
malformed one, are not settled. That is issue #38 and it is open. Nothing in
this document fixes the fields, and a file written today against a format that
does not exist yet is a file nobody promised to read. The reason for the
boundary is settled; the bytes are not.

The one trace format that is settled is documented in
[docs/format/trace.md](docs/format/trace.md), which is the authority for it.

## What to have run before pushing

The four commands under "Running the checks locally", at the commit being
pushed, plus the two server-side ones where the change touches what they read.

Beyond running them, three rules decide whether a change is finished.

**An asserted fact carries the command that produced it**, run at the commit
being pushed and against the reference a reader will have rather than against a
working copy. Where a claim cannot be backed by a command, it is written as a
claim. Reading a working checkout and reporting it as the mainline is the
canonical form of getting this wrong.

**No guard ships without proof that it bites, for the reason it names.** Delete
the guard, watch the suite go red, put the guard back, and say in the pull
request body which cases went red. A near-miss that could not have failed proves
less than one that nearly did, so the mutation worth writing is the
one-character mistake somebody will actually make.

**A negative disclosure stays negative.** Where something was not run, not
measured or not evaluated, the sentence saying so survives every edit. Rewriting
an admission that a check did not run into a statement that it passed is worse
than what it removed.

Those three are prose here. No check in this repository refuses a claim made
without its command, an unproven guard, or a disclosure that grew a positive
tense, and none is going to catch them for a reader.

## The voice

English in everything tracked. State residual risk rather than drawing a
conclusion for the reader, cite the command, and keep measured, assumed and not
evaluated as three different words for three different states. Where a run
covered less than the whole set, say which part it did not cover, in the same
place a reader would otherwise read it as complete.

No tool name, generated-by marker or attribution to anything that is not a
person belongs in a tracked file, a commit message or an issue.

## Size

A change that will not fit under about four hundred changed lines without
losing quality is usually an issue whose scope was planned wrong, and the first
response is to re-plan that issue into sub-issues rather than to carve up a
finished diff. Two pull requests split out of one change only make sense
together, which is the cap met and its purpose defeated.

Some large changes are one readable thing, because a single property holds
across every changed byte and a reader checks the property instead of the diff.
Where that is the case, the pull request body names the property. The hygiene
gate marks size in its advisory tier and never fails on it, so this is a
judgement a reader makes and not one a machine makes.

## Reporting a vulnerability

Not here. [SECURITY.md](SECURITY.md) says what counts as one in a benchmark with
no network surface, and where to send it privately.
