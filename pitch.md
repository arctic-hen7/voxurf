# Solution

## What it does

Right now, assistive technology on the web consists largely of using a screen reader to read content to you, and then touch-typing or using a Braille keyboard to *act* on a website, which is totally impractical for those with acquired visual impairment, especially the elderly.

Voxurf is a web extension, built entirely in Rust, that provides a simple interface where users can speak a command, which is interpreted by an AI in the context of a simplified version of the active webpage. It then executes actions corresponding to the parts of the user's command, allowing the blind and visually impaired to act in a way screen readers could never allow.

## How we built it

We were committed to doing this entire project in Rust for extreme speed and safety, so we began by setting up a web extension that uses Rust to do everything, from rendering the popup to interacting with the browser. After writing a little bit of glue code and wading through some content security policy documentation, we got this working pretty quickly, and proceeded on to get transcription and website structure extraction working. One member of our team worked on a server that would support transcribing speech to text using OpenAI's Whisper model, all completely locally, while another worked on the UI and another on the extraction component.

By using Chrome's DevTools protocol, we were able to extract the browser's computed accessibility tree, which then needed to be deserialised into an appropriate data structure in Rust, and filtered to remove irrelevant elements. Once this was done (which took unreasonably long, thank you JS types and Chrome docs), we formatted this in a way an LLM could ingest, and engineered a prompt that would get it to produce some JavaScript code that would execute the actions the user's command corresponded to.

For that, we needed to implement a system that resolved the *backend* IDs the accessibility API returns into *frontend* IDs that could be used to reference the nodes in the DOM API, and then we needed to add *attributes* to those nodes so the JS code could reference these. Again, thank you Chrome.

*Then*, with the transcription server ready, we threw everything together in the UI and got a record-transcribe-execute loop working! After figuring out a way of executing JS from a web extension, bypassing Chrome's normal security settings, we were in business, and could execute the mind-blowing action of hiding a sidebar in the Chrome developer documentation, using your voice! Very happily, no further code changes were needed to use the system to file GitHub issues, and even fill out the UniHack registration form.

## Challenges we ran into

Along the way, we ran into plenty of challenges. The first was in getting a Wasm-based web extension to work: that required manually writing the glue code between Wasm and JS, rather than using a build tool, because inline execution is disabled in v3 Chrome extensions. Then we had to work with the accessibility tree, which, as mentioned above, gives completely different IDs than the DOM API uses, and the latter all start as 0 unless you specially initialise them through an undocumented method. Figuring this out involved going through Chromium's actual source code, which was a spiritual experience, in some sense or another.

Even worse than this, half the elements provided in that tree have `ignored: true`, and are totally irrelevant to completing actions on the page. But, Chrome provides a flat tree structure, where each element references the IDs of its parent and children. If you filter out irrelevant nodes, you can't reconstitute the nested structure of the tree (because their children might be relevant), so we had to implement an algorithm to reconstitute the tree while also hoisting relevant children out from under irrelevant parents. With that done, we found ourselves stymied by JS execution, which was the biggest bottleneck by far.

In manifest v3, Chrome prevents extensions from executing anything dynamic, only allowing static scripts packed in with the extension to be run. This means no `eval()`, no inserting scripts into the `<head>` of the host page, and even enabling the `userScripts` privilege didn't work, because we needed to dynamically register and execute scripts, which that facility couldn't do (it only supports running scripts that have been pre-registered when a page loads). After a good deal of head-banging and trying to sneak `eval()` calls in anywhere we could, we realised the Chrome debugger API actually supports arbitrary code execution, so we used that, with its very convenient ability to *totally bypass content security policies*. Interestingly, Chrome requires users to enable developer mode to use the `userScripts` privilege, but the infinitely more powerful `debugger` permission does *not* require this, rendering that entire part of their security model utterly pointless.

We also initially tried to do transcription inside the browser, but this completely failed due to compilation issues, so we settled with a native server to communicate with over REST for this prototype.

## Accomplishments that we're proud of

We managed to build a Rust-based web extension, using a Rust frontend framework to render everything, and we also managed to make that extension let you use websites with your voice, even submitting GitHub issues with it! That was enough of an exciting moment that we got a noise complaint in the Library ;)

## What we learned

Through this project, we learned how to build a web extension, none of us having ever done that before, and one in Rust at that! We also implemented a complex tree traversal algorithm that taught us both algorithmic lessons and some lessons in Rust lifetimes. We also spent plenty of time prompt engineering, learning about what GPT-3.5 interprets as subtle signals to either follow your instructions better, or completely ignore them.

From working with the DevTools API, we learned all sorts of things about how Chrome processes the DOM, and how JS works under the hood, including how a part of Chrome's extension security model is largely pointless.

## What's next for Voxurf

Our roadmap can be split into three phases

### Phase I

This is where we are now, so we'll polish the code we have, especially focusing on getting recording and transcription working natively without a separate server (which, while it still runs locally, makes portability of the extension much more complex). We'll also continue to refine the prompt to allow the AI to execute composite actions (where it runs some code, then looks at the new state of the page, and then does some more). We tried to get that working for our prototype, but it was very unreliable.

### Phase II

Here, we'll aim to integrate our solution with [Sotto](https://arctic-hen7.github.io/post/sotto), a previous project of one of our team members, allowing users to dictate and edit longform text (imagine Vim, but for voice), and to then interact with the web, to do anything from write and edit an email to post an academic paper to ArXiV.

We also want to scale our solution to work with web-based apps (e.g. Electron), and eventually any native app through an OS layer. Mobile support will also be in order here.

### Phase III

Here, we would build a proprietary hardware device that could run our system against any app or interface, allowing the blind and visually impaired to interact with a device completely optimised for them, while also providing a productivity aid to anyone who wants it.
