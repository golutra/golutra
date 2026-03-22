# Corporate Authorization Registry

本文件用于说明企业贡献授权的维护规则。

机器可读的授权源文件位于：

- [`corporate-authorizations.json`](./corporate-authorizations.json)

## 规则

- 授权编号建议采用 `CCA-YYYY-NNN` 格式，例如 `CCA-2026-001`。
- 只有维护者确认收到有效 `CCLA` 或等效书面授权后，才应在 `corporate-authorizations.json` 中新增记录。
- PR 中如果声明“代表组织贡献”，必须填写这里已有的授权编号。
- `authorizedGitHubUsernames` 用于声明该授权覆盖的 GitHub 用户名。
- `authorizedEmails` 用于声明该授权覆盖的共同作者邮箱；如果 PR 中存在 `Co-authored-by:`，这些邮箱也必须被覆盖。
- `status` 建议使用 `Active`、`Expired`、`Revoked` 这类明确状态值；只有 `Active` 会被工作流放行。
- 授权过期、撤销或范围变更时，应及时更新状态与备注。

## JSON 结构示例

```json
{
  "version": 1,
  "authorizations": [
    {
      "authorizationReference": "CCA-2026-001",
      "organization": "Example Corp",
      "authorizedGitHubUsernames": ["alice", "bob"],
      "authorizedEmails": ["alice@example.com", "bob@example.com"],
      "effectiveDate": "2026-03-21",
      "expirationDate": "2027-03-21",
      "status": "Active",
      "notes": "Optional notes"
    }
  ]
}
```
