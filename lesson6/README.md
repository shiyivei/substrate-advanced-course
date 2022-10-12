# 第6课作业

1. 为 proof of existence (poe) 模块的可调用函数 create_claim, revoke_claim, transfer_claim 添加 benchmark 用例，并且将 benchmark 运行的结果应用在可调用函数上

```
./benchmarks-poe/pallets/poe/src/benchmarking.rs
./benchmarks-poe/pallets/poe/src/lib.rs
```

2. 生成chain spec,两种格式：

```
shiyivei-staging.json
shiyivei-staging-raw.json
```

3. -（附加题）根据 Chain Spec，部署公开测试网络