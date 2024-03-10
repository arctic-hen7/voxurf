/// A trait for AI models which Voxurf can work with. This allows being generic over
/// both local and remote models of varying quality.
pub trait Model {
    /// Errors that may occur when working with the model.
    type Error: std::error::Error + 'static;

    /// Prompts the model with the given prompt and returns its response
    /// as a string. This is expected to use chat-style prompting.
    async fn prompt(&self, prompt: &str) -> Result<String, Self::Error>;
}

/// A factor by which to multiply the index of a selector in the list of all selectors in
/// the element tree. Creating more distance between them reduces hallucinations.
const SELECTOR_TO_ID_FACTOR: usize = 3;

/// Converts the given selector to an ID, based on a list of all selectors in the element tree.
pub(crate) fn selector_to_id<S: PartialEq + Eq>(selector: &S, selectors: &Vec<S>) -> usize {
    selectors.iter().position(|s| s == selector).unwrap() * SELECTOR_TO_ID_FACTOR
}
/// Converts the given ID to a selector, given a list of all selectors in the element tree.
pub(crate) fn id_to_selector<S>(id: usize, selectors: &Vec<S>) -> Option<&S> {
    if id % SELECTOR_TO_ID_FACTOR != 0 { return None }

    selectors.get((id / SELECTOR_TO_ID_FACTOR) as usize)
}
