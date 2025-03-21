# rust-style-factory

一个 Rust 实现的样式处理工具。

## 功能特性

- CSS 到代码的转换
- 样式工厂模式
- Token 转换支持

## 安装

```bash
cargo add rust-style-factory
```

## 使用方法

```rust
use rust_style_factory::style_factory;

fn main() {
    // 使用样式工厂创建样式
    let style = style_factory();
    // ...
}
```

## 模块说明

- `css_to_code`: CSS 转换为代码的实现
- `style_factory`: 样式工厂核心实现
- `convert_token`: Token 转换工具

## 许可证

GNU AGPL v3 - 查看 [LICENSE](LICENSE) 文件了解更多详情

此许可证要求：
- 必须开源
- 必须保留版权信息
- 修改后的软件也必须使用相同许可证
- 网络服务也需要开源
