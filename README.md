中文/[English](./README_EN.md)
## 项目介绍

一个使用 [ yellowstone gRPC](https://github.com/rpcpool/yellowstone-grpc)监控 solana 上合约事件的 rust 项目，解析特定的合约事件(可自行拓展)。

## 项目结构

```
src/
├── client/         # gRPC 客户端包装
├── filters/        # 创建过滤请求SubscribeRequest
├── types/          # 事件数据模型,特定事件解析处理
├── utils/          # 一些辅助函数
└── main.rs         # 程序入口
```

## 运行

```
cargo run
```

## 参考项目

[rpcpool/yellowstone-grpc](https://github.com/rpcpool/yellowstone-grpc)<br>
[ChainBuff/yellowstone-grpc-rust](https://github.com/ChainBuff/yellowstone-grpc-rust)<br>
[Fuck-Meme/grpc-demo](https://github.com/Fuck-Meme/grpc-demo)<br>
[docs.triton.one](https://docs.triton.one/project-yellowstone/dragons-mouth-grpc-subscriptions#example-subscribe-requests)<br>
