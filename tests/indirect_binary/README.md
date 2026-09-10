# Indirect binary precedence

This grammar tests direct and indirect operands, precedence, associativity,
exclusions, and alternatives without recursive ends.

Precedence declarations relate alternatives through recursive ends of their
head nonterminal. They do not assign a global precedence to operators or
guarantee that every input has one parse.

## Presence of indirect recursive ends

For `X = Y`, `Y = E`, grammar analysis establishes that every path reaches
`E`. The intermediate nonterminals pass the precedence argument toward `E`
and forward its result. No runtime presence check is needed.

For `Y = E | "b"`, analysis identifies only the `E` alternative as exposing
the recursive end. Parsing selects the alternative. The `E` branch forwards
its precedence; the `"b"` branch returns undefined precedence because it has
no `E`-end. The implementation encodes this value as `UNDEFINED_PRECEDENCE`
(-1). A present end where `E` derives an atom returns precedence zero.

An absent left end bypasses the enclosing alternative's precedence checks.
An absent right end makes its precedence return zero. This is the treatment
of absent indirect ends in [PEPM16, section 3.5](https://cdn.jsdelivr.net/gh/iguana-parser/papers@master/pepm16.pdf).
The binary rewrite applies this distinction at both operands and enforces
binary associativity only when both ends are present.

The `ClosedPrefix`, `ClosedPostfix`, `ClosedLeftAssoc`, and `ClosedRightAssoc`
cases exercise branches without recursive ends. Their accepted inputs and
ambiguities are intentional.

## Literals in the operand and in the head

The `OperandLiteral` cases use this shape:

```text
A = "x"
  > left AO "*" AO
  > left AO "+" AO
AO = A | "1"
```

`AO → A → "x"` reaches the recursive head. `AO → "1"` does not. An inner
`1+1` returns precedence zero and no associativity group, so an enclosing
operator accepts it without either restriction. The golden files retain both
groupings of `1+1*1` and both nestings of `1+1+1`.

The `HeadLiteral` cases instead declare the atom level as `A = "x" | "1"`
and use `AO = A`. Every operand then reaches `A`.

| Input | Literal only in `AO` | Literal in `A` |
|-------|----------------------|----------------|
| `x+x*x` | `x+(x*x)` | `x+(x*x)` |
| `x+x+x` | `(x+x)+x` | `(x+x)+x` |
| `1+1*1` | Both groupings | `1+(1*1)` |
| `1+1+1` | Both nestings | `(1+1)+1` |
| `x+1*1` | Both groupings | `x+(1*1)` |
| `1*1+1` | Both groupings | `(1*1)+1` |

## Nullable boundaries

End discovery uses the first and last grammar symbols. It does not scan past
nullable symbols. For `X = E Opt`, a non-empty `Opt` hides `E` from the right
boundary. The `NullableBoundary*` cases verify that the rewrite accepts these
inputs and retains the ambiguities left when `Opt` derives empty.

The `Dynamic/prefix_left` golden file also contains two groupings:
its prefix and binary operators share a precedence level. Binary associativity
does not choose between a prefix and a binary operator.
