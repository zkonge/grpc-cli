# grpc-cli (WIP)
Useful functions for interacting with gRPC, including:
  * dummy server (only one method each running)
  * client
  * protobuf compiler
  * protobuf descriptor inspector
  * protobuf binary-json converter

# help command

```
Usage: grpc-cli <command> [<args>]

Useful functions for interacting with gRPC, including:
  * dummy server (only one method each running)
  * client
  * protobuf compiler
  * protobuf descriptor inspector
  * protobuf binary-json converter

Before running the program, you need to precompile the protobuf files into protobuf file descriptors.
you can either use the `protoc` command or the grpc-cli builtin command `compile` to do this.

'''bash
$ grpc_cli compile -i ./proto ./proto/hello.proto
# output to "./output.desc", for more command line options, run see the help of `compile`
'''

Options:
  --help, help      display usage information

Commands:
  compile           compile the protobuf files into a descriptor set file
  inspect           print detailed protobuf type info from the descriptor set
  server            acting as a server to handle a gRPC method
  client            acting as a client to call a gRPC method
  json              convert data between protobuf binary data and JSON
  version           print the version of the application
```
