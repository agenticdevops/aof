# AOF Governance

This document describes the governance model for the AOF (Agentic Ops Framework) project.

## Project Mission

AOF aims to provide a production-ready, open-source framework for building agentic applications that can orchestrate LLM-powered agents for DevOps, SRE, and infrastructure automation tasks.

## Governance Model

AOF follows a **meritocratic governance model** aligned with CNCF best practices.

### Roles

#### Users
Anyone who uses AOF. Users may provide feedback through issues, discussions, or community channels.

#### Contributors
Anyone who contributes to the project through:
- Code contributions (PRs)
- Documentation improvements
- Bug reports and feature requests
- Community support and advocacy
- Testing and validation

#### Maintainers
Maintainers have write access to the repository and are responsible for:
- Reviewing and merging pull requests
- Triaging issues
- Releasing new versions
- Ensuring code quality and security
- Guiding architectural decisions

Current Maintainers:
- @gshah - Project Lead

#### Technical Steering Committee (TSC)
The TSC is responsible for:
- Project direction and roadmap
- Major architectural decisions
- Governance changes
- Maintainer elections
- CNCF coordination

TSC Members:
- @gshah - Chair

## Decision Making

### Lazy Consensus
Most decisions are made through lazy consensus. A proposal is considered accepted if:
1. It is posted publicly (GitHub issue/PR/discussion)
2. A reasonable amount of time has passed (typically 72 hours)
3. No maintainer has objected

### Voting
For significant decisions where consensus cannot be reached:
- Maintainers may call for a vote
- Each maintainer gets one vote
- Simple majority wins
- Tie-breaker: TSC Chair

### What Requires a Vote
- Adding or removing maintainers
- Major breaking changes
- Governance changes
- Licensing changes

## Becoming a Maintainer

Contributors may be nominated for maintainer status based on:
- Sustained contributions over 6+ months
- Deep understanding of the codebase
- Demonstrated good judgment
- Community engagement
- Endorsement by existing maintainer

Process:
1. Existing maintainer nominates candidate
2. TSC reviews contribution history
3. Maintainers vote (2/3 majority required)
4. 1-week objection period
5. Access granted

## Code of Conduct

All participants must follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Communication Channels

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: Questions, ideas, community engagement
- **Slack/Discord**: Real-time chat (coming soon)

## Release Process

1. Maintainer proposes release (version bump PR)
2. Changelog updated
3. CI passes all tests
4. At least one maintainer approval
5. Tag created and pushed
6. Release published to crates.io
7. Release notes published on GitHub

## CNCF Alignment

AOF is designed for potential CNCF sandbox submission. We follow:
- [CNCF Code of Conduct](https://github.com/cncf/foundation/blob/main/code-of-conduct.md)
- [CNCF IP Policy](https://github.com/cncf/foundation/blob/main/charter.md)
- Cloud-native principles

### Sandbox Requirements
For CNCF Sandbox acceptance:
- [ ] Apache 2.0 License
- [ ] Clear governance document
- [ ] Code of Conduct
- [ ] CONTRIBUTING guide
- [ ] Security policy
- [ ] 2+ maintainers from different organizations
- [ ] Public roadmap
- [ ] Vendor neutrality

### Incubation Requirements
For CNCF Incubation:
- [ ] All sandbox requirements
- [ ] Adoption by 3+ organizations
- [ ] Healthy contributor growth
- [ ] Documented production use cases
- [ ] Security audit completed
- [ ] Clear project scope

### Graduation Requirements
For CNCF Graduation:
- [ ] All incubation requirements
- [ ] Broad adoption (10+ production users)
- [ ] Committers from 3+ organizations
- [ ] Documented security response process
- [ ] Achieved CII Best Practices badge

## Amendments

This governance document may be amended through:
1. Pull request with proposed changes
2. Discussion period (minimum 1 week)
3. TSC approval
4. Maintainer vote (2/3 majority)

---

*Last updated: December 2024*
