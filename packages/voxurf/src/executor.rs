use crate::{
    error::{ActionParseError, ExecutionError},
    interface::Interface,
    model::Model,
    sleep,
    tree::Tree,
};
use std::collections::HashMap;

/// The prompt used for the model. This is universal across all models, and optimised for the
/// GPT family.
static PROMPT: &str = include_str!("prompt.txt");

/// A system for executing natural-language commands against an interface.
pub struct Executor<'i, 'm, I: Interface, M: Model> {
    /// The interface against which the actions will be executed.
    interface: &'i I,
    /// The AI model which will be used for processing the user's command and converting it
    /// into a series of actions.
    model: &'m M,
    /// Options for the executor.
    opts: ExecutorOpts,

    /// A map of lightweight internal element IDs to heavyweight interface selectors. This
    /// is done to simplify the API and minimise the load of passing full-blown selectors to
    /// the model (which usually works better with integer IDs).
    id_map: HashMap<u32, I::Selector>,
}
impl<'i, 'm, I: Interface, M: Model> Executor<'i, 'm, I, M> {
    /// Creates a new executor against the given interface, using the given model and options.
    pub fn new(interface: &'i I, model: &'m M, opts: ExecutorOpts) -> Self {
        debug_assert!(
            opts.stability_threshold_ms > opts.tree_poll_interval_ms,
            "stability threshold must be greater than tree poll interval"
        );
        debug_assert!(
            opts.stability_timeout_ms > opts.tree_poll_interval_ms,
            "stability timeout must be greater than tree poll interval"
        );

        Self {
            interface,
            model,
            opts,
            id_map: HashMap::new(),
        }
    }

    /// Executes the given natural-language user command against this interface. This will
    /// interact with an AI model, potentially over multiple steps, to perform arbitrarily
    /// complex multi-step actions in generic user interfaces.
    ///
    /// This will return a multiline description of the steps involved with executing this
    /// command, essentially a description (from the model) of everything it did. This may
    /// or may not be accurate!
    pub async fn execute_command(&self, command: &str) -> Result<String, ExecutionError> {
        // This stores the last recorded (filtered and parsed) state of the element tree of
        // the page, for comparison to determine when it's changed. This will be `Some(_)`
        // for everything but the first trip.
        let mut last_tree = None;
        // This stores descriptions of the actions we've taken in each trip. A trip is ended
        // by either a `WAIT` or `FINISH` instruction from the model, both of which have
        // descriptions associated with them of the actions that have just been completed.
        // Putting all these together at the end gives a full description of everything that's
        // been done to complete the user's command.
        let mut previous_actions = Vec::new();
        // This just checks if we're completed the action by the end of the loop
        let mut action_complete = false;

        let mut num_trips = 0;
        while num_trips < self.opts.max_round_trips {
            // If this is the first run, we can get the tree as it is and assume it's stable.
            // If this is a later run, we should wait for the tree to meet basic stability
            // heuristics (likely the last action changed something, otherwise we would've
            // been able to do it all in one go).
            if num_trips == 0 {
                let tree = self
                    .interface
                    .compute_tree()
                    .await
                    .map_err(|err| ExecutionError::InterfaceError { source: err.into() })?;
                last_tree = Some(tree);
            } else {
                // The last tree is guaranteed to be set. This method puts the new tree in
                // `last_tree`, so the semantics of this whole block work by putting our
                // result in `last_tree` for convenience.
                self.get_stable_tree(last_tree.as_mut().unwrap()).await?;
            };
            let tree = last_tree.as_ref().unwrap();

            // Construct the prompt to send to the model
            let mut tree_str = String::new();
            for node in &tree.roots {
                tree_str.push_str(&node.to_string(0));
                tree_str.push('\n');
            }
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
                prompt
                    .split("{{ previous_actions_cutoff }}")
                    .next()
                    .unwrap()
                    .trim()
                    .to_string()
            } else {
                prompt.replace("{{ previous_actions_cutoff }}", "")
            };

            // Get the model's response and parse it into a series of actions
            let response_str = self
                .model
                .prompt(&prompt)
                .await
                .map_err(|err| ExecutionError::ModelError { source: err.into() })?;
            let mut instructions = Vec::new();
            let mut in_code_fence = false;
            for line in response_str.lines() {
                if in_code_fence && line == "```" {
                    // We've reached the end of the code fence, nothing else is relevant
                    break;
                } else if in_code_fence {
                    instructions.push(line);
                } else if line == "```" {
                    in_code_fence = true;
                }
            }

            // Parse the actions into a group of actions
            let actions = ActionGroup::parse_action_strings(&instructions)?;
            for action in actions.actions {
                self.execute_action(action).await?;
            }
            previous_actions.push(actions.description);

            // If we're not complete, we'll go on to the next trip and wait for an update to the tree
            if actions.complete {
                action_complete = true;
                break;
            } else {
                num_trips += 1;
            }
        }

        if !action_complete {
            Err(ExecutionError::CommandNotFinished { num_trips })
        } else {
            Ok(previous_actions.join("\n"))
        }
    }

    /// Executes the given action with this executor. This might fail if given an action with an
    /// invalid internal ID, which would be a case of model hallucination.
    async fn execute_action(&self, action: Action) -> Result<(), ExecutionError> {
        match action {
            Action::Click { id } => {
                let selector = self
                    .id_map
                    .get(&id)
                    .ok_or(ExecutionError::IdNotFound { id })?;
                self.interface
                    .primary_click_element(selector)
                    .await
                    .map_err(|err| ExecutionError::InterfaceError { source: err.into() })?;
            }
            Action::Type { id, text } => {
                let selector = self
                    .id_map
                    .get(&id)
                    .ok_or(ExecutionError::IdNotFound { id })?;
                self.interface
                    .type_into_element(selector, &text)
                    .await
                    .map_err(|err| ExecutionError::InterfaceError { source: err.into() })?;
            }
        }

        Ok(())
    }

    /// Polls the tree continuously, delaying for a specified number of milliseconds
    /// each time to give the interface time to update the element tree (some may
    /// need this). If we find it doesn't change for a set number of iterations,
    /// we'll consider it stable and move on. If it doesn't stabilise after another
    /// number of iterations, we'll abort.
    ///
    /// The one exception to this paradigm is that we'll wait for the full timeout
    /// duration if we don't observe any change in the tree. The fact that we've hit
    /// a waitpoint the model has asked for means there should be a change, and if
    /// there's not, the model will likely complete the same actions in a loop,
    /// leading to unnecessary usage and nonsensical actions. Better to fail quickly
    /// if the model's expectations are not met.
    ///
    /// Somewhat unorthodox for Rust, this will put the new tree in `last_tree`,
    /// handling the conditions that the tree didn't stabilise or that it didn't change.
    async fn get_stable_tree(&self, last_tree: &mut Tree) -> Result<(), ExecutionError> {
        // Calculating this like so lets the user specify everything in milliseconds,
        // but we only have to use one timer
        let num_iters_stable = self.opts.stability_threshold_ms / self.opts.tree_poll_interval_ms;
        let num_iters_timeout = self.opts.stability_timeout_ms / self.opts.tree_poll_interval_ms;

        // This tracks whether *any* change has occurred o we know if we've "stabilised"
        // on the original value from the last trip
        let mut change_recorded = false;

        let mut total_iters = 0;
        let mut stable_iters = 0;
        while total_iters < num_iters_timeout && stable_iters < num_iters_stable {
            // Yes, this might take more than a millisecond, but that's a cost of doing
            // timeouts with iteration counts, which is so much simpler to do cross-platform
            let curr_tree = self
                .interface
                .compute_tree()
                .await
                .map_err(|err| ExecutionError::InterfaceError { source: err.into() })?;

            if curr_tree != *last_tree {
                change_recorded = true;
                stable_iters = 0;
            } else if change_recorded {
                stable_iters += 1;
            }

            *last_tree = curr_tree;
            total_iters += 1;
            sleep(self.opts.tree_poll_interval_ms as u64).await;
        }

        if !change_recorded {
            // The tree never changed
            return Err(ExecutionError::NoTreeUpdate {
                timeout_ms: self.opts.stability_threshold_ms,
            });
        } else if stable_iters < num_iters_stable {
            // The tree didn't stabilise in time
            return Err(ExecutionError::TreeStabilisationTimeout {
                timeout_ms: self.opts.stability_timeout_ms,
            });
        }

        Ok(())
    }
}

/// Options for executing a command against an interface.
pub struct ExecutorOpts {
    /// The maximum number of round trips to be made to and from the AI. Lower values here
    /// will limit network requests, but
    pub max_round_trips: u32,
    /// The number of milliseconds to wait between each poll of the element tree. This should
    /// be a long enough amount of time to let the interface recover (can be tiny in browsers),
    /// but should be short enough that delays in polling are largely imperceptible.
    pub tree_poll_interval_ms: u32,
    /// A number of milliseconds to wait before a page is considered stable. This is involved
    /// when we poll the page's element tree; if we find it has changed since the last time
    /// we polled and we're expecting an update, we'll wait for this long before polling
    /// it again to determine if the change is "complete".
    ///
    /// Simple timer checks are done in lieu of more complex event-based approaches for now,
    /// as these are highly platform-specific.
    ///
    /// This is used by dividing it by `tree_poll_interval_ms` to determine how many polls of
    /// the tree this amounts to.
    pub stability_threshold_ms: u32,
    /// The number of milliseconds before a page should be considered volatile (i.e. not stabilising)
    /// and waiting for its stabilisation should be aborted. If this timeout is reached, a command
    /// may stop executing midway. Network failures in particular can cause this problem. This should
    /// generally be a fairly long amount of time, but not long enough that the user will wonder
    /// what's happening.
    ///
    /// This is used by dividing it by `tree_poll_interval_ms` to determine how many polls of
    /// the tree this amounts to.
    pub stability_timeout_ms: u32,
}

/// An action we should take on an interface. These are internal to executors,
/// and use their internal element IDs.
#[derive(Debug)]
enum Action {
    /// Click an element.
    Click {
        /// The unique ID of the element to click.
        id: u32,
    },
    /// Type on an element.
    Type {
        /// The unique ID of the element to fill out.
        id: u32,
        /// The text to fill it out with.
        text: String,
    },
}

/// A group of actions with a description of what they do.
#[derive(Debug)]
struct ActionGroup {
    /// The actions in this group.
    pub actions: Vec<Action>,
    /// The description of the actions taken.
    pub description: String,
    /// Whether or not this action group is *final* in the context of the command it
    /// furthers.
    pub complete: bool,
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
    /// Parses action strings from the model into proper actions.
    fn parse_action_strings(actions: &[&str]) -> Result<Self, ActionParseError> {
        let mut group = Self::default();

        let mut actions = actions.iter().peekable();
        while let Some(action_str) = actions.next() {
            // No action has more than three parameters
            let mut parts = action_str.splitn(3, ' ');
            // Guaranteed to be at least one part
            let action_ty = parts.next().unwrap();

            if action_ty == "CLICK" {
                let id = parts.next().ok_or(ActionParseError::MissingId {
                    ty: action_ty.to_string(),
                })?;
                // The ID might be wrapped in square brackets (some models do this, others don't)
                let id = id
                    .strip_prefix('[')
                    .unwrap_or(id)
                    .strip_suffix(']')
                    .unwrap_or(id);
                let id = id
                    .parse()
                    .map_err(|_| ActionParseError::NonIntegerId { id: id.to_string() })?;

                group.actions.push(Action::Click { id })
            } else if action_ty == "FILL" {
                let id = parts.next().ok_or(ActionParseError::MissingId {
                    ty: action_ty.to_string(),
                })?;
                let id = id
                    .strip_prefix('[')
                    .unwrap_or(id)
                    .strip_suffix(']')
                    .unwrap_or(id);
                let id = id
                    .parse()
                    .map_err(|_| ActionParseError::NonIntegerId { id: id.to_string() })?;

                let text = parts.next().ok_or(ActionParseError::MissingTextInType)?;
                let text = text
                    .strip_prefix('\'')
                    .map(|t| t.strip_suffix('\''))
                    .flatten()
                    .ok_or(ActionParseError::TextInTypeNotSingleQuoted {
                        text: text.to_string(),
                    })?;

                group.actions.push(Action::Type {
                    id,
                    text: text.to_string(),
                })
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
                        return Err(ActionParseError::MissingDescription);
                    }
                    group.description = description.trim().to_string();

                    group.complete = false;
                    break;
                }
            } else if action_ty == "FINISH" {
                let description = parts.collect::<Vec<_>>().join(" ");
                if description.is_empty() {
                    return Err(ActionParseError::MissingDescription);
                }
                group.description = description.trim().to_string();
                group.complete = true;

                if actions.peek().is_some() {
                    return Err(ActionParseError::ActionsAfterFinish);
                }
            }
        }

        Ok(group)
    }
}
