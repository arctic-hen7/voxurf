use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::openai::OpenAiApi;

/// Maximum number of iterations in tree reconstitution. This prevents infinite loops.
const MAX_ITERS: usize = 50;
/// Maximum number of round trips to be made with the LLM.
const MAX_TRIPS: usize = 2;

/// The number of milliseconds to wait at a maximum for the page layout to stabilise after
/// an interaction. If this is reached, the user's command will be aborted in a possibly
/// broken state.
const STABILITY_TIMEOUT: usize = 10_000;
/// A number of milliseconds after which future accessibility trees will not be observed.
/// Once a change is observed from the old tree, we will wait this long for a new change,
/// and, if one is not observed, we'll consider the page stable.
const STABILITY_THRESHOLD: usize = 250;

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

#[derive(Serialize, PartialEq, Eq)]
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
    fn to_string(&self, indent_level: usize) -> String {
        format!(
            "{tabs}- [{id}] \"{name}\"{role}{desc}{props}{value}{children}",
            tabs = "\t".repeat(indent_level),
            id = self.dom_id,
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
            value = if let Some(val) = &self.value {
                format!(" with value {val}")
            } else {
                String::new()
            },
            children = if !self.children.is_empty() {
                let mut children_str = String::new();
                for child in &self.children {
                    children_str.push('\n');
                    children_str.push_str(&child.to_string(indent_level + 1));
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
    async fn click_element(tab_id: u32, selector: &str);
    async fn fill_element(tab_id: u32, selector: &str, text: &str);

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
/// digestible by an LLM. This also returns a list of the backend DOM node
/// IDs referenced in the tree.
async fn get_ax_tree(tab_id: u32) -> (Vec<Node>, Vec<u32>) {
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

    #[cfg(debug_assertions)]
    log(&format!(
        "Total nodes {} reduced to {} relevant nodes",
        nodes_ref_map.len(),
        num_inserted
    ));

    (tree, dom_ids)
}

/// Executes the given command against the page's accessibility tree, calling out
/// to an LLM for processing.
///
/// This returns a description of all the actions the LLM took.
pub async fn execute_command(command: &str) -> String {
    let mut previous_actions = Vec::new();

    let mut num_trips = 0;
    let mut action_complete = false;
    while num_trips < MAX_TRIPS {
        // Attach the debugger to the current tab
        let tab_id = get_tab_id().await.as_f64().unwrap() as u32;
        attach_debugger(tab_id).await;

        let (tree, dom_ids) = get_ax_tree(tab_id).await;
        // Map the DOM IDs to CSS query selectors; this will work for all our selected nodes
        // because focusable nodes are guaranteed to exist on the page (apart from the root,
        // which we've filtered out).
        //
        // We do this *after* we know the page to be stable to avoid unnecessary work.
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
        detach_debugger(tab_id).await;

        // Construct the prompt for the LLM
        let mut tree_str = String::new();
        for node in tree {
            tree_str.push_str(&node.to_string(0));
            tree_str.push('\n');
        }
        let tree_str = tree_str.trim();
        let prompt = PROMPT
            .replace("{{ tree_text }}", &tree_str)
            .replace("{{ user_command }}", command)
            .replace(
                "{{ previous_actions }}",
                &if !previous_actions.is_empty() {
                    format!("- {}", previous_actions.join("\n- "))
                } else {
                    String::new()
                },
            );
        // Cut the prompt off before the previous actions if we have none
        let prompt = if previous_actions.is_empty() {
            prompt.split("{{ previous_actions_cutoff }}").next().unwrap().trim().to_string()
        } else {
            prompt.replace("{{ previous_actions_cutoff }}", "")
        };
        #[cfg(debug_assertions)]
        log(&prompt);

        // Send the prompt to the LLM, parsing from it the actions to be taken
        let actions = get_llm_response(&prompt).await;
        log(&format!("{:#?}", &actions));
        actions.execute(&dom_id_map, tab_id).await;
        // Add this *after* we've executed the actions so only the successful ones are described
        // (partial group executions will lead to issues here, but that's okay)
        previous_actions.push(actions.description);

        if actions.complete {
            action_complete = true;
            break;
        } else {
            num_trips += 1;
        }
    }

    if !action_complete {
        panic!("failed to complete action in {} trips", MAX_TRIPS);
    } else {
        // We handily get a description of actions for free!
        previous_actions.join("\n")
    }
}

/// Sends the given prompt to the LLM and parses its response into a group of actions
/// to be taken on the page.
async fn get_llm_response(prompt: &str) -> ActionGroup {
    let response = OpenAiApi::call(prompt).await.unwrap();
    log(&response);
    // Define a regular expression for matching JS code fence
    let re = Regex::new(r"```(.*)\n([\s\S]+?)\n```").unwrap();

    // Attempt to find a match in the response
    if let Some(captures) = re.captures(&response) {
        let mut group = ActionGroup::default();

        let mut actions = captures.get(2).unwrap().as_str().trim().lines().peekable();
        while let Some(action_str) = actions.next() {
            // No action has more than three parameters
            let mut parts = action_str.splitn(3, ' ');
            // Guaranteed to be at least one part
            let action_ty = parts.next().unwrap();

            if action_ty == "CLICK" {
                let id = parts
                    .next()
                    .expect("expected id in click stage");
                let id = id.strip_prefix('[').unwrap_or(id).strip_suffix(']').unwrap_or(id);
                let id = id.parse().expect("expected integer id");

                group.add(Action::Click { id })
            } else if action_ty == "FILL" {
                let id = parts.next().expect("expected id in fill stage");
                let id = id.strip_prefix('[').unwrap_or(id).strip_suffix(']').unwrap_or(id);
                let id = id.parse().expect("expected integer id");
                let text = parts.next().expect("expected text in fill stage");
                let text = text.strip_prefix('\'').map(|t| t.strip_suffix('\'')).flatten().expect("expected text in fill stage to be single-quoted");

                group.add(Action::Fill { id, text: text.to_string() })
            } else if action_ty == "WAIT" {
                // We've reached a waitpoint, ignore everything from here on. We can't
                // stop the LLM from generating this without it getting panicky about
                // whether or not it needs to do anything more, so we let it generate
                // garbage with ID placeholders and then calmly ignore it, telling it
                // later what it's already done and letting it generate sensible code
                // once it's seen the updated state of the page.
                //
                // Importantly, we should only register this as a legitimate wait-point
                // if the LLM has *tried* to do something afterward. If it's just being
                // cautious, we shouldn't do anything.
                if actions.peek().is_some() {
                    // The parameter to the "WAIT" action is a description of everything
                    // done so far.
                    let description = parts.collect::<Vec<_>>().join(" ");
                    if description.is_empty() {
                        panic!("expected description in wait stage");
                    }
                    group.description = description.trim().to_string();
                    group.complete = false;
                    break;
                }
            } else if action_ty == "FINISH" {
                let description = parts.collect::<Vec<_>>().join(" ");
                if description.is_empty() {
                    panic!("expected description in wait stage");
                }
                group.description = description.trim().to_string();
                group.complete = true;

                if actions.peek().is_some() {
                    panic!("found stages after finish")
                }
            }
        }

        group
    } else {
        log(&response);
        panic!("invalid response from llm");
    }
}

/// An action we should take on the page.
#[derive(Debug)]
enum Action {
    /// Click an element. This will be done using the `.click()` method.
    Click {
        /// The unique ID of the element to click.
        id: u32,
    },
    /// Fill in an input. This will be done using user-style keyboard events to avoid
    /// interfering with reactivity frameworks.
    Fill {
        /// The unique ID of the element to fill out.
        id: u32,
        /// The text to fill it out with.
        text: String,
    },
}
impl Action {
    async fn execute(&self, dom_id_map: &HashMap<u32, String>, tab_id: u32) {
        match &self {
            Self::Click { id } => {
                let query_selector = dom_id_map.get(id).expect("found invalid id");
                click_element(tab_id, &query_selector).await;
            },
            Self::Fill { id, text } => {
                let query_selector = dom_id_map.get(id).expect("found invalid id");
                fill_element(tab_id, &query_selector, text).await;
            },
        }
    }
}

/// A group of actions, with a possible description of them.
#[derive(Debug)]
struct ActionGroup {
    /// The actions in this group.
    actions: Vec<Action>,
    /// The description of the actions taken.
    description: String,
    /// Whether or not we're done with the whole action yet.
    complete: bool,
}
impl Default for ActionGroup {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
            description: String::new(),
            // We'll assume we're done in case we don't get `FINISH`
            complete: true,
        }
    }
}
impl ActionGroup {
    fn add(&mut self, action: Action) {
        self.actions.push(action);
    }
    async fn execute(&self, dom_id_map: &HashMap<u32, String>, tab_id: u32) {
        for action in &self.actions {
            action.execute(dom_id_map, tab_id).await;
        }
    }
}
