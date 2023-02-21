use crate::types::{
    defs::{Statement, Subject},
    enums::subject::Subjects,
    error::StatementError,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, JsonSchema, Serialize)]
#[serde(rename = "statement")]
pub struct Reddit {
    pub handle: String,
    pub subject: Subjects,
}

impl Statement for Reddit {
    fn generate_statement(&self) -> Result<String, StatementError> {
        Ok(format!(
            "I am attesting that this Reddit handle {} is linked to the {} {}",
            self.handle,
            self.subject.statement_title()?,
            self.subject.display_id()?
        ))
    }
}
