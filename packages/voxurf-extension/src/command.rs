use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::openai::OpenAiApi;

/// Maximum number of iterations in tree reconstitution. This prevents infinite loops.
const MAX_ITERS: usize = 50;
/// Maximum number of round trips to be made with the LLM.
const MAX_TRIPS: usize = 5;

/// Prompt for the LLM
static PROMPT: &str = include_str!("../prompt.txt");

fn undefined() -> JsValueSerde {
    JsValueSerde(JsValue::UNDEFINED)
}

fn js_value_to_string(val: JsValue) -> String {
    let wrapped = format!("{:?}", val);
    let output = wrapped.strip_prefix("JsValue(").unwrap();
    let output = output.strip_suffix(')').unwrap();
    let output = output.strip_prefix('"').unwrap_or(output);
    let output = output.strip_suffix('"').unwrap_or(output);

    output.to_string()
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct JsValueSerde(#[serde(with = "serde_wasm_bindgen::preserve")] JsValue);

#[derive(Deserialize)]
struct AxTree {
    nodes: Vec<AxNode>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AxNode {
    node_id: String,
    ignored: bool,
    role: Option<AxValue>,
    name: Option<AxValue>,
    description: Option<AxValue>,
    value: Option<AxValue>,
    properties: Option<Vec<AxProperty>>,
    parent_id: Option<String>,
    #[serde(rename = "backendDOMNodeId")]
    backend_dom_node_id: Option<u32>,
}
impl AxNode {
    fn into_intermediate(self) -> IntermediateNode {
        let role = self.role.map(|val| val.value.0.as_string()).flatten();
        IntermediateNode {
            remove: !self.properties.as_ref().is_some_and(|props| {
                props
                    .iter()
                    .any(|prop| prop.name == "focusable" && prop.value.value.0 == JsValue::TRUE)
            }) || self.ignored
                || role.as_ref().is_some_and(|r| r == "RootWebArea"),
            // Only ones that need the default will be later filtered out
            dom_id: self.backend_dom_node_id.unwrap_or(0),
            name: self.name.map(|val| val.value.0.as_string()).flatten(),
            description: self
                .description
                .map(|val| val.value.0.as_string())
                .flatten(),
            role,
            value: self.value.map(|val| js_value_to_string(val.value.0)),
            properties: self
                .properties
                .map(|props| {
                    props
                        .into_iter()
                        // All the ones we'll keep are focused, there's no point in preserving
                        // this
                        .filter(|prop| prop.name != "focusable")
                        .map(|prop| (prop.name, js_value_to_string(prop.value.value.0)))
                        .collect()
                })
                .unwrap_or_default(),
            id: self.node_id,
            parent_id: self.parent_id,
            children: Vec::new(),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct AxValue {
    #[serde(rename = "type")]
    ty: String,
    #[serde(default = "undefined")]
    value: JsValueSerde,
}

#[derive(Deserialize, Serialize)]
struct AxProperty {
    name: String,
    value: AxValue,
}

// This has information needed for nested tree reconstitution
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
    fn into_final(self) -> Node {
        Node {
            dom_id: self.dom_id,
            name: self.name,
            description: self.description,
            role: self.role,
            value: self.value,
            properties: self.properties,
            children: self.children.into_iter().map(|n| n.into_final()).collect(),
        }
    }
}

#[derive(Serialize)]
struct Node {
    dom_id: u32,
    name: Option<String>,
    description: Option<String>,
    role: Option<String>,
    value: Option<String>,
    properties: HashMap<String, String>,
    children: Vec<Node>,
}
impl Node {
    /// Converts the node into a string suitable for LLM ingestion. This deliberately elides
    /// irrelevant information to save on tokens.
    fn into_string(self, indent_level: usize) -> String {
        format!(
            "{tabs}- [{id}] \"{name}\"{role}{desc}{props}{value}{children}",
            tabs = "\t".repeat(indent_level),
            id = self.dom_id,
            name = self.name.unwrap_or("<null>".to_string()),
            role = if let Some(role) = self.role {
                format!(" ({role})")
            } else {
                String::new()
            },
            desc = if let Some(desc) = self.description {
                format!(" ({desc})")
            } else {
                String::new()
            },
            props = if !self.properties.is_empty() {
                let mut s = " {".to_string();
                for (key, val) in self.properties {
                    s.push_str(&key);
                    s.push_str(": ");
                    s.push_str(&val);
                    s.push_str(", ")
                }
                format!("{}}}", s.strip_suffix(", ").unwrap())
            } else {
                String::new()
            },
            value = if let Some(val) = self.value {
                format!(" with value {val}")
            } else {
                String::new()
            },
            children = if !self.children.is_empty() {
                let mut children_str = String::new();
                for child in self.children {
                    children_str.push('\n');
                    children_str.push_str(&child.into_string(indent_level + 1));
                }
                children_str
            } else {
                String::new()
            }
        )
    }
}

#[wasm_bindgen(module = "/src/glue.js")]
extern "C" {
    async fn attach_debugger(tab_id: u32);
    async fn detach_debugger(tab_id: u32);
    async fn get_tab_id() -> JsValue;
    async fn get_raw_ax_tree(tab_id: u32) -> JsValue;
    async fn execute_js(tab_id: u32, script: &str);
    async fn dom_enable(tab_id: u32);
    async fn dom_disable(tab_id: u32);
    async fn dom_id_to_selector(id: u32, tab_id: u32) -> JsValue;
}
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(v: &str);
}

/// Gets the accessibility tree and filters it, preparing it in a format
/// digestible by an LLM. This also returns a map of DOM IDs to CSS selectors.
async fn get_ax_tree(tab_id: u32) -> (Vec<Node>, HashMap<u32, String>) {
    let tree = get_raw_ax_tree(tab_id).await;
    let tree: AxTree = serde_wasm_bindgen::from_value(tree).unwrap();
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
    let mut num_inserted = 0;
    while flat_tree.iter().any(|n| n.is_some()) && iters < MAX_ITERS {
        for node_opt in flat_tree.iter_mut() {
            if let Some(node) = node_opt.take() {
                if let Some(parent_id) = &node.parent_id {
                    if let Some(parent_loc_ref) = nodes_ref_map.get_mut(parent_id) {
                        // Recursively gets the children of the element with the provided location
                        // vector. This returns the children so we can abstract over returning the
                        // entire tree if necessary, as in the case of top-level hoisting.
                        fn get_tree_children(
                            elems: &mut Vec<Node>,
                            mut loc: Vec<usize>,
                        ) -> &mut Vec<Node> {
                            if loc.is_empty() {
                                return elems;
                            }

                            let first_loc = loc.remove(0);
                            if loc.is_empty() {
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
                            num_inserted += 1;
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
                        num_inserted += 1;
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
        panic!(
            "failed to reconstitute nested accessibility tree after {} iterations",
            MAX_ITERS
        );
    }

    // Map the DOM IDs to CSS query selectors; this will work for all our selected nodes
    // because focusable nodes are guaranteed to exist on the page (apart from the root,
    // which we've filtered out)
    dom_enable(tab_id).await;
    let mut dom_id_map = HashMap::new();
    for dom_id in dom_ids {
        let selector = dom_id_to_selector(dom_id, tab_id)
            .await
            .as_string()
            .unwrap();
        dom_id_map.insert(dom_id, selector);
    }
    dom_disable(tab_id).await;

    #[cfg(debug_assertions)]
    log(&format!(
        "Total nodes {} reduced to {} relevant nodes",
        nodes_ref_map.len(),
        num_inserted
    ));

    (tree, dom_id_map)
}

/// Executes the given command against the page's accessibility tree, calling out
/// to an LLM for processing.
pub async fn execute_command(command: &str) {
    let mut previous_actions = Vec::new();

    let mut num_trips = 0;
    let mut action_complete = false;
    while num_trips < MAX_TRIPS {
        // Attach the debugger to the current tab
        let tab_id = get_tab_id().await.as_f64().unwrap() as u32;
        attach_debugger(tab_id).await;

        let (tree, dom_id_map) = get_ax_tree(tab_id).await;

        // Construct the prompt for the LLM
        let mut tree_str = String::new();
        for node in tree {
            tree_str.push_str(&node.into_string(0));
            tree_str.push('\n');
        }
        let tree_str = tree_str.trim();
        let prompt = PROMPT
            .replace("{{ tree_json }}", &tree_str)
            .replace("{{ user_command }}", command)
            .replace(
                "{{ previous_actions }}",
                &if !previous_actions.is_empty() {
                    format!("- {}", previous_actions.join("\n- "))
                } else {
                    String::new()
                },
            );
        #[cfg(debug_assertions)]
        log(&prompt);

        // Send the prompt to the LLM, extracting its description of the actions it
        // has taken and the script that wil ltake those actions
        let (action_description, response_script) = get_llm_response(prompt).await;

        // Resolve DOM node IDs to CSS query selectors in the script
        let response_script = Regex::new(r#"selectorFromId\((\d+)\)"#)
            .unwrap()
            .replace_all(&response_script, |caps: &Captures| {
                // If this fails, the LLM is referencing a nonexistent node
                format!("'{}'", dom_id_map.get(&caps[1].parse().unwrap()).unwrap())
            });
        #[cfg(debug_assertions)]
        log(&response_script);

        // This uses the debugger API
        execute_js(tab_id, &response_script).await;
        // Detach the debugger immediately so the extension works if the user presses
        // the button again
        detach_debugger(tab_id).await;

        // If the LLM thinks it's done, finish, otherwise keep going
        if action_description.contains("CONTINUE") {
            // We'll add the action description to our list, and do everything again
            previous_actions.push(action_description);
            num_trips += 1;
        } else {
            action_complete = true;
            break;
        }
    }

    if !action_complete {
        panic!("failed to complete action in {} trips", MAX_TRIPS);
    }
}

/// Sends the given prompt to the LLM and breaks its response into a description of the
/// action it has taken to further the user's command and the script that will take that
/// action.
async fn get_llm_response(prompt: String) -> (String, String) {
    let response = OpenAiApi::call(&prompt).await.unwrap();
    // Define a regular expression for matching JS code fence
    let re = Regex::new(r"```js\n([\s\S]+?)\n```").unwrap();

    // Attempt to find a match in the response
    if let Some(captures) = re.captures(&response) {
        // Extract code and description
        let code = captures.get(1).unwrap().as_str();
        let remaining_response = &response[captures.get(0).unwrap().end()..].trim();

        (remaining_response.to_string(), code.to_string())
    } else {
        panic!("invalid response from llm");
    }
}
