//! Lesson runner: the step-advance state machine.
//!
//! The runner owns the current lesson steps and drives progression.
//! It is pure logic; no I/O, no storage calls, no UI.
//!
//! After each [`advance`] the caller is responsible for persisting
//! [`LessonRunner::state`] to storage before presenting the next step.

use crate::error::{EngineError, EngineResult};
use crate::exercise::{evaluate, ExerciseAnswer, EvalResult};
use crate::state::{LessonState, LessonStateStatus};
use crate::step::{StepContent, StepKind};

/// Events that a caller can feed into the runner.
#[derive(Debug)]
pub enum RunnerEvent {
    /// Advance past a non-interactive step (text, SVG, audio).
    Advance,
    /// Submit an exercise answer.
    Answer(ExerciseAnswer),
    /// Navigate backward one step.
    Back,
}

/// The runner's response to an event.
#[derive(Debug, PartialEq, Eq)]
pub enum RunnerResponse {
    /// Moved to the next step; persist state now.
    StepAdvanced { new_order: u32 },
    /// Exercise was correct; advance to next step.
    AnswerCorrect { new_order: u32 },
    /// Exercise was incorrect; stay on the same step.
    AnswerIncorrect { attempts_used: u32, max_attempts: u32 },
    /// Exercise max attempts exhausted; auto-advance.
    MaxAttemptsReached { new_order: u32 },
    /// Moved back one step.
    StepBack { new_order: u32 },
    /// Already at the first step.
    AtFirstStep,
    /// Lesson is complete.
    LessonComplete,
}

/// Active lesson runner.
pub struct LessonRunner {
    pub state: LessonState,
    steps: Vec<StepContent>,
    /// Attempts on the current exercise step (reset on advance).
    current_attempts: u32,
}

impl LessonRunner {
    /// Create a new runner, restoring from an existing [`LessonState`] if provided.
    pub fn new(state: LessonState, steps: Vec<StepContent>) -> Self {
        Self { state, steps, current_attempts: 0 }
    }

    /// The current step index (zero-based).
    pub fn current_order(&self) -> u32 {
        self.state.next_step_order()
    }

    /// The step currently being displayed (immutable view).
    pub fn current_step(&self) -> EngineResult<&StepContent> {
        let order = self.current_order() as usize;
        self.steps
            .get(order)
            .ok_or(EngineError::StepOutOfRange { index: order, len: self.steps.len() })
    }

    /// Total number of steps in this lesson.
    pub fn total_steps(&self) -> u32 {
        self.steps.len() as u32
    }

    /// Process a runner event and return how the state changed.
    pub fn handle(&mut self, event: RunnerEvent) -> EngineResult<RunnerResponse> {
        if self.state.status == LessonStateStatus::Completed {
            return Ok(RunnerResponse::LessonComplete);
        }

        match event {
            RunnerEvent::Advance => self.do_advance(),
            RunnerEvent::Answer(answer) => self.do_answer(answer),
            RunnerEvent::Back => self.do_back(),
        }
    }

    fn do_advance(&mut self) -> EngineResult<RunnerResponse> {
        let order = self.current_order();
        let step = self
            .steps
            .get(order as usize)
            .ok_or(EngineError::StepOutOfRange { index: order as usize, len: self.steps.len() })?;

        // Exercises must be answered, not advanced past.
        if matches!(step.kind, StepKind::Exercise(_)) {
            // Allow advancing past an exercise only if max attempts exhausted.
            // Normal path is through RunnerEvent::Answer.
            return Err(EngineError::InvalidAnswer);
        }

        self.complete_current_step(order)
    }

    fn do_answer(&mut self, answer: ExerciseAnswer) -> EngineResult<RunnerResponse> {
        let order = self.current_order();
        let step = self
            .steps
            .get(order as usize)
            .ok_or(EngineError::StepOutOfRange { index: order as usize, len: self.steps.len() })?;

        let spec = match &step.kind {
            StepKind::Exercise(s) => s.clone(),
            _ => return Err(EngineError::InvalidAnswer),
        };

        let max = crate::exercise::max_attempts(&spec).unwrap_or(u32::MAX);
        self.current_attempts += 1;

        match evaluate(&spec, &answer)? {
            EvalResult::Correct => {
                self.current_attempts = 0;
                self.complete_current_step(order)
                    .map(|r| match r {
                        RunnerResponse::StepAdvanced { new_order } => {
                            RunnerResponse::AnswerCorrect { new_order }
                        }
                        other => other,
                    })
            }
            EvalResult::Incorrect | EvalResult::Partial => {
                let attempts = self.current_attempts;
                if attempts >= max {
                    self.current_attempts = 0;
                    self.complete_current_step(order)
                        .map(|r| match r {
                            RunnerResponse::StepAdvanced { new_order } => {
                                RunnerResponse::MaxAttemptsReached { new_order }
                            }
                            other => other,
                        })
                } else {
                    Ok(RunnerResponse::AnswerIncorrect {
                        attempts_used: attempts,
                        max_attempts: max,
                    })
                }
            }
        }
    }

    fn do_back(&mut self) -> EngineResult<RunnerResponse> {
        let order = self.current_order();
        if order == 0 {
            return Ok(RunnerResponse::AtFirstStep);
        }
        // Move back by unwinding last_completed_step_order.
        let prev_order = order - 1;
        self.state.last_completed_step_order = if prev_order == 0 {
            None
        } else {
            Some(prev_order - 1)
        };
        self.current_attempts = 0;
        Ok(RunnerResponse::StepBack { new_order: prev_order })
    }

    fn complete_current_step(&mut self, order: u32) -> EngineResult<RunnerResponse> {
        let is_last = self.state.complete_step(order);
        if is_last {
            Ok(RunnerResponse::LessonComplete)
        } else {
            Ok(RunnerResponse::StepAdvanced { new_order: order + 1 })
        }
    }
}
