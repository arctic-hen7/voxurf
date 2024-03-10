use thiserror::Error;

/// Errors that can occur in Voxurf.
#[derive(Debug, Error)]
pub enum Error {}

/// Errors that can occur while executing a user command.
#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("model returned id {id}, but no such id exists in the executor's id map (likely hallucination)")]
    IdNotFound { id: usize },
    #[error("element tree did not stabilise in {timeout_ms}")]
    TreeStabilisationTimeout { timeout_ms: u32 },
    #[error("tree did not update at model-designated waitpoint after {timeout_ms} (either an error has occurred in the page or the model incorrectly estimated when the page would update)")]
    NoTreeUpdate { timeout_ms: u32 },
    #[error("error occurred in model")]
    ModelError {
        #[source]
        source: Box<dyn std::error::Error>,
    },
    #[error("error occurred in interface")]
    InterfaceError {
        #[source]
        source: Box<dyn std::error::Error>,
    },
    #[error("failed to parse an action from the model (likely hallucination or failure to follow prompt)")]
    ActionParseError(#[from] ActionParseError),
    #[error("command not finished after {num_trips} trips (threshold prevented further requests to model)")]
    CommandNotFinished { num_trips: u32 },
}

/// Errors that can occur while parsing an action string from the model.
#[derive(Debug, Error)]
pub enum ActionParseError {
    #[error("missing id in action of type '{ty}'")]
    MissingId { ty: String },
    #[error("found non-integer id")]
    NonIntegerId { id: String },
    #[error("missing text to type in typing action")]
    MissingTextInType,
    #[error("text in typing stage not single-quoted")]
    TextInTypeNotSingleQuoted { text: String },
    #[error("missing description in wait/finish action")]
    MissingDescription,
    #[error("found actions after finish")]
    ActionsAfterFinish,
}
