以下是針對 mc_translator 專案的 CI/CD 優化建議（精簡版，只包含優化建議與具體細節）。我已根據專案實際結構（Rust + Tauri GUI + 獨立 CLI、frontend 使用 Vitest）、現有 workflow（rust.yml、frontend.yml、miri.yml、dependabot.yml）、git-cliff 自動化 changelog，以及目前 release 情況（v0.7.3 為最新，主要產生 Windows .exe，尚未完整多平台資產）整理出以下建議。1. 最高優先建議（建議 1–3 天內執行）合併分散的 workflow 成單一主 CI目前 rust.yml、frontend.yml、miri.yml 分開執行，造成重複安裝 Rust/Node、cache 浪費。
具體做法：建立 .github/workflows/ci.yml，使用 matrix 策略同時測試多平台。

推薦 matrix 配置：yaml

strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]

核心檢查步驟（建議包含）：Rust：cargo fmt --all -- --check、cargo clippy --all-targets -- -D warnings、cargo test --all-targets（使用 cargo-nextest）
Frontend：npm ci、npm run lint、npm run test（Vitest + coverage）
安全檢查（只在 ubuntu 跑一次）：cargo audit、cargo deny check
覆蓋率：llvm-cov + Codecov（已準備 codecov.yml，可直接整合）

把 cargo fmt、clippy、lint 設為 Required status checks（Repository Settings → Branches）。2. 跨平台與建置優化（短期，1 周內）擴展 Release 到完整多平台目前主要只產出 Windows .exe（GUI + CLI）。
建議：使用 tauri-apps/tauri-action 建立多平台 matrix，同時產生：Windows：.exe + NSIS installer
Linux：.AppImage + .deb
macOS：.app + .dmg

推薦 release workflow 結構（新建 release.yml，觸發 push tags: ['v*']）：yaml

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
          - os: windows-latest
          - os: macos-latest
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }   # git-cliff 需要完整歷史
      - name: Setup Rust & Node
      - name: Build CLI
        run: cargo build --release --bin mc_translator_cli
      - name: Build Tauri GUI
        uses: tauri-apps/tauri-action@v0
        with:
          tagName: ${{ github.ref_name }}
          releaseName: "mc_translator ${{ github.ref_name }}"
      - name: Generate changelog
        uses: orhun/git-cliff-action@v2
        with:
          config: cliff.toml
      - name: Upload assets
        uses: softprops/action-gh-release@v2

這能讓 push tag 後自動產生跨平台資產並上傳到 GitHub Releases。區分 PR 與 Release 流程：PR / push main：只跑 check、test、lint（快速反饋）
push tag：完整 build + 多平台 bundling

3. 測試與品質強化Miri 整合：目前為獨立 miri.yml，建議改為 schedule（每週一次）或只在 main push 時執行，避免 PR 變慢。
E2E 測試：目前較弱，建議增加：使用 tauri-driver + Playwright 做 GUI E2E（重點測試 Windows GUI）。

覆蓋率：Rust（llvm-cov）與 Frontend（Vitest）已支援，建議在 CI 中強制設定最低覆蓋率門檻（透過 codecov.yml）。
預覽版（Canary）：每次 push main 時，額外產生標記為 canary 的 artifact，讓社群快速測試最新版。

4. 其他實用細節優化效能提升：Rust cache：使用 Swatinem/rust-cache@v2
Node cache：actions/setup-node with cache: 'npm'
可考慮將 npm 換成 pnpm（更快、更省空間）

Branch Protection Rules（強烈建議開啟）：Require a pull request before merging
Require status checks to pass（包含 fmt、clippy、lint、test）
Dismiss stale approvals

安全性：保留現有的 cargo audit、cargo-deny、dependabot
可新增 CodeQL（GitHub 內建）掃描

版本與 Changelog：現有 cliff.toml + git-cliff 已不錯，可繼續使用 conventional commits 風格。
未來可考慮引入 release-please 實現全自動版本 bump + changelog + tag。

執行優先序總結立即：合併成 ci.yml + matrix + 強制 lint/fmt/clippy
短期：擴展多平台 Release（Linux + macOS） + 優化 cache
中期：增加 GUI E2E + Canary 版本
長期：完整自動化版本管理 + 通知機制


name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
      fail-fast: false

    runs-on: ${{ matrix.os }}
    timeout-minutes: 45

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      # Rust 環境設定
      - name: Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Rust cache
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: |
            .
            src-tauri

      # Node 環境設定
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: 'npm'
          cache-dependency-path: package-lock.json

      - name: Install frontend dependencies
        run: npm ci

      # ==================== Rust 檢查 ====================
      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run Rust tests (nextest)
        run: cargo install cargo-nextest --locked && cargo nextest run --all-targets --all-features

      # ==================== Frontend 檢查 ====================
      - name: Frontend lint & type check
        run: |
          npm run lint || true          # 如果你還沒設定 lint，可先註解
          npm run type-check || true    # 如果有 tsc 或類似指令

      - name: Frontend Vitest tests + coverage
        run: npm run test:frontend -- --coverage

      # ==================== 安全性檢查（只在 ubuntu 執行一次） ====================
      - name: Security & dependency checks
        if: matrix.os == 'ubuntu-latest'
        run: |
          cargo install cargo-audit cargo-deny --locked
          cargo audit
          cargo deny check

      # ==================== 覆蓋率上傳 ====================
      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v5
        with:
          files: ./lcov.info,./coverage/coverage-final.json   # 根據你的實際輸出調整
          fail_ci_if_error: false
          token: ${{ secrets.CODECOV_TOKEN }}   # 可選，若已設定 GitHub token 可省略

      # ==================== GUI E2E 測試（重點：Windows） ====================
      - name: Install tauri-driver
        if: matrix.os == 'windows-latest'
        run: cargo install tauri-driver

      - name: Install Playwright browsers
        if: matrix.os == 'windows-latest'
        run: npx playwright install --with-deps chromium

      - name: Build Tauri app in debug mode for E2E
        if: matrix.os == 'windows-latest'
        run: npm run tauri build -- --debug

      - name: Run GUI E2E Tests (Playwright + tauri-driver)
        if: matrix.os == 'windows-latest'
        run: npx playwright test
        env:
          TAURI_WEBVIEW_AUTOMATION: 1
          # 如果你的測試需要測試用 API Key 或其他變數，可在此新增
          # EXAMPLE_API_KEY: ${{ secrets.TEST_API_KEY }}

      # ==================== 額外：macOS / Linux 基本建置檢查 ====================
      - name: Build CLI (cross-platform check)
        if: matrix.os != 'windows-latest'
        run: cargo build --release --bin mc_translator_cli

      - name: Build Tauri debug (non-Windows)
        if: matrix.os != 'windows-latest'
        run: npm run tauri build -- --debug
