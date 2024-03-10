use crate::tree::Tree;

/// The core trait in Voxurf, which allows interacting with arbitrary interfaces.
/// This abstracts over the accesibility properties of native, web, mobile, and any
/// other interface imaginable to provide something we can work with in a unified manner
/// internally.
pub trait Interface {
    /// The type of identifiers that can be used for selecting elements for action, like
    /// clicking.
    type Selector: PartialEq + Eq;
    /// Errors that can occur while interacting with the interface. Generally, most operations
    /// in interfaces will be infallible, but this exists for those rare cases where something
    /// like clicking an element might fail for any reason other than the heat death of the
    /// universe.
    type Error: std::error::Error + 'static;

    /// Clicks the given element.
    async fn primary_click_element(&self, selector: &Self::Selector) -> Result<(), Self::Error>;
    /// Types the given text into the given element. This may entail creating a focus
    /// state, but such information shoudl be abstracted from Voxurf.
    async fn type_into_element(
        &self,
        selector: &Self::Selector,
        text: &str,
    ) -> Result<(), Self::Error>;
    /// Computes the tree of relevant elements for this interface. This should be a low-latency
    /// operation, and care should be taken to ensure this operates as quickly as feasibly
    /// possible.
    async fn compute_tree(&self) -> Result<Tree<Self::Selector>, Self::Error>;

    // TODO Announcement functions
}
