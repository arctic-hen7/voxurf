# Notes

This will serve as a basic documentation workspace where we can all explain what we've done and how we've done it so interop between our different components is a little easier and we don't have to be posting the same things repeatedly. In short, put your half-arsed docs in here!

## Fritz

## Lucas

## Miguel

## Michael

## Sam

The project scaffold has two crates: `voxurf` (the library which handles basically everything), and `voxurf-extension`, which wraps that library to provide something working. The idea is that `voxurf` should be generally independent of the extension API, but this distinction is not required for the prototype. If something is a bit hacky, that's fine!

We can use `web-extensions-sys` to access web extension APIs I think!
