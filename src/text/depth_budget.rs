use crate::text::error::{TextError, TextResult};

#[derive(Copy, Clone)]
pub struct DepthBudget {
    budget: usize,
}

impl DepthBudget {
    pub fn new(budget: usize) -> Self {
        DepthBudget { budget }
    }
    pub fn child(&self) -> TextResult<DepthBudget> {
        Ok(DepthBudget {
            budget: self
                .budget
                .checked_sub(1)
                .ok_or(TextError::DepthBudgetExceeded)?,
        })
    }
}
