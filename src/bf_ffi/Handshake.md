# `bf_ffi` handshake protocol

## Definitions

* `BF` -> The BF program that is being run,
conformant with this protocol
* `host` -> The process communicating with `BF`
* `in` -> The `stdin` of `BF`
* `out` -> The `stdout` of `BF`
* `import` -> A function imported by `BF`
* `exported` -> A function exported by `BF`

## Types
Types used during the handshake and for function calls

* `void` -> An empty type encoded using 0 bytes
* `u{N}` -> An `N` bit unsigned integer encoded in Little Endian order. `N` must be a multiple of 8
* `ImportId` -> A unique `u16` describing an `import`
* `ExportId` -> A unique `u16` describing an `export`
* `Slice(T)` -> A sequence of elements of type `T`. Encoded as:
    * **length** = `u16`
    * **length** values of type `T`
* `String` -> A string. Encoded as:
    * `Slice(u8)`, a series of UTF8 codepoints
* `StringAscii` -> An ascii string. Encoded as:
    * `Slice(u8)`, the bytes of an ASCII string
* `(T, G, ..., Y, Z)` -> A tuple of any number of elements, of varying types. Encoded as:
    * Value of type `T`
    * Value of type `G`
    * ...
    * Value of type `Y`
    * Value of type `Z`

### Handshake Types
Types used only during the handshake

* `import_dec` -> Request for an `import` to be provided by `host`, associating the `import` with **UID**
    * **UID** = `ImportId`
    * **Common Import Name** = `StringAscii`
* `export_dec` -> Request for an `export` to be registered by `host` with the given name, associating the `export` with **UID**
    * **UID** = `ExportId`
    * **Common Export Name** = `StringAscii`

## Protocol

* `BF` sends `Slice(import_def)`
* `BF` sends `Slice(export_def)`
* Loop
    * `host` sends `ExportId`
        * `BF` starts running the matching `export` 
    * Loop
        * `BF` sends `ImportId`
        * `host` runs the matching `import`

## Calling convention

Calling convention, for an `import` or `export` `F` with parameter type `T` and return type `U`:
* `caller` sends `T`
* `callee` runs `F`
* `callee` sends `U`

For an `import`:
* `caller` is `BF`
* `callee` is `host`
* An `import` **_cannot_** call any `export`s when running

For an `export`:
* `caller` is `host`
* `callee` is `BF`
* An `export` **_can_** call `import`s when running

# Standard Features

Features which provide common behavior and may be dangerous if allowed for an untrusted program

Each feature is a list of `import`s with unique names

Format:

`import_common_name` = `ParamType -> ReturnType`
* Description of what this import does
* CONDITIONS:
    * A condition that must be upheld
    * Another condition

## Core
Must be supported by any `host`

`core::control_flow::bf_return` = `T -> void`
* `BF` signals termination of the current function call, and a new `export` may be called by `host`
* CONDITIONS:
    * `T` _must_ be the return type of the function that `BF` is returning from

TODO
## Std IO
Provides access to the `stdin,stdout,sterr` of `host`

TODO