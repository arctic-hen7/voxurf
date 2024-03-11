use crate::{
    compute_tree::get_ax_tree,
    glue::{
        attach_debugger, click_element, detach_debugger, dom_disable, dom_enable,
        dom_id_to_selector, fill_element, get_tab_id, log,
    },
};
use thiserror::Error;
use voxurf::{Interface, Tree};

/// A Voxurf interface for a web extension interacting with the host page the user's browser
/// is visiting. This can, more generally, control the entire browser. Note that this is designed
/// for Chromium-based browsers *only*, and is unlikely to work correctly in others, as it makes
/// a heavy dependence on the Chrome debugger API.
///
/// To use this, the following privileges are needed: `debugger`, `activeTab`, and `scripting`.
pub struct WebExtensionInterface {
    /// The unique identifier of the tab we're currently in. This requires the `activeTab`
    /// permission.
    tab_id: u32,
}
impl WebExtensionInterface {
    /// Creates a new web extension interface for the currently active tab.
    pub async fn new() -> Self {
        // As wacky as this is, it is guaranteed to work
        let tab_id = get_tab_id().await.as_f64().unwrap() as u32;
        Self { tab_id }
    }
    /// Attaches the debugger and enables the DOM API. This must be called before any command
    /// execution is performed with this interface.
    pub async fn pre_execute(&self) {
        attach_debugger(self.tab_id).await;
        dom_enable(self.tab_id).await;
    }
    /// Disables the DOM API and detaches the debugger. This must be called after any command
    /// execution is performed with this interface.
    pub async fn post_execute(&self) {
        dom_disable(self.tab_id).await;
        detach_debugger(self.tab_id).await;
    }
}
impl Interface for WebExtensionInterface {
    type Error = WebExtensionInterfaceError;
    // We'll resolve backend DOM node IDs into CSS selectors on-demand
    type Selector = u32;

    async fn primary_click_element(&self, selector: &Self::Selector) -> Result<(), Self::Error> {
        let css_selector = dom_id_to_selector(*selector, self.tab_id)
            .await
            .as_string()
            .unwrap();
        click_element(self.tab_id, &css_selector).await;
        Ok(())
    }
    async fn type_into_element(
        &self,
        selector: &Self::Selector,
        text: &str,
    ) -> Result<(), Self::Error> {
        let css_selector = dom_id_to_selector(*selector, self.tab_id)
            .await
            .as_string()
            .unwrap();
        fill_element(self.tab_id, &css_selector, text).await;
        Ok(())
    }
    async fn compute_tree(&self) -> Result<Tree<Self::Selector>, Self::Error> {
        get_ax_tree(self.tab_id).await
    }
}

#[derive(Error, Debug)]
pub enum WebExtensionInterfaceError {
    #[error(
        "failed to reconstitute nested accessibility tree structure after {num_iters} iterations"
    )]
    TreeReconstitutionTimeout { num_iters: usize },
    #[error("failed to parse raw accessibility tree")]
    AxTreeParseFailed {
        #[source]
        source: serde_wasm_bindgen::Error,
    },
}
// So I don't go mad...
pub(crate) type WeiError = WebExtensionInterfaceError;

// /// Executes the given command against the page's accessibility tree, calling out
// /// to an LLM for processing.
// ///
// /// This returns a description of all the actions the LLM took.
// pub async fn execute_command(command: &str) -> String {
//     let mut previous_actions = Vec::new();

//     let mut num_trips = 0;
//     let mut action_complete = false;
//     while num_trips < MAX_TRIPS {
//         // Attach the debugger to the current tab
//         let tab_id = get_tab_id().await.as_f64().unwrap() as u32;
//         attach_debugger(tab_id).await;

//         let (tree, dom_ids) = get_ax_tree(tab_id).await;
//         // Map the DOM IDs to CSS query selectors; this will work for all our selected nodes
//         // because focusable nodes are guaranteed to exist on the page (apart from the root,
//         // which we've filtered out).
//         //
//         // We do this *after* we know the page to be stable to avoid unnecessary work.
//         dom_enable(tab_id).await;
//         let mut dom_id_map = HashMap::new();
//         for dom_id in dom_ids {
//             let selector = dom_id_to_selector(dom_id, tab_id)
//                 .await
//                 .as_string()
//                 .unwrap();
//             dom_id_map.insert(dom_id, selector);
//         }
//         dom_disable(tab_id).await;
//         detach_debugger(tab_id).await;

//         // Construct the prompt for the LLM
//         let mut tree_str = String::new();
//         for node in tree {
//             tree_str.push_str(&node.to_string(0));
//             tree_str.push('\n');
//         }
//         let tree_str = tree_str.trim();
//         let prompt = PROMPT
//             .replace("{{ tree_text }}", &tree_str)
//             .replace("{{ user_command }}", command)
//             .replace(
//                 "{{ previous_actions }}",
//                 &if !previous_actions.is_empty() {
//                     format!("- {}", previous_actions.join("\n- "))
//                 } else {
//                     String::new()
//                 },
//             );
//         // Cut the prompt off before the previous actions if we have none
//         let prompt = if previous_actions.is_empty() {
//             prompt.split("{{ previous_actions_cutoff }}").next().unwrap().trim().to_string()
//         } else {
//             prompt.replace("{{ previous_actions_cutoff }}", "")
//         };
//         #[cfg(debug_assertions)]
//         log(&prompt);

//         // Send the prompt to the LLM, parsing from it the actions to be taken
//         let actions = get_llm_response(&prompt).await;
//         log(&format!("{:#?}", &actions));
//         actions.execute(&dom_id_map, tab_id).await;
//         // Add this *after* we've executed the actions so only the successful ones are described
//         // (partial group executions will lead to issues here, but that's okay)
//         previous_actions.push(actions.description);

//         if actions.complete {
//             action_complete = true;
//             break;
//         } else {
//             num_trips += 1;
//         }
//     }

//     if !action_complete {
//         panic!("failed to complete action in {} trips", MAX_TRIPS);
//     } else {
//         // We handily get a description of actions for free!
//         previous_actions.join("\n")
//     }
// }
