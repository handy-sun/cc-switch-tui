# Hermes YAML 序列缩进设计

## 目标

Hermes 写回 `config.yaml` 的任意顶层 section 时，YAML 序列项的 `-` 相对父 key 多缩进一级（两个空格），与 Hermes 官方配置样式一致。嵌套序列按相同规则逐层缩进。

示例：

```yaml
custom_providers:
  - name: foo
    models:
      - id: model-a
```

不再输出：

```yaml
custom_providers:
- name: foo
  models:
  - id: model-a
```

## 实现边界

- 修改 Hermes 的 section 序列化出口 `serialize_yaml_section`，在 `serde_yaml` 生成内容后规范序列及其子节点的缩进。
- 保持现有 section 级替换、备份、原子写入和未修改 section 的文本保留逻辑不变。
- 不替换 YAML 依赖，不重写完整配置文件，不改变解析后的 YAML 数据结构或字段顺序。

## 格式化规则

- 每遇到一个由 `serde_yaml` 输出的无缩进序列层级，该序列及其所有子节点增加两个空格。
- 同一序列的相邻项目共享同一缩进层级。
- 离开序列后，后续同级映射恢复父级缩进。
- 顶层序列和映射内部的嵌套序列使用同一规则。

## 验证

- 添加回归测试，直接检查顶层序列的 `-` 比父 key 多两个空格。
- 添加嵌套序列断言，检查每个嵌套层级继续多缩进两个空格，同时序列项内的映射字段保持正确对齐。
- 将格式化后的文本重新解析为 YAML，并与输入值比较，证明仅改变样式、不改变数据。
- 运行 Hermes 模块测试及完整 Rust 测试集。
