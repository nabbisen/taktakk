# RFC-013: Lesson runner state machine

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M4 |
| **Priority** | — |

## Summary

LessonRunner: Advance/Answer/Back events → StepAdvanced/AnswerCorrect/AnswerIncorrect/MaxAttemptsReached/LessonComplete responses.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
