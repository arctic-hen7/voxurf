use std::future::Future;

use crate::Tree;

/// A trait for AI models which Voxurf can work with. This allows being generic over
/// both local and remote models of varying quality.
pub trait Model {
    /// Errors that may occur when working with the model.
    type Error: std::error::Error + 'static;

    /// Prompts the model with the given state. This should return a list of raw actions
    /// to take on the interface, which will be parsed and executed (before being reformulated
    /// so the model can reference its past actions in future ones if need be).
    fn prompt<S: Eq>(&self, prompt: State<S>) -> impl Future<Output = Result<Vec<RawAction>, Self::Error>>;
    // TODO Implement the above function for the user and have them implement a string-based one
}

/// The state of the "conversation" with the model. This catalogues all part interactions with the
/// processed actions that came of them, together with the old states of the tree and the current state,
/// along with the user's command. Through this representation, the model acts as a function for
/// converting a state into a list of [`RawAction`]s.
pub struct State<S: Eq> {
    /// The user's natural-language command.
    command: String,
    /// The current element tree of the page.
    tree: Tree<S>,
    /// Previous interactions with the model.
    past_interactions: Vec<Interaction<S>>,
}

/// A single interaction with the model. This includes the tree sent to the model and a *processed*
/// list of the actions it returned (i.e. any after `wait` ignored, etc.). The user's command is
/// not included because of the context in which this is used.
pub struct Interaction<S: Eq> {
    /// The state of the element tree at this time of this interaction.
    tree: Tree<S>,
    /// The (processed) list of actions the model sent back. Under the hood, a list of [`RawAction`]s
    /// is processed into an action group that ignores anything after a waitpoint (as the model is
    /// highly likely to hallucinate from then on about element IDs it doesn't yet know). For creating
    /// this representation, [`RawAction`]s are reconstituted. Although the parameters are identical,
    /// this is more like an idealised list of the actions we want the model to think it produced!
    actions: Vec<RawAction>,
}

/// A raw action produced by the model as a function call. Models which may not constrain their
/// function calls to the provided ones should be constrained in the model's code, and incorrect
/// parameters should be handled there too. However, the semantic interpretations of different
/// sequences will be handled by the caller of the model.
pub enum RawAction {
    /// The model wants to click an element.
    Click {
        /// The element's unique ID as provided to the model.
        id: usize,
    },
    /// The model wants to type some text into an element.
    Type {
        /// The element's unique ID.
        id: usize,
        /// The text the model wants to type in.
        text: String,
    },
    /// The model wants to wait for the page layout to change.
    Wait {
        /// The reason the model gave for wanting to wait.
        reason: String,
    }
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
    if id % SELECTOR_TO_ID_FACTOR != 0 {
        return None;
    }

    selectors.get((id / SELECTOR_TO_ID_FACTOR) as usize)
}
