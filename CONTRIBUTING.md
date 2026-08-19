# Contributing

Contributions to Iguana should be focused, tested, and small enough to review.
The [development guide](docs/development.md) covers setup, build commands, and
generated-code workflows. The [architecture guide](docs/architecture.md)
describes the repository and the parser-generation pipeline.

## Before opening a pull request

- **Build and test the change.** Run the checks relevant to the files you
  changed. Generator changes require the full regeneration sequence described
  in the development guide.
- **Review generated output.** A generated diff should follow from the grammar
  or generator change. Unexpected generated changes usually indicate a problem
  in the generator or workflow. Change generated files through the grammar or
  generator, then run the corresponding regeneration workflow; do not patch
  generated files afterward with text-rewrite scripts.
- **Describe the change and its checks.** The pull request should state what
  changed, why it changed, and which checks you ran.
- **Keep the change reviewable.** Separate unrelated work. A large generated
  change may be rejected on size alone.

## Licensing of contributions

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in Iguana is licensed under `MIT OR Apache-2.0`,
without additional terms or conditions. See [LICENSING.md](LICENSING.md).

Code contributed to the generator may be reproduced in generated parsers. By
submitting such a contribution, you grant recipients the same permission for
that reproduced material that the generated-code terms in
[LICENSING.md](LICENSING.md#generated-code) grant for other generated files.

If you contribute as part of your job, confirm that you have your employer's
permission when the employer owns the work.

If any part of a submission is not your own work, identify its source and
license in the pull request. This information is needed to review license
compatibility and required notices.

When adding third-party code or assets to a distributed artifact, preserve the
license information and update the artifact's notice file in the same change.

## Use of AI tools

AI tools may be used, but the contributor remains responsible for the result.
You must understand the concepts, code, and behavior of the change and be able
to explain all three in review. A contribution whose author cannot do so may
be closed without detailed review.

Commits list only the submitter as author. Do not add AI co-author or
attribution trailers; CI rejects them. Whatever tools you use, you are
responsible for having the rights to submit the result. AI output can reproduce
copyrighted material, and processing it with a tool does not remove that
copyright.
