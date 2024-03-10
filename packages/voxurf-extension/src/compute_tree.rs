use crate::{glue::get_raw_ax_tree, interface::WeiError};
use serde::Deserialize;
use std::collections::HashMap;
use voxurf::{Node, Tree};
use wasm_bindgen::JsValue;

/// Maximum number of iterations in tree reconstitution. This prevents infinite loops.
const MAX_ITERS: usize = 50;

/// Converts the given `JsValue` to a string intelligently for sending arbitrary
/// properties to a model.
fn js_value_to_string(val: JsValue) -> String {
    let wrapped = format!("{:?}", val);
    let output = wrapped.strip_prefix("JsValue(").unwrap();
    let output = output.strip_suffix(')').unwrap();
    let output = output.strip_prefix('"').unwrap_or(output);
    let output = output.strip_suffix('"').unwrap_or(output);

    output.to_string()
}
/// Helper function to produce [`JsValue::UNDEFINED`] as a Serde default value.
fn undefined() -> JsValue {
    JsValue::UNDEFINED
}

/// The raw accessibility tree returned by Chrome.
#[derive(Deserialize)]
struct AxTree {
    nodes: Vec<AxNode>,
}

/// A node in the raw accessibility tree returned by Chrome.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AxNode {
    /// A unique accessibility-domain-specific node identifier.
    node_id: String,
    /// Whether or not this node has been deemed uninteresting by the browser.
    ignored: bool,
    /// The accessibility role of the node.
    role: Option<AxValue>,
    /// The name associated with the node.
    name: Option<AxValue>,
    /// Any accessibility description of the node.
    description: Option<AxValue>,
    /// The "value" (i.e. state) of the node.
    value: Option<AxValue>,
    /// Arbitrary properties the node may have.
    properties: Option<Vec<AxProperty>>,
    /// The unique identifier of the parent in the accessibility tree.
    parent_id: Option<String>,
    /// The backend node identifier used across other debugger APIs.
    #[serde(rename = "backendDOMNodeId")]
    backend_dom_node_id: Option<u32>,
}
impl AxNode {
    /// Parses this raw node into an intermediate representation. This also designates which
    /// nodes should be filtered after the nested tree has been reconstituted.
    fn into_intermediate(self) -> IntermediateNode {
        let role = self.role.map(|val| val.value.as_string()).flatten();
        IntermediateNode {
            remove: !self.properties.as_ref().is_some_and(|props| {
                props
                    .iter()
                    .any(|prop| prop.name == "focusable" && prop.value.value == JsValue::TRUE)
            }) || self.ignored
                || role.as_ref().is_some_and(|r| r == "RootWebArea"),
            // Only ones that need the default will be later filtered out
            dom_id: self.backend_dom_node_id.unwrap_or(0),
            name: self.name.map(|val| val.value.as_string()).flatten(),
            description: self.description.map(|val| val.value.as_string()).flatten(),
            role,
            value: self.value.map(|val| js_value_to_string(val.value)),
            properties: self
                .properties
                .map(|props| {
                    props
                        .into_iter()
                        // All the ones we'll keep are focused, there's no point in preserving
                        // this
                        .filter(|prop| prop.name != "focusable")
                        .map(|prop| (prop.name, js_value_to_string(prop.value.value)))
                        .collect()
                })
                .unwrap_or_default(),
            id: self.node_id,
            parent_id: self.parent_id,
            children: Vec::new(),
        }
    }
}

/// The generic representation of a dynamically typed value used by Chrome's accessibility
/// debugger API.
#[derive(Deserialize)]
struct AxValue {
    /// The type of the value.
    #[serde(rename = "type")]
    ty: String,
    /// The value itself, which we interpret using JS semantics (lower-cost in a web extension,
    /// which puts everything through JS anyway).
    #[serde(default = "undefined")]
    #[serde(with = "serde_wasm_bindgen::preserve")]
    value: JsValue,
}

/// The generic representation of an accessibility property.
#[derive(Deserialize)]
struct AxProperty {
    /// The property's name.
    name: String,
    /// The property's value.
    value: AxValue,
}

/// An intermediate parsed representation of the Chrome accessibility tree. This stil has some
/// properties redundant in the final node, which we use for reconstituting the nested structure
/// of the tree.
struct IntermediateNode {
    id: String,
    parent_id: Option<String>,
    dom_id: u32,
    name: Option<String>,
    description: Option<String>,
    role: Option<String>,
    value: Option<String>,
    properties: HashMap<String, String>,
    // These are implanted by the parent ID property
    children: Vec<IntermediateNode>,
    // Whether or not this node should be removed from the tree structure
    // as irrelevant
    remove: bool,
}
impl IntermediateNode {
    /// Converts this intermediate node into a proper Voxurf [`Node`]. This removes accessibility-related
    /// IDs.
    fn into_final(self) -> Node<u32> {
        Node {
            selector: self.dom_id,
            name: self.name,
            description: self.description,
            role: self.role,
            state: self.value,
            properties: self.properties,
            children: self.children.into_iter().map(|n| n.into_final()).collect(),
        }
    }
}

/// Gets the accessibility tree and filters it, preparing it in a format
/// digestible by an LLM.
pub(crate) async fn get_ax_tree(tab_id: u32) -> Result<Tree<u32>, WeiError> {
    let tree = get_raw_ax_tree(tab_id).await;
    let tree: AxTree = serde_wasm_bindgen::from_value(tree)
        .map_err(|err| WeiError::AxTreeParseFailed { source: err })?;
    // Filter and parse the tree into our own `Node` struct; this will be "flat"
    // in that each node will have references to its parents and so forth
    let mut flat_tree: Vec<Option<IntermediateNode>> = tree
        .nodes
        .into_iter()
        // We'll change these to `None` as we go
        .map(|raw| Some(raw.into_intermediate()))
        .collect();

    // The actual tree structure
    let mut tree = Vec::new();
    let mut dom_ids = Vec::new();
    // A map of IDs to locations within `tree` (gradually populated)
    let mut nodes_ref_map: HashMap<String, Vec<usize>> = HashMap::new();
    // Keep iterating back through again and again until there's nothing left
    let mut iters = 0;
    while flat_tree.iter().any(|n| n.is_some()) && iters < MAX_ITERS {
        for node_opt in flat_tree.iter_mut() {
            if let Some(node) = node_opt.take() {
                if let Some(parent_id) = &node.parent_id {
                    if let Some(parent_loc_ref) = nodes_ref_map.get_mut(parent_id) {
                        // Recursively gets the children of the element with the provided location
                        // vector. This returns the children so we can abstract over returning the
                        // entire tree if necessary, as in the case of top-level hoisting.
                        fn get_tree_children(
                            elems: &mut Vec<Node<u32>>,
                            mut loc: Vec<usize>,
                        ) -> &mut Vec<Node<u32>> {
                            if loc.is_empty() {
                                return elems;
                            }

                            let first_loc = loc.remove(0);
                            if loc.is_empty() {
                                // This is safe because we're working with predefined indices
                                &mut elems.get_mut(first_loc).unwrap().children
                            } else {
                                get_tree_children(
                                    &mut elems.get_mut(first_loc).unwrap().children,
                                    loc,
                                )
                            }
                        }
                        // This is the vector we're inserting our child into
                        let parent_children = get_tree_children(&mut tree, parent_loc_ref.clone());

                        let id = node.id.clone();
                        // If this node should be removed, then we'll insert children
                        // in the same place this was going to be inserted. Otherwise,
                        // they'll be inserted within this node.
                        let child_insertion_loc = if node.remove {
                            parent_loc_ref.clone()
                        } else {
                            dom_ids.push(node.dom_id.clone());
                            parent_children.push(node.into_final());
                            let mut self_loc = parent_loc_ref.clone();
                            self_loc.push(parent_children.len() - 1);
                            self_loc
                        };
                        nodes_ref_map.insert(id, child_insertion_loc);
                    } else {
                        // Parent isn't in the tree yet, leave this node behind;
                        // we'll get it on the next pass
                        *node_opt = Some(node);
                    }
                } else {
                    let id = node.id.clone();
                    // If this node should be removed, then we'll insert children at
                    // the top-level, otherwise within this node in the tree
                    let child_insertion_loc = if node.remove {
                        Vec::new()
                    } else {
                        dom_ids.push(node.dom_id.clone());
                        tree.push(node.into_final());
                        vec![tree.len() - 1]
                    };
                    nodes_ref_map.insert(id, child_insertion_loc);
                }
            }
        }
        iters += 1;
    }
    if flat_tree.iter().any(|n| n.is_some()) {
        return Err(WeiError::TreeReconstitutionTimeout {
            num_iters: MAX_ITERS,
        });
    }

    Ok(Tree {
        roots: tree,
        selectors: dom_ids,
    })
}
