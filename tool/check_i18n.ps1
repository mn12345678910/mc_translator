# Minecraft 模組翻譯器 - I18n 同步檢查工具
# 本腳本旨在快速檢查 JSON 資產與 Rust 結構體是否一致

Write-Host "🔍 正在檢查 I18n 資產對齊情況..." -ForegroundColor Cyan

# 執行 Cargo Test 中的專屬測試案例
cargo test test_ensure_assets_alignment -- --nocapture

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ 檢查通過！所有 JSON 鍵值皆已在 Rust 結構體中定義。" -ForegroundColor Green
} else {
    Write-Host "❌ 檢查失敗！偵測到未定義的鍵值，請更新 src/i18n.rs 中的結構體。" -ForegroundColor Red
    Write-Host "提示：查看上方錯誤訊息中的 'Unknown field' 以定位缺失欄位。" -ForegroundColor Yellow
    exit 1
}
