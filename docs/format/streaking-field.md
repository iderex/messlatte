# The streaking field: the vector potential, the phase, and the signs

The authority for the conventions the generator uses for the streaking field
and for the phase a photoelectron accumulates in it. The code in
`crates/generator/src/field.rs` is one implementation of this document and not
the other way round.

Everything here is in atomic units. Why the numerics hold atomic units and every
file holds SI is #13, and the conversion between them is
`crates/units/src/convert.rs`.

Issue #43 is where these choices are argued.

## What is fixed here, and why each of the three is a choice

**The vector potential is the primary quantity.** The electric field is derived
from it. Written the other way round, the potential is an integral of the field
and carries a constant of integration that the field does not fix; the usual
choice, that the potential vanishes long before and long after the pulse, is
then available only for a field whose time integral is zero, which is a
condition somebody has to impose and can forget. Defining the potential first,
with an envelope that reaches zero at both ends, makes both statements true by
construction.

**The carrier is referred to the envelope's peak.** The envelope peaks at t = 0,
so the carrier-envelope phase is the carrier's phase there. A pulse with a
carrier-envelope phase of zero has its potential at a maximum at t = 0.

**The electric field is minus the time derivative of the potential.** Dipole
approximation and Coulomb gauge, so the potential depends on time and not on
position. This is the sign a plot of a trace does not show. Get it wrong and
every statement a reconstruction makes about the direction of the chirp reverses
while the trace still looks right, which is why it is written down here and
checked by a case rather than left in the code.

## The field

Four parameters: the peak of the potential `A0`, the carrier's angular frequency
`w`, the envelope's full support in optical cycles `n`, and the
carrier-envelope phase `cep`.

    period = 2 * pi / w
    half   = n * period / 2

The support is `-half <= t <= half`. Outside it every quantity below is exactly
zero.

    envelope(t) = cos(pi * t / (2 * half))^2
    A(t)        = A0 * envelope(t) * cos(w * t + cep)
    E(t)        = -dA/dt
                = -A0 * ( envelope'(t) * cos(w * t + cep)
                          - w * envelope(t) * sin(w * t + cep) )
    envelope'(t) = -(pi / (2 * half)) * sin(pi * t / half)

`n` is the full width of the support, not a width at half maximum. The two
differ by a factor nobody agrees on, and a number that has to be converted
before it can be compared is a number that will be compared without being
converted.

The envelope is a raised cosine and not a Gaussian. A Gaussian never reaches
zero, so a potential built on one is truncated somewhere, and where it is
truncated is a parameter nobody declares and every implementation picks for
itself. This one ends where it says it ends, with value and slope both zero
there.

## The worked example

One field, its potential and its electric field at three instants, so a reader
can check their own convention against this one with arithmetic rather than by
reading code.

    A0  = 1
    w   = 1/20
    n   = 4
    cep = 0

so

    period = 40 * pi = 125.663_706_143_591_72
    half   = 80 * pi = 251.327_412_287_183_45

| instant | t | envelope | A(t) | E(t) |
| --- | --- | --- | --- | --- |
| the envelope's peak | 0 | 1 | 1 | 0 |
| a quarter period later | 10 pi | cos(pi/16)^2 | 0 | w cos(pi/16)^2 |
| half a period later | 20 pi | cos(pi/8)^2 | -cos(pi/8)^2 | -sqrt(2)/320 |

In decimal, to the precision a double carries:

| t | envelope | A(t) | E(t) |
| --- | --- | --- | --- |
| 0 | 1 | 1 | 0 |
| 31.415_926_535_897_93 | 0.961_939_766_255_643_4 | 0 | 0.048_096_988_312_782_17 |
| 62.831_853_071_795_86 | 0.853_553_390_593_273_7 | -0.853_553_390_593_273_7 | -0.004_419_417_382_415_92 |

Each row is checkable by hand. `cos(pi/8)^2` is `(2 + sqrt(2)) / 4`, and
`cos(pi/16)^2` is `(2 + sqrt(2 + sqrt(2))) / 4`.

The three instants are chosen to separate the two terms of the field rather than
to be round. At the peak the envelope's slope is zero and the carrier's sine is
zero, so the field is zero from both sides at once. A quarter period later the
carrier's cosine is zero, so the potential is zero and the whole field comes
from the carrier term. Half a period later the carrier's sine is zero, so the
whole field comes from the envelope's slope, and its sign is the one that a
wrong sign convention flips.

Two of those zeros are zeros of exact arithmetic and not of a double. At a
quarter period the potential carries `cos(pi/2)`, which a double evaluates as
about `6e-17` rather than as zero, so the value there is a few times `1e-17` and
a check on it is a check against a small absolute bound. The other entries are
compared relatively.

## The phase

In the strong-field approximation and the velocity gauge, an electron born at
time `t` with asymptotic momentum `p` along the polarisation accumulates

    phase(p, t) = integral from t to half of
                      p * A(s) + A(s)^2 / 2   ds

The upper limit is the end of the support and not infinity. That is exact rather
than a truncation, because `A` is identically zero beyond it.

Two properties of this expression are decisions and not algebra.

**The momentum enters as itself.** It is not expanded about a central momentum.
That approximation is the one whose cost this repository exists to measure, so
the generator does not make it. A method that makes it declares it, which is
what makes the comparison in #5 mean anything.

**The squared term is kept.** A case may drop it, and then the case's own
declaration says so, which is #37. Dropping it silently would remove a
momentum-independent phase whose time dependence is exactly what a streaking
measurement reads.

The first term is `p` times a quantity that depends on the field alone:

    displacement(t) = integral from t to half of A(s) ds

which is worth naming because it is what makes the quadrature checkable without
a second implementation. Its derivative with respect to `t` is `-A(t)`, so a
reader with the potential and the integral can verify one against the other. The
same holds for the squared term against `-A(t)^2 / 2`. Those are the checks
`crates/generator/tests/field.rs` runs, rather than comparing the integral with
a second copy of itself.

The quadrature is composite Simpson over the interval, at a stated number of
intervals per optical cycle. What that number buys is measured by the derivative
check rather than asserted here.

## What this document does not fix

The target and its dipole, which is #44. The streaking operator that turns this
phase into a trace, which is #42. The grids these are evaluated on and the
resolution conditions the generator owes, which is #47. The polarisation
geometry is one dimension along the polarisation axis here, and a case with an
angle-resolved detector is not described by this document.
