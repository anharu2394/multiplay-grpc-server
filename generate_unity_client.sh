#!/bin/bash
protoc protos/multiplay.proto -I=protos --csharp_out=unity-client --grpc_out=unity-client --plugin=protoc-gen-grpc=/usr/local/bin/grpc_csharp_plugin
