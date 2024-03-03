# Voxurf voice recording and transcribing

This package contains a server with two endpoints.
The purpose of this server is to facilitate easy communication with the WASM-based browser extension.
`/start-recording` starts recording audio until `/end-recording` is invoked.
Once the recording has been stopped by invoking `/end-recording`, the audio is automatically transcribed, and will be returned in textual form.
If the recording and transcribing succeed, the following JSON is returned by `/end-recording`:
```json
{ "transcription": transcription }
```
Otherwise, the following JSON will be returned by `/end-recording`:
```json
{ "error": error-message }
```
