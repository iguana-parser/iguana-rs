# Contributing

Thanks for your interest in contributing to iguana-rs. Setup, build, and
test workflows are covered in [docs/development.md](docs/development.md), and
the project layout in [docs/architecture.md](docs/architecture.md).

## Licensing of contributions

iguana-rs is multi-licensed (see [LICENSE.md](LICENSE.md) for the per-crate breakdown), and parts of that arrangement, such as the GPL section 7 permission covering generated parsers, require unified licensing authority to maintain.

Contributions therefore require signing the [Contributor License Agreement](CLA.md), based on the Harmony HA-CLA-I template. You keep the copyright in your contribution. By signing, you grant the project owner a broad, perpetual, irrevocable license to it, including the right to sublicense it under any terms. Licensing decisions for the project rest solely with the owner; your contribution is also always available under the license the receiving crate carried when you submitted it, as listed in LICENSE.md. The agreement covers iguana-rs and any other project of ours you later contribute to. Signing is electronic: the CLA Assistant check on your first pull request asks you to sign with your GitHub account.

If you contribute as part of your job, the agreement's section 3(c) requires your employer's approval or a signed [entity agreement](CLA-ENTITY.md). Arrange that before opening your first pull request.

If any part of your submission is not your own work, do not fold it into the contribution: mark it clearly in the pull request and identify its source and license. Such material stays under its original license and is not covered by the CLA. Whether that license is compatible with the receiving crate is our call.

## Use of AI tools

Using AI tools is fine. We use Claude in developing Iguana. What matters is not how code was produced but who stands behind it: you are responsible for the code you submit. You need to fully understand the concepts, the code, and the behavior of your change, and be able to explain all three in review without the tool. Contributions whose authors cannot are closed without detailed review.

Commits carry the submitter's name only. Do not add AI co-author trailers (CI rejects them). The [CLA](CLA.md) applies unchanged: whatever tools you used, you certify the rights in what you submit. AI output can silently break that certification: a model can reproduce copyrighted code from its training data, and passing it through a tool does not remove the copyright.

Keep changes sized for human review. A large generated change is rejected on size alone.
