# Contributing

Thanks for contributing to Trust Tasks. This page covers how to get a change
merged. The rest of the documentation:

- **A Trust Task specification** — [`CONTRIBUTING-SPECS.md`](CONTRIBUTING-SPECS.md).
- **The framework specification** — `SPEC.md` here is generated from
  [trustoverip/dtgwg-trust-tasks-spec](https://github.com/trustoverip/dtgwg-trust-tasks-spec);
  a framework change is a pull request to that repository. Never edit `SPEC.md`
  here ([why](docs/DEVELOPMENT.md#specmd-is-generated--never-edit-it-here)).
- **The build, the website, the generators and the four libraries** —
  [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md), with per-library notes in
  [`docs/packages/`](docs/packages/) ([Rust](docs/packages/rust.md),
  [TypeScript](docs/packages/typescript.md), [Go](docs/packages/go.md),
  [Dart](docs/packages/dart.md)).
- **Releases** — [`RELEASING.md`](RELEASING.md).
- **Security issues** — [`SECURITY.md`](SECURITY.md); do not open a public issue.

By opening a pull request you agree to the Open Web Foundation Contributor
License Agreement, reproduced in full [below](#contributor-license-agreement).
Source code is contributed under the Apache License 2.0 in
[`SOURCE_CODE.md`](SOURCE_CODE.md).

## Every commit MUST be DCO signed

This repo enforces the **Developer Certificate of Origin**. *Every* commit needs
a `Signed-off-by: Name <email>` trailer matching the commit author — the DCO CI
check fails the PR otherwise. Use **`git commit -s`** (and `git commit -s --amend`
to fix the most recent commit, or an interactive rebase with `--signoff` to fix a
range). Verify with `git log -1 --format='%(trailers:key=Signed-off-by)'`.

## Before you open the pull request

```sh
npm install        # one-time, at the repository root (Node 24, see .nvmrc)
npm run check      # validation and the repository's own consistency checks
npm run build      # validate specs + regenerate the website registry
```

- **Changed anything under `specs/`?** Regenerate **all four** libraries in the
  same PR — Rust, TypeScript, Go and Dart. CI's drift jobs fail otherwise. The
  commands are in
  [Changing a spec or payload schema](docs/DEVELOPMENT.md#changing-a-spec-or-payload-schema--regenerate-the-libraries).
- **Added a category, a framework-reserved slug, a binding or a library?** Each
  has a hand-maintained list to update as well —
  [Hand-maintained lists](docs/DEVELOPMENT.md#hand-maintained-lists).
- **Do not bump a version or write a `CHANGELOG.md` entry.** Versions and
  changelogs are produced by the Release PR from the merged commits; CI refuses
  a version change anywhere else. See
  [Merging is not releasing](docs/DEVELOPMENT.md#merging-is-not-releasing--read-releasingmd-before-touching-a-version).

## The pull request title is the changelog

PRs squash-merge, so the title becomes the commit subject — both the published
changelog entry and the signal that sizes each package's version bump.
`.github/workflows/commit-lint.yml` enforces the conventional-commit format:

- `feat(<slug>):` when you **add** a spec family; `spec(<slug>):` for an edit
  inside one (`spec` is scored as a patch).
- `fix(…)`, `docs:`, `ci(…)` and the other conventional types as usual.
- **A `!` is only for a package's public API, scoped to the package**
  (`feat(rs)!:`, `fix(codegen)!:`) — **never for a spec change.** One `!` makes
  every crate, and the npm, Go and Dart packages, a breaking release; commit-lint
  refuses a `!` whose scope is not a package, and a `BREAKING CHANGE` line in a
  commit message or the PR description.

The commit body is included in the changelog verbatim.

## Review

- Touch only your own spec folder/namespace where possible. `.github/CODEOWNERS`
  is effectively single-owner today (one maintainer on every rule but
  `/specs/process-attestation/`), so it does not route review per slug and there
  is no second reviewer of record (#1144); don't write prose that says otherwise.
  Displayed spec authors come from front matter or git history, never from
  CODEOWNERS.
- See [`CONTRIBUTING-SPECS.md`](CONTRIBUTING-SPECS.md) for the full
  spec-authoring guide.

## Contributor License Agreement

                                    OWF Contributor License Agreement 1.0 - Copyright and Patent
                                                    Open Web Foundation

                                            Contributor License Agreement (CLA 1.0)
                                                (Patent and Copyright Grants)


1.  The Purpose of this Contributor License Agreement.  This CLA sets forth the terms under which I will participate in and contribute to the development of the Specification. Capitalized terms are defined in the CLA's last section.

2.  Copyrights.

2.1.  Copyright Grant.   I grant to you a perpetual (for the duration of the applicable copyright), worldwide, non-exclusive, no-charge, royalty-free, copyright license, without any obligation for accounting to me, to reproduce, prepare derivative works of, publicly display, publicly perform, sublicense, distribute, and implement any Contribution to the full extent of my copyright interest in the Contribution.

2.2.  Attribution.  As a condition of the copyright grant, you must include an attribution to the Specification in any derivative work you make based on the Specification.  That attribution must include, at minimum, the Specification name and version number.

3.  Patents.

3.1.  Patent Non-Assert.

3.1.1. The Promise.  I, on behalf of myself and my successors in interest and assigns, irrevocably promise not to assert my Granted Claims against you for your Permitted Uses, subject to the terms and conditions of Section 3.1.  This is a personal promise directly from me to you, and you acknowledge as a condition of benefiting from it that no rights from me are received from suppliers, distributors, or otherwise in connection with this promise. This promise also applies to your Permitted Uses of any other specifications incorporating all required portions of the Specification.

3.1.2.  Termination.

3.1.2.1.  As a Result of Claims by You.  All rights, grants, and promises made by me to you under this CLA are terminated if you file, maintain, or voluntarily participate in a lawsuit against me or any person or entity asserting that its Permitted Uses infringe any  Granted Claims you would have had the right to enforce had you signed this CLA, unless that suit was in response to a corresponding suit first brought against you.

3.1.2.2.  As a Result of Claims by a Related Entity of Mine.  If a Related Entity of mine files, maintains, or voluntarily participates in a lawsuit asserting that a Permitted Use infringes any Granted Claims it would have had the right to enforce had it signed this CLA, then I relinquish any rights, grants, and promises I have received for the Specification from other signatories of this CLA, unless a) my promise to you was terminated pursuant to section 3.1.2.1, or b) that suit was in response to a corresponding suit first brought by you against the Related Entity.

3.1.3.  Additional Conditions.  This promise is not an assurance (i) that any of my copyrights or issued patent claims cover an implementation of the Specification or are enforceable or (ii) that an implementation of the Specification would not infringe intellectual property rights of any third party. Notwithstanding the personal nature of my promise, this promise is intended to be binding on any future owner, assignee or exclusive licensee who has been given the right to enforce any Granted Claims against third parties.

3.1.4. Bankruptcy.  Solely for purposes of Section 365(n) of Title 11, United States Bankruptcy Code and any equivalent law in any foreign jurisdiction, this promise will be treated as if it were a license and you may elect to retain your rights under this promise if I (or any owner of any patents or patent applications referenced herein), as a debtor in possession, or a bankruptcy trustee, reject this non-assert.

3.2.  Patent License Commitment.  In addition to rights granted in 3.1, on behalf of me and my successors in interest and assigns, I agree to grant to you a no charge, royalty free license to my Granted Claims on reasonable and non-discriminatory terms, where such license applies only to those Granted Claims infringed by the implementation of my Contribution(s) alone or by combination of my Contribution(s) with the Specification, solely for your Permitted Uses.

4.  No Other Rights. Except as specifically set forth in this CLA, no other express or implied patent, trademark, copyright, or other property rights are granted under this CLA, including by implication, waiver, or estoppel.

5.  Limited Opt-Out.  I may withdraw my Contribution by providing written notice of that withdrawal within 45 days of submitting that Contribution.  Notice of a Contribution withdrawal must be made, at minimum, in writing using the same communication mechanisms that were used to submit the corresponding Contribution and must include the exact material being withdrawn. Upon providing such valid notice, any obligations I incurred under this CLA for that particular identified Contribution will be null and void.

6.  Open Web Foundation Agreement ("OWFa") version 1.0 Execution.  I acknowledge that the goal of this CLA is to develop a specification that will be subject to the OWFa version 1.0.  While I have no legal obligation to execute the OWFa version 1.0 for any version of the specification being developed under this CLA, I agree that the selection and terms of the OWFa version 1.0 will not be subject to negotiation.

7.  Antitrust Compliance.  I acknowledge that I may compete with other participants, that I am under no obligation to implement the Specification, that each participant is free to develop competing technologies and standards, and that each party is free to license its patent rights to third parties, including for the purpose of enabling competing technologies and standards.

8.  Non-Circumvention.  I agree that I will not intentionally take or willfully assist any third party to take any action for the purpose of circumventing my obligations under this CLA.

9.  Representations, Warranties and Disclaimers.  I represent and warrant that 1) I am legally entitled to grant the rights and promises set forth in this CLA and 2) I will not intentionally include any third party materials in any Contribution unless those materials are available under terms that do not conflict with this CLA. IN ALL OTHER RESPECTS MY CONTRIBUTIONS ARE PROVIDED "AS IS."  The entire risk as to implementing or otherwise using the Contribution or the Specification is assumed by the implementer and user. Except as stated herein, I expressly disclaim any warranties (express, implied, or otherwise), including implied warranties of merchantability, non-infringement, fitness for a particular purpose, or title, related to the Contribution or the Specification. IN NO EVENT WILL ANY PARTY BE LIABLE TO ANY OTHER PARTY FOR LOST PROFITS OR ANY FORM OF INDIRECT, SPECIAL, INCIDENTAL, OR CONSEQUENTIAL DAMAGES OF ANY CHARACTER FROM ANY CAUSES OF ACTION OF ANY KIND WITH RESPECT TO THIS CLA, WHETHER BASED ON BREACH OF CONTRACT, TORT (INCLUDING NEGLIGENCE), OR OTHERWISE, AND WHETHER OR NOT THE OTHER PARTY HAS BEEN ADVISED OF THE POSSIBILITY OF SUCH DAMAGE. All of my obligations under Section 3 regarding the transfer, successors in interest, or assignment of Granted Claims will be satisfied if I notify the transferee or assignee of any patent that I know contains Granted Claims of the obligations under Section 3. Nothing in this CLA requires me to undertake a patent search.

10.  Definitions.

10.1.  Bound Entities.  "Bound Entities" means the entity listed below and any entities that the Bound Entity Controls.

10.2.  CLA.  "CLA" means this document, which sets forth the rights, grants, promises, limitations, conditions, obligations, and disclaimers made available for my Contributions to the particular Specification.

10.3.  Contribution.   "Contribution" means any original work of authorship, including any modifications or additions to an existing work, that I intentionally submit for inclusion in the Specification, which is included in the Specification.  For the purposes of this definition, "submit" means any form of electronic, oral, or written communication for the purpose of discussing and improving the Specification, but excluding communication that I conspicuously designate in writing as not a contribution.

10.4.  Control.  "Control" means direct or indirect control of more than 50% of the voting power to elect directors of that corporation, or for any other entity, the power to direct management of such entity.

10.5.  Granted Claims.  "Granted Claims" are those patent claims that I own or control, including those patent claims I acquire or control after the Date below, that are infringed by Permitted Uses. Granted Claims include only those patent claims that are infringed by the implementation of any portions of the Specification where the Specification describes the functionality causing the infringement in detail and does not merely reference the functionality causing the infringement. Granted Claims under this CLA exclude those patent claims that would be infringed by an implementation of the Specification if my Contribution to that Specification were removed.

10.6.  I, Me, or My.   "I," "me," or "my" refers to the signatory below and its Bound Entities, if applicable.

10.7.  Permitted Uses.  "Permitted Uses" means making, using, selling, offering for sale, importing or distributing any implementation of the Specification 1) only to the extent it implements the Specification and 2) so long as all required portions of the Specification are implemented. Permitted Uses do not extend to any portion of an implementation that is not included in the Specification.

10.8.  Related Entities.  "Related Entities" means 1) any entity that Controls the Bound Entity ("Upstream Entity"), and 2) any other entity that is Controlled by an Upstream Entity that is not itself a Bound Entity.

10.9.  Specification.  "Specification" means the Specification identified below as of the date of my last Contribution.

10.10.  You or Your.  "You," "you," or "your" means any person or entity who exercises copyright or patent rights granted under this CLA, and any person or entity you Control.
