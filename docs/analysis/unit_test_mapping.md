# 單元測試對照表

本文件根據「三向測試原則」整理目前建議的單元測試對象。

## utils/text_processing.rs

- Happy Path：`preprocess_text` / `postprocess_text` 正常配對
- Edge：UTF-8 字元、空白、特殊符號
- Robustness：`detect_loop` 對重複字串

## utils/skip_rules.rs

- Happy Path：一般文字不被跳過
- Edge：snake_case、命名空間 ID
- Robustness：空字串與純數字

## translation/glossary/automaton.rs

- Happy Path：大小寫不敏感匹配
- Edge：表情符號與複合 UTF-8
- Robustness：循環定義不影響匹配

## translation/api/client.rs

- Happy Path：`extract_json_from_text` 解析完整 JSON
- Edge：Markdown code block 內 JSON
- Robustness：缺尾括號自我修復

## file/scanner.rs

- Happy Path：目錄遞迴收集 `.jar`/`.json`/`.js`
- Edge：Windows 路徑大小寫差異
- Robustness：`strip_prefix` 失敗回退
