# Hermes 模型参数编辑与持久化设计

## 背景与根因

Hermes provider 表单目前复用了 OpenClaw 的模型编辑字段。表单把上下文长度写成
`contextWindow`，但 Hermes 的 `custom_providers[].models` 使用 `context_length`；现有
Hermes 清洗逻辑只转换 provider 顶层字段，没有转换每个模型对象，因此 UI 看似保存成功，
Hermes 实际不会采用该参数。此外，`max_tokens` 和多模型管理仍依赖手工编辑 JSON。

## 目标

- Hermes provider 使用独立、结构化的模型编辑交互。
- 支持添加、编辑、删除多个模型。
- 每个模型直接编辑 `id`、`context_length` 和 `max_tokens`。
- 保存后数据库快照与 `~/.hermes/config.yaml` 使用 Hermes 原生字段。
- 编辑已存在的配置时保留模型对象中的未知高级字段。
- JSON 预览继续作为高级兜底，但常规模型设置不再依赖手工 JSON。

## 非目标

- 不把 Hermes 所有未来 provider 字段一次性表单化。
- 不改变 Hermes `providers:` 字典来源条目的只读规则。
- 不改变 OpenClaw、OpenCode 或其它应用的模型编辑行为。

## 交互设计

Hermes provider 表单保留名称、网站、备注、API key、Base URL 等已有字段，并用专用的
“模型”行打开模型列表编辑器。列表中的每行对应一个模型，支持新增、编辑和删除。

模型编辑器提供三个字段：

- 模型 ID：必填，保存时去除首尾空白；同一 provider 内必须唯一。
- 上下文长度：可选正整数，对应 `context_length`。
- 最大输出 Token：可选正整数，对应 `max_tokens`。

空白数值表示删除该字段。无效数字、零、重复或空白模型 ID 会阻止保存并显示错误。
编辑器不展示未知高级字段，但更新已知字段时必须原样保留它们。

Hermes 不再显示从 OpenClaw 借用的 User-Agent 开关。API 模式继续沿用现有 provider 字段，
但输出必须符合 Hermes 的 snake_case schema。

## 数据流

1. 打开 provider 时，将 `settingsConfig.models` 数组加载为模型编辑器状态，同时接受历史
   `contextWindow` / `context_window` / `maxTokens` 别名以便迁移。
2. 表单保存时，把编辑器状态写回 `settingsConfig.models`，统一使用 `context_length` 和
   `max_tokens`，并保留每个模型的其它字段。
3. `ProviderService::update` 保存数据库快照；Hermes live 写入把数组转换为 YAML 字典，模型
   ID 作为字典 key。
4. 当前 provider 更新或切换后，顶层 `model.provider` 和 `model.default` 继续由现有
   `apply_switch_defaults` 维护。

## 兼容与错误处理

- 读取历史 camelCase 字段时自动迁移，写回只生成 Hermes 原生字段。
- 未知模型字段和 provider 字段保持不变。
- `models` 不符合数组/字典形状时不静默覆盖；结构化表单保存前返回明确错误。
- 没有模型时允许保存 provider，但切换时继续沿用现有 Hermes 默认模型回退语义。

## 测试

- 回归测试先证明当前 Hermes 表单把 `contextWindow` 写入快照而非 `context_length`。
- 表单序列化测试覆盖 `context_length`、`max_tokens`、别名迁移和未知字段保留。
- 模型编辑器交互测试覆盖新增、编辑、删除、重复 ID 和数字校验。
- provider service 测试覆盖数据库快照与 Hermes YAML 的最终字段形状。
- 运行相关模块测试、完整串行 Rust 测试、格式和静态检查。
