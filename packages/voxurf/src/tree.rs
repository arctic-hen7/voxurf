use std::collections::HashMap;

use crate::model::selector_to_id;

/// The element tree for an interface. This should contain all elements relevant to some
/// action the user may wish to perform. This has a focus on interactive elements, like buttons,
/// links, and inputs, rather than noninteractive elements like blocks of text.
#[derive(PartialEq, Eq)]
pub struct Tree<S: PartialEq + Eq> {
    /// Nodes at the top-level of the tree.
    pub roots: Vec<Node<S>>,
    /// All the selectors in the tree. This is placed here for interfaces to construct to avoid
    /// recursive iteration twice (once in the tree construction and again in selector acquisition).
    pub selectors: Vec<S>,
}

/// A node in the element tree.
#[derive(PartialEq, Eq)]
pub struct Node<S> {
    /// The internal identifier of this node that can be used to select it for clicking, etc.
    pub selector: S,
    /// The name associated with the element. This should be able to be read to indicate the
    /// function of the element (e.g. the text of a link or button, the label of an input).
    pub name: Option<String>,
    /// A description of the element. In lieu of a name, this should say everything about the
    /// element.
    pub description: Option<String>,
    /// A freeform role for this element. This should be easily intelligible, something like
    /// `button`, `link`, or `input`. Provided the model can understand this, it's fine.
    pub role: Option<String>,
    /// The state of this element. This applies primarily to things like inputs, where this would
    /// contain their current contents.
    pub state: Option<String>,
    /// An arbitrary list of properties.
    pub properties: HashMap<String, String>,
    /// Children within this element in the interface.
    pub children: Vec<Node<S>>,
}
impl<S: PartialEq + Eq> Node<S> {
    /// Converts the node into a string suitable for LLM ingestion. This deliberately elides
    /// irrelevant information to save on tokens. This also takes a list of all selectors in the
    /// context of this node (i.e. the tree), which will be used to map this node's selector to
    /// a number, which allows interface-agnostic prompting and smaller token usage.
    pub(crate) fn to_string(&self, indent_level: usize, selectors: &Vec<S>) -> String {
        let internal_id = selector_to_id(&self.selector, selectors);

        format!(
            "{tabs}- [{id}] \"{name}\"{role}{desc}{props}{value}{children}",
            tabs = "\t".repeat(indent_level),
            id = internal_id,
            name = self.name.as_ref().map(|s| s.as_str()).unwrap_or("<null>"),
            role = if let Some(role) = &self.role {
                format!(" ({role})")
            } else {
                String::new()
            },
            desc = if let Some(desc) = &self.description {
                format!(" ({desc})")
            } else {
                String::new()
            },
            props = if !self.properties.is_empty() {
                let mut s = " {".to_string();
                for (key, val) in &self.properties {
                    s.push_str(&key);
                    s.push_str(": ");
                    s.push_str(&val);
                    s.push_str(", ")
                }
                format!("{}}}", s.strip_suffix(", ").unwrap())
            } else {
                String::new()
            },
            value = if let Some(val) = &self.state {
                format!(" with state {val}")
            } else {
                String::new()
            },
            children = if !self.children.is_empty() {
                let mut children_str = String::new();
                for child in &self.children {
                    children_str.push('\n');
                    children_str.push_str(&child.to_string(indent_level + 1, selectors));
                }
                children_str
            } else {
                String::new()
            }
        )
    }
}
