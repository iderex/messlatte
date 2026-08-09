# The streaking operator: the expression, its symbols and what it leaves out

The authority for the forward model this repository generates traces with.
`crates/generator/src/operator.rs` is one implementation of it, and the printed
form below is held in that file and reproduced here verbatim, with a case
comparing the two, so a model that moves in one and not in the other reddens.

Everything is in atomic units. Why the numerics hold atomic units and every file
holds SI is #13. The streaking field, the phase and their signs are
`docs/format/streaking-field.md`. The target's transition amplitude is
`docs/format/dipole.md`.

Issue #42 is where these choices are argued.

## The form the operator prints

Printed by

    messlatte model

and this is what it prints.

```
The streaking operator, in atomic units.

    S(p, tau) = | a(p, tau) |^2

    a(p, tau) = integral over s of
                    d_amp(  e(s + tau) )
                  * exp( i * d_phase( e(s + tau) ) )
                  * E(s)
                  * exp( -i * phi(p, s + tau) )
                  * exp( i * ( p^2 / 2 + Ip ) * s )

    e(t)      = ( p + A(t) )^2 / 2

Symbols.

    p         the final momentum along the polarisation
    tau       the delay of the pulse relative to the streaking envelope's peak
    s         time on the pulse's own grid
    t         time referred to the streaking envelope's peak, t = s + tau
    E(s)      the complex extreme-ultraviolet field on that grid
    A(t)      the streaking field's vector potential, docs/format/streaking-field.md
    phi(p, t) the phase accumulated in the streaking field from t onwards, the
              same document. It enters with a minus beside a drift term that
              enters with a plus, which is what makes the exponent stationary
              where the instantaneous kinetic energy is the photon energy less
              the ionisation potential. Written with a plus, the trace streaks
              the wrong way and looks right.
    e(t)      the kinetic energy of the shifted momentum
    d_amp     the target's transition amplitude at that energy, docs/format/dipole.md
    d_phase   its phase at that energy, in this repository's convention
    Ip        the ionisation potential of the target

What is left out, and each is a case this board cannot currently speak about.

    no depletion of the ground state
    no space charge
    no propagation of either field through the target
    no vector character beyond the single declared polarisation direction
    one active electron
    one final state per target, unless a case declares more

Two factors of the amplitude are absent for arithmetic rather than physical
reasons, and both are stated because a reader comparing this with a publication
will look for them.

    the prefactor -i, which is a constant of modulus one
    the factor exp( i * ( p^2 / 2 + Ip ) * tau ), which does not depend on s

Neither survives the squared modulus, and the second is a large phase whose
cosine a double resolves badly, so it is dropped rather than computed and
discarded.

The integral is composite Simpson over the pulse's own samples, which is why the
pulse carries an odd number of them. The phase is integrated by the quadrature
`crates/generator/src/field.rs` describes.
```

## What each choice in it is

**The integration variable is the pulse's own time.** The pulse is a complex
field on a uniform grid and the integral runs over its samples, with the
streaking field and the phase evaluated at the shifted time `s + tau`. So the
pulse is never interpolated, and a delay is not restricted to a whole number of
samples. Written the other way round, with the integral on a fixed time grid and
the pulse shifted onto it, every delay that is not a multiple of the step would
carry an interpolation error that varies along the delay axis, which is
indistinguishable in a trace from a real effect of the delay.

**The accumulated phase enters with a minus.** `docs/format/streaking-field.md`
defines that phase as an integral and leaves the sign it enters with to this
document, which is the right place for it: the integral is a property of the
field and the sign is a property of the amplitude. The minus is what makes the
derivative of the exponent equal to `( p + A(t) )^2 / 2 + Ip`, so the exponent is
stationary where the instantaneous kinetic energy is the photon energy less the
ionisation potential, which gives an electron born at `t` with speed `v0` a final
momentum of `v0 - A(t)`. Written with a plus, the derivative is
`( p - A(t) )^2 / 2 + Ip - A(t)^2`, which is stationary nowhere a streaking
picture describes, and the trace streaks the other way while still looking like a
streaking trace. That is #123, and it is how this operator was first written.

**The momentum enters the phase as itself.** It is not expanded about a central
momentum. That approximation is the one whose cost this repository exists to
measure, so the generator does not make it. A method that makes it declares it,
which is #5 and #7.

**The dipole is evaluated at the shifted momentum.** `e(t)` is the kinetic
energy of `p + A(t)` and not of `p`. A model evaluating the transition amplitude
at the final momentum instead would remove exactly the effect #6 exists to keep,
and the trace would look right.

**The trace is a squared modulus and carries no absolute scale.** It is written
as normalised counts with the sentence saying what it was divided by, which is
the largest cell. A case declaring an absolute photon number is #48 and does not
exist.

## What the omissions mean

Each line under "what is left out" is a term somebody could add later and a case
this board cannot currently speak about. Printing them beside the expression is
what stops a reader taking the absence of a term for a claim that the term is
negligible.

The two arithmetic absences are different in kind and are separated for that
reason. They are factors of modulus one that are constant over the integration
variable, so the squared modulus removes them exactly rather than approximately.
The second is dropped rather than computed because `( p^2 / 2 + Ip ) * tau` is a
large number at any useful delay and the cosine of a large number is where a
double loses its digits.

## What this document does not fix

The grids the operator is evaluated on, and the resolution conditions a
generator owes before it writes a trace, which is #47. The perturbations applied
to a trace afterwards, which is milestone 05. The limits this model has to
reproduce and the tolerance each is checked to, which is #46 and is open: the
cases beside this operator today check the zero-field limit and that the phase
reaches the trace, and they are not those four limits.
