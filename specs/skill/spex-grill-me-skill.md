# `spex-grill-me` skill

Grills the user relentlessly about a plan, decision, or idea to stress-test their thinking, until the user and agent share an understanding.

## Acceptance criteria

- MUST interview the user relentlessly about every aspect of the plan, decision, or idea until they reach a shared understanding.
- MUST walk down each branch of the decision tree, resolving the dependencies between decisions one by one.
- MUST provide its recommended answer for each question.
- MUST ask the questions one at a time, waiting for the user's feedback on each before continuing.
- MUST look up a fact that can be found by exploring the environment (filesystem, tools, etc.) rather than asking the user.
- MUST put each decision to the user and wait for their answer: the decisions are the user's.
- MUST NOT ask multiple questions at once.
- MUST NOT act on the plan, decision, or idea until the user confirms they have reached a shared understanding.
