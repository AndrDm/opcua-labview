# opcua-LabVIEW
OPC UA wrapper library for LabVIEW written entirely in Rust.
This is just a proof of concept and a feasibility study.

Based on the [server / client API implementation for Rust](https://github.com/FreeOpcUa/async-opcua).

The node's tree builder is based on [OPC UA Server Browser](https://github.com/jacobson3/UA-Server-Browser)

0.2.0 - 21-MAR-2025
+ ClientBuilder from Config
+ Read/Wrte doubles added
+ ns added as param to read
+ GetNodeInfo added and browser with OnChange example

0.1.0 - Initial Draft 16-MAR-2025