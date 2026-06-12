# Household Budget App

React + TypeScript と Rust で作成している家計簿アプリです。

収入・支出の登録、一覧表示、検索、月別フィルター、並び替え、ページネーション、収支集計、カテゴリ別集計、予定取引の管理に対応しています。

## 概要

このアプリは、日々の収入・支出を管理するための家計簿アプリです。

単純な取引登録だけではなく、以下のような実用的な運用を想定しています。

- 確定済みの収入・支出を記録する
- 未来の支出予定を「予定取引」として登録する
- 予定取引を実際に発生したタイミングで確定する
- 月別・キーワード・状態で取引を絞り込む
- 収入合計、支出合計、残高を確認する
- カテゴリ別の支出を確認する

## 技術スタック

### Frontend

- Vite
- React
- TypeScript
- React Query
- React Hook Form
- Zod
- Recharts
- Tailwind CSS

### Backend

- Rust
- Axum
- SQLx
- Tokio
- PostgreSQL

### Infrastructure

- Docker
- Docker Compose
- PostgreSQL

## ディレクトリ構成

```txt
household-budget-app
├── frontend
│   ├── src
│   │   ├── app
│   │   └── features
│   │       └── transactions
│   └── package.json
│
├── backend
│   ├── src
│   │   ├── handlers
│   │   ├── models
│   │   ├── repositories
│   │   ├── services
│   │   ├── validators
│   │   ├── errors
│   │   ├── state
│   │   └── main.rs
│   └── Cargo.toml
│
└── docker-compose.yml
```

## 主な機能

### 取引管理

- 取引一覧表示
- 取引登録
- 取引編集
- 取引削除
- 収入 / 支出の管理
- カテゴリ管理
- メモ管理
- 確定済み / 予定の状態管理

### フィルター・検索

- 月別フィルター
- キーワード検索
- 状態フィルター
  - すべて
  - 確定済み
  - 予定
- 並び替え
  - 日付の新しい順
  - 日付の古い順
  - 金額の高い順
  - 金額の低い順

### 集計

- 収入合計
- 支出合計
- 残高
- カテゴリ別支出

### 予定取引

未来の支出や収入を「予定」として登録できます。

予定取引は、実際に発生したタイミングで「確定済み」に変更できます。  
未来日付の予定取引を確定する場合は、日付を今日に変更して確定する導線があります。

## セットアップ

### 前提

以下がインストールされている必要があります。

- Node.js
- npm
- Rust
- Docker
- Docker Compose

## 起動方法

### 1. リポジトリをクローン

```bash
git clone https://github.com/keisukehayano/household-budget-app.git
cd household-budget-app
```

### 2. PostgreSQL を起動

```bash
docker compose up -d
```

PostgreSQL は以下の設定で起動します。

```txt
host: localhost
port: 5432
database: household_budget
user: app_user
password: app_password
```

### 3. 環境変数を設定

プロジェクトルートに `.env` を作成します。

```env
DATABASE_URL=postgres://app_user:app_password@localhost:5432/household_budget
BACKEND_HOST=127.0.0.1
BACKEND_PORT=8080
```

フロントエンド側で API の接続先を明示したい場合は、`frontend/.env` を作成します。

```env
VITE_API_BASE_URL=http://127.0.0.1:8080
```

未設定の場合、フロントエンドは `http://127.0.0.1:8080` に接続します。

### 4. バックエンドを起動

```bash
cd backend
cargo run
```

起動後、以下で疎通確認できます。

```bash
curl http://127.0.0.1:8080/api/health
curl http://127.0.0.1:8080/api/db-health
```

### 5. フロントエンドを起動

別ターミナルで実行します。

```bash
cd frontend
npm install
npm run dev
```

Vite の起動後、表示された URL にアクセスします。

## API エンドポイント

### ヘルスチェック

```txt
GET /api/health
GET /api/db-health
```

### 取引

```txt
GET    /api/transactions
POST   /api/transactions
PUT    /api/transactions/{id}
DELETE /api/transactions/{id}
```

### 集計

```txt
GET /api/transactions/summary
```

## 取引一覧 API のクエリパラメータ

`GET /api/transactions` では以下のパラメータを指定できます。

| パラメータ | 内容 | 例 |
|---|---|---|
| month | 対象月 | `2026-06` |
| q | 検索キーワード | `食費` |
| sort | 並び替え | `date-desc` |
| page | ページ番号 | `1` |
| limit | 1ページ件数 | `10` |
| status | 状態 | `confirmed`, `planned`, `all` |

例:

```bash
curl "http://127.0.0.1:8080/api/transactions?month=2026-06&status=confirmed&page=1&limit=10"
```

## 取引データ

取引は以下のようなデータを持ちます。

```json
{
  "id": "uuid",
  "type": "expense",
  "date": "2026-06-12",
  "category": "food",
  "amount": 1200,
  "memo": "昼食",
  "status": "confirmed",
  "createdAt": "2026-06-12T00:00:00Z",
  "updatedAt": "2026-06-12T00:00:00Z"
}
```

### type

| 値 | 内容 |
|---|---|
| income | 収入 |
| expense | 支出 |

### category

| 値 | 内容 |
|---|---|
| food | 食費 |
| daily | 日用品 |
| transport | 交通費 |
| entertainment | 娯楽 |
| salary | 給与 |
| other | その他 |

### status

| 値 | 内容 |
|---|---|
| confirmed | 確定済み |
| planned | 予定 |

## 開発用コマンド

### Frontend

```bash
cd frontend

npm run dev
npm run build
npm run lint
npm run preview
```

### Backend

```bash
cd backend

cargo run
cargo test
cargo fmt
cargo clippy
```

## 今後の実装予定

- 認証機能
- ユーザーごとの取引管理
- カテゴリ管理機能
- 予算管理機能
- グラフ表示の強化
- CSV インポート / エクスポート
- 定期取引の自動生成
- スマートフォン表示の改善
- 本番環境向け CORS 設定
- API エラーハンドリングの改善

## 開発目的

このプロジェクトは、React + TypeScript によるフロントエンド開発と、Rust によるバックエンド API 開発を学習することを目的としています。

家計簿という身近な題材を使いながら、以下を実践的に学ぶためのアプリです。

- フロントエンドのコンポーネント設計
- TypeScript による型安全な開発
- Rust/Axum による REST API 開発
- PostgreSQL を使ったデータ永続化
- フロントエンドとバックエンドの API 連携
- バリデーション
- 検索・フィルター・ページネーション
- 集計処理
