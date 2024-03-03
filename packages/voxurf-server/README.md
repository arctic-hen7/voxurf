# Voxurf voice recording and transcribing

This package contains a server with two endpoints.
The purpose of this server is to facilitate easy communication with the WASM-based browser extension.
`/start-recording` starts recording audio until `/end-recording` is invoked.
Once the recording has been stopped by invoking `/end-recording`, the audio is automatically transcribed, and will be returned in textual form.
If the recording and transcribing succeed, `/end-recording` will return the raw transcribed text as a string, otherwise the server panics.
