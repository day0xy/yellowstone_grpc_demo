## Project Introduction

A Rust project using [yellowstone gRPC](https://github.com/rpcpool/yellowstone-grpc) to monitor Solana on-chain contract events and parse specific contract events (extensible for your own needs).

## Project Structure

```
src/
├── client/         # gRPC client wrapper
├── filters/        # Create SubscribeRequest filters
├── types/          # Event data models and specific event parsing
├── utils/          # Utility functions
└── main.rs         # Program entry point
```

## Run

```
cargo run
```

## Reference Projects

- [rpcpool/yellowstone-grpc](https://github.com/rpcpool/yellowstone-grpc)
- [ChainBuff/yellowstone-grpc-rust](https://github.com/ChainBuff/yellowstone-grpc-rust)
- [Fuck-Meme/grpc-demo](https://github.com/Fuck-Meme/grpc-demo)
- [docs.triton.one](https://docs.triton.one/project-yellowstone/dragons-mouth-grpc-subscriptions#example-subscribe-requests)
