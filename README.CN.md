# Guided Review

> 本项目正在密集开发中。

**把任何 Pull Request 变成一张可读的地图 —— 每个论断都有真实代码作证。**

[English Version](./README.md)

![渲染后的 Guided Review 页面](./docs/screenshot.png)

---

## 为什么需要 Guided Review？

你打开一个改了 60 个文件的 PR。描述里说的是一件事，diff 里是四十件。你一个
文件一个文件地滚动，线索越看越散，最后要么凭感觉批准，要么揪着一个小毛病
不放，而真正的风险悄悄上线了。

Review 工具在"展示 diff"上越来越强，却从没学会"解释 diff"。当 AI 写的代码
越来越多，瓶颈早已不是写出改动，而是理解改动。

**Guided Review 的出发点很简单：评审产物应该降低理解成本，而不是增加仪式感。**

## 设计哲学

### 没有证据就不算数

Guided Review 里的每一句话 —— 主旨、每条风险、每个回答 —— 都必须引用 diff
中的真实代码行。渲染器会拒绝没有代码摘录的论断、行数对不上的摘录，以及
自相矛盾的"带阻塞项的批准"。无法验证的评审，就是无法信任的评审。

### 是地图，不是流水账

按文件逐个总结，只是复述你本来就能看到的东西。Guided Review 把变更还原成
一个系统：一句话主旨、建议的阅读顺序、变更的明线与暗线，以及值得人类注意
的风险。

### 判断权留给人

AI 起草地图，人类掌握合并决定。每个论断都标注为 *observed*（直接读到）或
*synthesis*（综合推断），评审者永远知道哪些话是从代码里读出来的，哪些是
推理出来的。

## 你可以做什么

**读一页，而不是六十个文件**

- 每个 PR 一张自包含的 HTML 页面 —— 查看时不需要服务器、账号或构建步骤。
- 建议阅读顺序告诉你从哪里开始、为什么。
- 每个论断都链接到带语法高亮的代码摘录，精确到文件和行号。

**判断风险，而不是按 diff 顺序看代码**

- 风险分级：blocker、should-fix、follow-up。
- 每个论断的验证状态：verified、partial、unproven。
- 最终结论把阻塞项和非阻塞的打磨建议分开呈现。

**让你的 Agent 干粗活**

- `egr generate -h` 会打印出 JSON Schema，任何编码 Agent 都能照着填。
- [`skill/`](skill/SKILL.md) 内置一套现成的 Agent 技能，驱动完整流程：
  检查 diff、写 payload、渲染、本地预览。
- `egr serve` 在本地预览结果，关掉浏览器标签页后会自动优雅退出。

## 适合谁

Guided Review 适合这样的工程师：

- 要评审大型或 AI 生成的 PR，想看到故事主线而不是噪音；
- 在跑编码 Agent，想要真正可审计的评审产物；
- 相信"批准"应该意味着"我理解了"，而不是"我滚完了"。

## 愿景

代码评审正在成为软件流水线里最窄的一段。我们认为解法不是更快地略读，而是
让理解变得便宜、让验证成为强制的评审产物。Guided Review 希望成为评审者
伸手就拿的格式，就像今天的 diff 一样自然。

---

## 快速开始

```console
cargo install --git https://github.com/AkaraChen/guided-review guided-review   # 安装 `egr` 二进制
egr generate -h                   # 打印评审 JSON Schema
egr generate owner/repo#123 --review review.json --output out/index.html
egr serve out                     # 绑定 127.0.0.1 并打印 URL
```

完整 payload 见 [`examples/review.json`](examples/review.json)，Agent 工作流
见 [`skill/SKILL.md`](skill/SKILL.md)。
