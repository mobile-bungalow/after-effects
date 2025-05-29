# Rust Bindings for Adobe AfterEffects® & Premiere Pro® SDKs

High level bindings for the Adobe AfterEffects® (Ae) and Adobe Premiere Pro® SDKs.

This wraps many of the API suites in the Ae and Pr SDK and exposes them in _safe_ Rust.

It also defines a set of macros that implement all the plugin boilerplate for you, so you can
focus just on your actual plugin implementation.

_Building the plugins is done entirely with Rust – there's no need to use any external programs or
dependencies._

Packaging of the final plugin is done using a [`just`](https://github.com/casey/just) script.
Install with `cargo install just` and download
[`AdobePlugin.just`](https://raw.githubusercontent.com/virtualritz/after-effects/master/AdobePlugin.just)
and put it next to your `Cargo.toml`.

Adobe plugins contain a special resource describing the plugin called `PiPL`. This repository
includes a `PiPL` tool written in Rust which generates the needed resource in `build.rs`.

Pre-generated SDK bindings are included, so you can compile the final plugin by just running
`just release`, and it works on both macOS and Windows.

You can also re-generate the bindings by downloading the SDK headers from Adobe and setting
`AESDK_ROOT` and/or `PRSDK_ROOT` environment variables.

## Using

Add `after-effects` or `premiere` to your dependencies and `pipl` to your dev-dependencies.

```text
cargo add after-effects
cargo add --dev pipl
```

> [!NOTE]
> When using with Premiere and you want to use any of the Premiere's suites, make sure to add `println!("cargo:rustc-cfg=with_premiere");` to your `build.rs` so that the plugin can initialize `premiere`'s basic suite in `EffectMain`

## After Effects vs. Premiere

Adobe plugins are shared between After Effects and Premiere.

The main engine is based on After Effects, but Premiere loads most of the Ae plugins.
While they have many common parts, there are some areas that are separated.

- Premiere is missing all `AEGP` suites.

- Premiere uses only software rendering, even if the Ae plugin supports GPU render and Smart
  Render.

- Premiere has a separate entry point for GPU rendering, which can be defined using the
  `premiere::define_gpu_filter!` macro.

- After Effects and Premiere also have some separate areas that are implemented independently.

- You can't write a video filter plugin using only the Premiere SDK, the base engine is using Ae
  SDK.

## Getting Started

### Examples

A few basic examples are [included in the repository](https://github.com/virtualritz/after-effects/tree/master/examples).
For more advanced use cases, refer to the C/C++ examples from the SDK.

To build each example simply run the following from inside the desired example directory.

```bash
CARGO_TARGET_DIR=$(pwd)/target
just build
```

The `just` command will package and install the example plugin.

To debug the loading process consult the `Plugin Loading.log` or run AfterEffects from the CLI in
your debugger of choice.

The most common causes of plugin loading failure on MacOs are lack of signing, and incorrect PkgInfo or Info.plist contents.

For a more advanced sample with full GPU rendering you can check out the
[Gyroflow plugin](https://github.com/gyroflow/gyroflow-plugins)

### Development

When developing your plugin it's best to use the debug build - it will catch and display panics
for you and it will log any messages and stack traces to `DbgView` (on Windows) or `Console` (on
macOS). This is done by running `just build`.

The release version can be built using `just release`

Some plugins may be slow in debug build, in this case you can add optimizations to the debug build
by using

```toml
[profile.dev]
opt-level = 3
```

or add debug symbols to your release build by using

```toml
[profile.release]
debug = true
debug-assertions = true
```

in your `Cargo.toml` file.

The release build doesn't catch panics by default to not add any additional overhead. You can
opt-in for the panic handler by enabling `catch-panics` `cargo` feature:

```toml
features = ["catch-panics"]
```

## Help Wanted/To Do

- If you need a suite that's not yet wrapped, feel free to create a PR wrapping that suite.

- Examples and documentation.

- Better error handling. Possibly using [`color`](https://crates.io/crates/color-eyre)`-`[`eyre`](https://crates.io/crates/eyre)?

### Using the Adobe SDK C++ Headers

Download the [_Adobe After Effects SDK_](https://console.adobe.io/downloads/ae).

> ⚠️ The SDK published by Adobe is outdated if you are using the 3D
> Artisan API to write your own 3D renderer plug-in.
> Also see [Features](#features) below for more information.
>
> Ignore this if you just want to develop 2D plugins (which still have
> access to 3D data).

Define the `AESDK_ROOT` environment variable that contains the path to your
Ae SDK. Typically the directory structure will look like this:

```ignore
AfterEffectsSDK
├── After_Effects_SDK_Guide.pdf
├── Examples
    ├── AEGP
    ├── Effect
    ├── ...
```

## Wrapped Suites

### After Effects

| AEGP                    | PF                                | DRAWBOT     | Other                 |
| ----------------------- | --------------------------------- | ----------- | --------------------- |
| 🔳 Artisan Util         | ✅ AE Adv App                     | ✅ Draw     | ✅ AE Plugin Helper   |
| ✅ Camera               | ✅ AE Adv Item                    | ✅ Image    | ✅ AE Plugin Helper 2 |
| ✅ Canvas               | 🔳 AE Adv Time                    | ✅ Path     |                       |
| 🔳 Collection           | ✅ AE App                         | ✅ Pen      |                       |
| ✅ Command              | ✅ AngleParam                     | ✅ Supplier |                       |
| ✅ Comp                 | 🔳 ANSI                           | ✅ Surface  |                       |
| ✅ Composite            | ✅ Background Frame               |             |                       |
| 🔳 Compute              | 🔳 Batch Sampling                 |             |                       |
| ✅ Dynamic Stream       | ✅ Cache On Load                  |             |                       |
| ✅ Effect               | ✅ Channel                        |             |                       |
| 🔳 File Import Manager  | ✅ Color Settings                 |             |                       |
| ✅ Footage              | ✅ Color Callbacks                |             |                       |
| 🔳 Hash                 | ✅ Color Callbacks 16             |             |                       |
| ✅ IO In                | ✅ Color Callbacks Float          |             |                       |
| 🔳 IO Out               | ✅ ColorParam                     |             |                       |
| ✅ Item                 | ✅ Effect Custom UI               |             |                       |
| 🔳 Item View            | ✅ Effect Custom UI Overlay Theme |             |                       |
| 🔳 Iterate              | ✅ Effect Sequence Data           |             |                       |
| ✅ Keyframe             | ✅ Effect UI                      |             |                       |
| 🔳 Layer Mask           | ✅ Fill Matte                     |             |                       |
| ✅ Layer Render Options | ✅ GPU Device                     |             |                       |
| ✅ Layer                | ✅ Handle                         |             |                       |
| ✅ Light                | ✅ Iterate8                       |             |                       |
| 🔳 Marker               | ✅ iterate16                      |             |                       |
| ✅ Mask Outline         | ✅ iterateFloat                   |             |                       |
| ✅ Mask                 | ✅ Param Utils                    |             |                       |
| 🔳 Math                 | ✅ Path Data                      |             |                       |
| ✅ Memory               | ✅ Path Query                     |             |                       |
| 🔳 Output Module        | ✅ Pixel Data                     |             |                       |
| 🔳 Persistent Data      | ✅ Pixel Format                   |             |                       |
| ✅ PF Interface         | ✅ PointParam                     |             |                       |
| ✅ Proj                 | 🔳 Sampling8                      |             |                       |
| 🔳 QueryXform           | 🔳 Sampling16                     |             |                       |
| ✅  Register             | 🔳 SamplingFloat                  |             |                       |
| ✅ Render Asyc Manager  | ✅ Source Settings                |             |                       |
| ✅ Render Options       | ✅ Transition                     |             |                       |
| 🔳 Render Queue Item    | ✅ Utility                        |             |                       |
| 🔳 Render Queue         | ✅ World                          |             |                       |
| ✅ Render               | ✅ World Transform                |             |                       |
| 🔳 RenderQueue Monitor  |                                   |             |                       |
| ✅ Sound Data           |                                   |             |                       |
| ✅ Stream               |                                   |             |                       |
| 🔳 Text Document        |                                   |             |                       |
| 🔳 Text Layer           |                                   |             |                       |
| 🔳 Tracker              |                                   |             |                       |
| 🔳 Tracker Utility      |                                   |             |                       |
| ✅ Utility              |                                   |             |                       |
| 🔳 Workspace Panel      |                                   |             |                       |
| ✅ World                |                                   |             |                       |

*The register suite currently excludes the artisan and AEIO registration API

### Premiere

| Premiere                  | MediaCore                        | Control Surface                 | Other                    |
| ------------------------- | -------------------------------- | ------------------------------- | ------------------------ |
| 🔳 Audio                  | 🔳 Accelerated Render Invocation | 🔳 ControlSurface               | ✅ PF Background Frame   |
| 🔳 Clip Render            | 🔳 App Info                      | 🔳 ControlSurface Command       | ✅ PF Cache On Load      |
| 🔳 Deferred Processing    | 🔳 Application Settings          | 🔳 ControlSurface Lumetri       | ✅ PF Pixel Format       |
| 🔳 Error                  | 🔳 Async File Reader             | 🔳 ControlSurface Marker        | ✅ PF Source Settings    |
| 🔳 Export Audio Param     | 🔳 Async Operation               | 🔳 ControlSurface Mixer         | ✅ PF Transition         |
| 🔳 Export Standard Param  | 🔳 Export File                   | 🔳 ControlSurface Transport     | ✅ PF Utility            |
| 🔳 ExportController       | 🔳 Export Info                   | 🔳 ControlSurfaceHost           | ✅ Opaque Effect Data    |
| 🔳 File Registration      | 🔳 Export Param                  | 🔳 ControlSurfaceHost Command   | 🔳 Captioning            |
| 🔳 Image Processing       | 🔳 Export Progress               | 🔳 ControlSurfaceHost Lumetri   | 🔳 Effect Stream Label   |
| 🔳 Legacy                 | 🔳 Exporter Utility              | 🔳 ControlSurfaceHost Marker    | 🔳 FlashCueMarkerData    |
| 🔳 Media Accelerator      | ✅ GPU Device                    | 🔳 ControlSurfaceHost Mixer     | 🔳 Importer File Manager |
| ✅ Memory Manager         | ✅ GPU Image Processing          | 🔳 ControlSurfaceHost Transport | 🔳 Marker                |
| 🔳 Palette                | 🔳 Playmod Immersive Video       | 🔳 ControlSurfacePlugin         |                          |
| 🔳 Pixel Format           | 🔳 Playmod Overlay               | 🔳 String                       |                          |
| 🔳 Playmod Audio          | 🔳 Sequence Audio                |                                 |                          |
| 🔳 Playmod Device Control | ✅ Sequence Info                 |                                 |                          |
| ✅ PPix                   | 🔳 Sequence Render               |                                 |                          |
| ✅ PPix 2                 | 🔳 Smart Rendering               |                                 |                          |
| 🔳 PPix Cache             | 🔳 String                        |                                 |                          |
| 🔳 PPix Creator           | 🔳 Transmit Invocation           |                                 |                          |
| 🔳 PPix Creator 2         | ✅ Video Segment                 |                                 |                          |
| 🔳 RollCrawl              | 🔳 Video Segment Render          |                                 |                          |
| 🔳 Threaded Work          |                                  |                                 |                          |
| ✅ Time                   |                                  |                                 |                          |
| ✅ Window                 |                                  |                                 |                          |

## License

Apache-2.0 OR BSD-3-Clause OR MIT OR Zlib at your option.
