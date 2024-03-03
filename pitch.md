# Solution

## What it does

Screen readers already greatly enhance accessibility on the web by reading the contents of websites out loud to users.
This helps people who have impaired vision or are unable to interact with their computers through standard hardware interfaces such as mice and keyboards.
However, these tools focus heavily on the user's consumption of information.
Voxurf, on the other hand, allows user who rely on assistive technology to perform actions and execute commands on websites in their browsers.
It is a browser webextension that records the user's voice commands and executes them in the browser.

## How we built it

We decided to use the blazingly fast programming language Rust for the entire software stack of our project, spanning from a backend server to the WebAssembly frontend.
Our software stack can be composed into the following pipeline:
First, we record the user's voice prompt and transcribe that audio to text using OpenAI's open-source Whisper speech recognition model, which we run locally on the user's machine for improved performance, security and data privacy.
TODO: Sam, describe the other steps

## Challenges we ran into

Initially, we wanted to package the whole application into a single WebAssembly (WASM) executable.
However, we had difficulties porting the Whisper model to WebAssembly, so we had to pivot and provide the voice transcribing feature through a simple web-server.

## Accomplishments that we're proud of

## What we learned

TODO: Sam: chrome has many security vulnerabilities, creating wasm chrome extensions was suprisingly smooth (WASM is next up)

## What's next for Voxurf

- Closed the feedback with an AI voice speaking back to you — opening the doors for applications such as long-form writing. Implementing this into our prototype was out of MVP scope.
