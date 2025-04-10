# Async OPC-UA Types

Part of [async-opcua](https://crates.io/crates/async-opcua), a general purpose OPC-UA library in rust.

This library contains a framework for encoding and decoding OPC-UA messages, as well as generated code for all types defined in the standard.

This includes:

1. All of the built-in data types described in OPC Part 6 Chapter 5 that are encodable.
2. All of the standard data types described in OPC Part 3 Chapter 8 (if not covered by 1.).
3. Autogenerated data types and request / responses as described in OPC Part 4.

Notable types include

 - `Variant`, a discriminated union of a number of primitive types.
 - `ExtensionObject` a wrapper around an OPC-UA structure identified by its encoding ID.

## Features

 - `json`, enables OPC-UA JSON encoding and decoding.
 - `xml`, enables OPC-UA XML decoding, notably this is _not_ yet full support for OPC-UA XML, only a limited subset intended for use with `NodeSet2` XML files.

## Usage

Usually this library is used as part of an OPC-UA client or server.

Encoding is done by writing to a type implementing `std::io::Write`, and decoding
by reading from a type implementing `std::io::Read`:

```rust
let context_owned = ContextOwned::default();
let context = context_owned.context();

let my_opcua_value = Variant::from(123);

// Get the byte length before encoding.
// This is not actually required, but can be useful.
let byte_len = my_opcua_value.byte_len(&context);
let mut stream = Cursor::new(vec![0u8; byte_len]);

// Encode to a stream.
let start_pos = stream.position();
value.encode(&mut stream, &context)?;

stream.seek(SeekFrom::Start(0))?;
let decoded = Variant::decode(&mut stream, &context)?;

assert_eq!(my_opcua_value, decoded);
```

### Custom types

In order to make a custom OPC-UA structure, it must implement a number of traits depending on which features are enabled:

 - `BinaryEncodable` and `BinaryDecodable`, implementing encoding using the OPC-UA Binary protocol.
 - `JsonEncodable` and `JsonDecodable`, implementing encoding using OPC-UA JSON, if the `"json"` feature is enabled.
 - `FromXml`, loading the type from a NodeSet2 XML file, if the `"xml"` feature is enabled.
 - `Clone`, `Send`, `Sync`, `Debug`, `PartialEq` are all required.
 - `ExpandedMessageInfo`, which provides full encoding IDs.

A type that satisfies these requirements can be stored in an `ExtensionObject` and sent to OPC-UA. In order to _receive_ the type, it must be added to a `TypeLoader`.

`BinaryEncodable`, `BinaryDecodable`, `JsonEncodable`, `JsonDecodable`, and `FromXml` all have derive macros.

Enums are simpler, the easiest way to make a custom OPC-UA enum (not a union) is to derive the `UaEnum` trait, which also implements a few other traits needed for numeric OPC-UA enums.
