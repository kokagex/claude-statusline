# claude-statusline

Rust で書いた Claude Code 用のシンプルなカスタム statusline。

![statusline](screenshot.jpg)

## 表示内容

| セクション | 内容 |
|-----------|------|
| モデル名 | 短縮名 + コンテキストサイズ (例: `Opus (1M)`) |
| effort | `settings.json` の `effortLevel` を DIM 表示 |
| CTX | コンテキストウィンドウ使用率 |
| 5h / 7d | レートリミット使用率 (5時間 / 7日) |
| ブランチ | 現在の Git ブランチ名 |

使用率は色で段階表示: 🟢 < 50% / 🟡 < 80% / 🔴 >= 80%

## インストール

```bash
git clone git@github.com:kokagex/claude-statusline.git ~/.claude/statusline-rs
cd ~/.claude/statusline-rs
bash setup.sh
```

`setup.sh` がビルドして `~/.claude/settings.json` に statusline コマンドを自動登録します。

## 手動セットアップ

```bash
cargo build --release
```

`~/.claude/settings.json` に以下を追加:

```json
{
  "statusLine": {
    "type": "command",
    "command": "/path/to/.claude/statusline-rs/target/release/statusline"
  }
}
```

## 要件

- Rust 1.70+
- Claude Code
