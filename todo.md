# To-Do List

This is a list of tasks to be completed for Voxurf during the hackathon.

- [ ] Set up a basic web extension boilerplate that executes a Rust Wasm binary (using Sycamore) which prints "Hello World!" to the extension's dialogue (make sure this can be installed in the browser); then link this with the `voxurf` library
- [ ] Create a function in the `voxurf` library that records user audio using the web APIs and then transcribes it to text using Whisper (all in Rust)
- [ ] Create a function in `voxurf` that iterates through the tab hierarchy of a website and produces an LLM-ingestable version thereof
- [ ] Create a system in `voxurf` that sends this to GPT-3.5/GPT-4 and executes the actions as specified (this could either use an array of actions in JSON or have the AI write a script we run...)
- [ ] Build a system that allows submitting variables to be used in script execution (e.g. from the clipboard); the user should be able to add variables manually through the web extension interface (ideally these should be persisted, but not *absolutely* required)
- [ ] Design a nice interface for the extension that lets the user record an utterance and have that be executed on the current website (one button, start-then-stop); this should all be written in Sycamore
- [ ] Write and record the pitch with a demo (ideally demo on two websites, just make sure it works with these)

We want our extension to work in Chromium-based browsers (if it breaks in Firefox that's fine for now), and we want it to work on two websites: one for filling in and submitting a basic form, and another for sending email.
