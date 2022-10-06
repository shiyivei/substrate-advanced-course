1. **在 Offchain Worker 中，使用 Offchain Indexing 特性实现从链上向 Offchain Storage 中写入数据**

![image](https://github.com/shiyivei/substrate-advanced-course/blob/master/lesson4/static/02.png)

2. **使用 js sdk 从浏览器 frontend 获取到前面写入 Offchain Storage 的数据**

![image](https://github.com/shiyivei/substrate-advanced-course/blob/master/lesson4/static/03.png)

3. **回答链上随机数（如前面Kitties示例中）与链下随机数的区别**

链上随机数([Randomness Module](https://docs.rs/pallet-randomness-collective-flip/3.0.0/pallet_randomness_collective_flip/))由当前节点的前81个block的哈希生成，通过这种方式获取的随机数并不是完全随机，存在潜在的风险（熵的来源并不是完全随机）所以链上随机数pallet只推荐在test时使用。

链下随机数([Offchain Random](https://docs.rs/sp-io/6.0.0/sp_io/offchain/fn.random_seed.html))由于是在链下执行，可以使用当前结节点系统关联生成不可预测的熵，以确保生成数的随机性

4. （可选）在 Offchain Worker 中，解决向链上发起不签名请求时剩下的那个错误。参考：https://github.com/paritytech/substrate/blob/master/frame/examples/offchain-worker/src/lib.rs

5. **（可选）构思一个应用场景，描述如何使用 Offchain Features 三大组件去实现它 扩展现有Kitty程序**

   假定生成kitty图片需要很大计算量，将kitty图片生成放入offchain worker执行。确定kitty图片id之后，再通过signed transaction通知链上更新此kitty。

- create/breed kitty的时候将kitty id 存入Indexing

- offchain worker 取得kitty id，进行计算 （sleep 8000ms模拟），根据block number的奇偶确定kitty id

- 将kitty id和计算后的图片id通过signed transaction回调链上extrinsic (update_kitty)
- update_kitty 更新最新kitty图片id到链上数据

**上述功能已经实现过程：每当新建/繁殖新的kitty后，kitty id同时会被保存到链下存储，链下工作机接着读取链下存储，并且按条件更新对应kitty的链上信息**

![image](https://github.com/shiyivei/substrate-advanced-course/blob/master/lesson4/static/02.png)

![image](https://github.com/shiyivei/substrate-advanced-course/blob/master/lesson4/static/03.png)

![image](https://github.com/shiyivei/substrate-advanced-course/blob/master/lesson4/static/04.png)

1. （可选）如果有时间，可以实现一个上述原型

## 