Protobuf
=========
This is a simple example using protobuf to write multiple entries of "person" in a serialized file.

## Dependencies

```bash
sudo apt install protobuf-compiler
cargo install protobuf-codegen
```

How to generate the protobuf? Execute the following on the folder containing **person.proto**
```bash
protoc --rust_out=. person.proto
```

Then we have to import the generated code
