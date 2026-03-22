# Contributing

感谢你愿意为 `golutra` 提交贡献。

在提交任何贡献前，请先理解一件事：

- `golutra` 当前按 `BSL 1.1` 分发，并在 [`LICENSE`](./LICENSE) 中声明未来切换到 `GPL-2.0-or-later`
- 被接收的贡献可能被用于当前 `BSL 1.1` 版本、未来 GPL 版本，以及项目维护者提供的商业授权版本
- 如果你不能接受这些用途，请不要提交贡献

## 提交流程

1. Fork 仓库并创建你的功能分支。
2. 提交代码、测试或文档修改。
3. 发起 Pull Request。
4. 填写 PR 模板，如实声明你是个人贡献还是代表组织贡献，并披露第三方来源、AI 辅助情况和共同作者信息。
5. 第一次提交 PR 时，`CLA` 与 `PR Compliance` 工作流会自动运行。
6. 如果你还没有签署个人协议，机器人会在 PR 下留言并把 `CLA` 检查标记为失败。
7. 阅读 [`CLA.md`](./CLA.md) 和 [`docs/legal/ICLA.md`](./docs/legal/ICLA.md) 后，在 PR 评论区回复下面这句完整文本：

`I have read the ICLA and I hereby sign this agreement.`

8. 机器人记录签署后，`CLA` 检查会自动通过；如果没有自动刷新，再评论 `recheck`。
9. 如果你代表公司、团队或其他组织提交贡献，还需要先让有权签字人完成 [`docs/legal/CCLA.md`](./docs/legal/CCLA.md) 或等效书面授权，并在 PR 中填写已经登记的授权编号。

## 仓库维护者初始化要求

为了让 CLA 流程正常工作，仓库维护者还需要完成以下配置：

1. 在 GitHub 仓库设置中启用 Actions。
2. 如果组织允许把 Workflow permissions 设为 `Read and write`，可以直接启用；如果组织层强制只读，则需要额外设置仓库 secret `CLA_BOT_TOKEN`，其值应为具有 `repo` 与 `workflow` 范围的 token。
3. 在默认分支保护规则里把 `CLA` 和 `PR Compliance` 两个检查都加入必过状态。
4. 额外创建一个未受保护的 `cla-signatures` 分支，专门存储 `.github/cla/signatures.json`。
5. 不要手动创建 `.github/cla/signatures.json`，首次有人签署时工作流会自动创建。
6. 如果收到企业贡献授权，先线下留档，再把授权编号登记到 [`docs/legal/corporate-authorizations.json`](./docs/legal/corporate-authorizations.json)。

创建 `cla-signatures` 分支的示例命令：

```bash
git switch --create cla-signatures
git commit --allow-empty -m "chore: initialize cla-signatures branch"
git push origin cla-signatures
git switch master
```

如果你的默认分支不是 `master`，最后一步切回对应默认分支即可。

## 说明

- 当前 CLA 流程基于 `contributor-assistant/github-action`。
- 默认只有 bot 账号在 `allowlist` 中自动豁免，维护者和普通开发者都需要至少完成一次真实签署。
- 机器人评论签署即代表你确认接受 [`docs/legal/ICLA.md`](./docs/legal/ICLA.md) 中的条款。
- 企业贡献需要额外的 [`docs/legal/CCLA.md`](./docs/legal/CCLA.md) 或等效书面授权。
- `PR Compliance` 会检查 PR 模板中的身份声明、第三方来源披露、AI 辅助披露、共同作者披露和企业授权编号。
- 对于来源不清、许可证不兼容或披露不完整的贡献，维护者可以直接拒绝或要求补充说明。
