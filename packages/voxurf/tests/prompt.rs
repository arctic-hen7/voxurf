use voxurf::{OpenAiModel, Model};

static DESCRIPTOR: &str = "ms_gpt_4";
const ITERS: usize = 3;

#[tokio::test]
async fn prompt_works() {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap();

    std::fs::write(format!("prompts/{DESCRIPTOR}.txt"), PROMPT).unwrap();
    let model = OpenAiModel::new(api_key.to_string()).with_model("gpt-4-0125-preview");
    for i in 0..ITERS {
        let response = model.prompt(PROMPT).await.unwrap();
        std::fs::write(format!("prompts/{DESCRIPTOR}-response-{i}.txt"), response).unwrap();
    }
}

static PROMPT: &str = r#"You are an AI browser extension that helps the blind and visually impaired use websites with their voice.

The following is a nested representation of relevant nodes in the accessibility tree of a website. All provided nodes are focusable, and may be referenced by their unique ID numbers provided in square brackets.

```text
- [0] "Jump to content" (link)
- [3] "Wikipedia The Free Encyclopedia" (link)
- [6] "Main menu" (button) {hasPopup: menu, invalid: false}
- [12] "Personal tools" (button) {hasPopup: menu, invalid: false}
- [15] "Toggle limited content width" (button) {invalid: false}
- [18] "Go to an article in another language. Available in 38 languages" (button) {hasPopup: menu, expanded: false, invalid: false}
- [21] "Creative Commons Attribution-ShareAlike License 4.0" (link)
- [24] "Terms of Use" (link)
- [27] "Privacy Policy" (link)
- [30] "Wikimedia Foundation, Inc." (link)
- [33] "Privacy policy" (link)
- [36] "About Wikipedia" (link)
- [39] "Disclaimers" (link)
- [42] "Contact Wikipedia" (link)
- [45] "Code of Conduct" (link)
- [48] "Developers" (link)
- [51] "Statistics" (link)
- [54] "Cookie statement" (link)
- [57] "Mobile view" (link)
- [60] "Wikimedia Foundation" (link)
- [63] "Powered by MediaWiki" (link)
- [66] "Search Wikipedia" (combobox) (Search Wikipedia [alt-shift-f]) {settable: true, hasPopup: listbox, expanded: false, required: false, invalid: false, controls: cdx-typeahead-search-menu-0, autocomplete: list, keyshortcuts: Alt+f, editable: plaintext}
- [69] "Create account" (link) (You are encouraged to create an account and log in; however, it is not mandatory)
- [72] "Log in" (link) (You're encouraged to log in; however, it's not mandatory. [alt-shift-o]) {keyshortcuts: Alt+o}
- [75] "Tools" (button) {hasPopup: menu, invalid: false}
- [78] "Glossary of botanical terms § dichotomous" (link) (Glossary of botanical terms)
- [81] "Dichotomy (album)" (link) (Dichotomy (album))
- [84] "" (link)
- [87] "verification" (link) (Wikipedia:Verifiability)
- [90] "improve this article" (link) (Special:EditPage/Dichotomy)
- [93] "adding citations to reliable sources" (link) (Help:Referencing for beginners)
- [96] ""Dichotomy"" (link)
- [99] "news" (link)
- [102] "newspapers" (link)
- [105] "books" (link)
- [108] "scholar" (link)
- [111] "JSTOR" (link)
- [114] "Learn how and when to remove this template message" (link) (Help:Maintenance template removal)
- [117] "" (link)
```

Using this, break the following command, which was transcribed from the user's speech, into stages. Each stage should be on a new line, and must be one of the following stages:

- "CLICK <element-id>" --- clicks on the element with the given ID
- "FILL <element-id> 'Text to fill'" --- types the given text into the element with the given ID (you don't need to click on an element first to fill it, this will be done automatically)
- "WAIT <brief description of what's been done so far; e.g. Clicked the new email button>" --- waits for the interface to change so you can keep going
- "FINISH <brief description, as in WAIT>" --- completes the action (use last)

In your response, first write your thought process about what to do, step-by-step, and then write the stages and their parameters in a Markdown code fence with type `text`. Do NOT use comments in this code fence, only descriptions where the stages take them. Be sure to finish with the FINISH stage at the end of your list of actions.

Make sure you use valid element IDs wherever you can, and only use placeholders *after* a WAIT stage!

User's command:

```text
Search Wikipedia for foobar and go to the disambiguation.
```"#;
