# AI Agent Instructions

## Commit message policy for AI-authored commits

When an AI assistant creates a commit in this repository, it must use a detailed
commit message.

- Use a concise, specific subject line.
- Include a body that explains the motivation for the change.
- Include important implementation details and any migration impact.
- Always record which AI model or models were used to produce the commit.
- Avoid one-line messages unless the change is truly trivial.

## Dependency policy

- Always use the `polkadot-sdk` crate for FRAME/Substrate dependencies in this repository.
