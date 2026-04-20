## 資料庫設計

本專案採用 **MongoDB** 作為主資料庫（資料庫名稱：`teamder`），共包含 5 個 collection。所有文件主鍵皆使用 UUID v4 字串作為 `_id`，並由後端 Rust 程式碼（`teamder-core` crate）定義型別。

### 1. `users` — 使用者

| 欄位 | 型別 | 說明 |
|---|---|---|
| `_id` | `String` (UUID v4) | 主鍵 |
| `email` | `String` | 登入信箱，unique |
| `password_hash` | `String` | bcrypt 雜湊，API 回傳時不外流 |
| `name` | `String` | 顯示名稱 |
| `initials` | `String` | 依 `name` 自動產生的縮寫（最多 2 字） |
| `role` | `String` | 自我介紹職稱 |
| `department` | `String` | 科系 |
| `university` | `String` | 學校（預設 `Fu Jen Catholic University`） |
| `year` | `String` | 年級 |
| `location` | `String?` | 所在地 |
| `bio` | `String[]` | 自我介紹段落 |
| `skills` | `{name: String, level: u8 (0–100)}[]` | 技能與熟練度 |
| `skill_tags` | `String[]` | 技能標籤（搜尋用） |
| `gradient` | `String` | 頭像漸層色 CSS |
| `work_mode` | enum `remote` \| `hybrid` \| `in_person` | 協作模式 |
| `availability` | enum `open_for_collab` \| `busy` \| `unavailable` | 可合作狀態 |
| `hours_per_week` | `String` | 每週可投入時數 |
| `languages` | `String[]` | 使用語言 |
| `portfolio` | `{title, kind, description?, url?}[]` | 作品集 |
| `reviews` | `{reviewer_id, reviewer_name, project_name, stars 1–5, body, created_at}[]` | 合作評價 |
| `match_score` | `u8` | 媒合分數 |
| `rating` | `f32` | 平均評分 |
| `projects_done` | `u32` | 完成專案數 |
| `collaborations` | `u32` | 合作次數 |
| `is_admin` | `bool` | 是否為管理員 |
| `created_at` / `updated_at` | `DateTime<Utc>` | 時間戳記 |

### 2. `projects` — 專案

| 欄位 | 型別 | 說明 |
|---|---|---|
| `_id` | `String` (UUID v4) | 主鍵 |
| `name` | `String` | 專案名稱 |
| `lead_user_id` | `String` | 領導人 `users._id` |
| `lead_name` | `String` | 領導人顯示名（快照） |
| `icon` / `icon_bg` | `String` | 顯示圖示與背景 |
| `status` | enum `recruiting` \| `active` \| `completed` \| `cancelled` | 專案狀態 |
| `description` | `String` | 專案介紹 |
| `goals` | `String?` | 目標說明 |
| `roles` | `{name, count_needed, filled}[]` | 徵才職位 |
| `skills` | `String[]` | 所需技能標籤 |
| `team` | `{user_id, initials, color, joined_at}[]` | 成員清單 |
| `deadline` | `String?` | 截止日期（自由字串） |
| `collab` | enum `remote` \| `hybrid` \| `in_person` | 協作模式 |
| `duration` | `String?` | 預計時程 |
| `category` | `String?` | 分類 |
| `is_public` | `bool` | 是否公開 |
| `created_at` / `updated_at` | `DateTime<Utc>` | 時間戳記 |

### 3. `competitions` — 競賽

| 欄位 | 型別 | 說明 |
|---|---|---|
| `_id` | `String` (UUID v4) | 主鍵 |
| `name` | `String` | 競賽名稱 |
| `organizer` | `String` | 主辦單位 |
| `icon` / `icon_bg` | `String` | 顯示圖示與背景 |
| `status` | enum `open` \| `closing_soon` \| `upcoming` \| `past` | 報名狀態 |
| `prize` | `String` | 獎金/獎項 |
| `team_size_min` | `u8` | 最少人數 |
| `team_size_max` | `u8` | 最多人數 |
| `deadline` | `String?` | 報名截止 |
| `duration` | `String` | 競賽時程 |
| `tags` | `String[]` | 分類標籤 |
| `description` | `String` | 說明 |
| `is_featured` | `bool` | 是否精選 |
| `registrations` | `{user_id, team_name?, registered_at}[]` | 報名紀錄 |
| `created_at` / `updated_at` | `DateTime<Utc>` | 時間戳記 |

### 4. `study_groups` — 讀書會

| 欄位 | 型別 | 說明 |
|---|---|---|
| `_id` | `String` (UUID v4) | 主鍵 |
| `name` | `String` | 讀書會名稱 |
| `goal` | `String` | 學習目標 |
| `icon` / `icon_bg` | `String` | 顯示圖示與背景 |
| `subject` | `String` | 主題科目 |
| `tags` | `String[]` | 標籤 |
| `members` | `{user_id, initials, color, joined_at, last_checkin?, streak}[]` | 成員清單 |
| `max_members` | `u8` | 人數上限（預設 6） |
| `schedule` | `String` | 聚會時間 |
| `duration_weeks` | `u8` | 總週數 |
| `current_week` | `u8` | 目前週次 |
| `is_open` | `bool` | 是否開放加入 |
| `created_by` | `String` | 建立者 `users._id` |
| `created_at` / `updated_at` | `DateTime<Utc>` | 時間戳記 |

### 5. `invites` — 邀請

| 欄位 | 型別 | 說明 |
|---|---|---|
| `_id` | `String` (UUID v4) | 主鍵 |
| `from_user_id` | `String` | 發送人 `users._id` |
| `from_user_name` | `String` | 發送人顯示名（快照） |
| `to_user_id` | `String` | 接收人 `users._id` |
| `project_id` | `String?` | 關聯專案（可選） |
| `project_name` | `String?` | 專案名稱（快照） |
| `message` | `String?` | 附加訊息 |
| `status` | enum `pending` \| `accepted` \| `declined` \| `expired` | 邀請狀態 |
| `created_at` | `DateTime<Utc>` | 建立時間 |
| `expires_at` | `DateTime<Utc>` | 過期時間（預設建立後 7 天） |

### 設計備註
- **欄位命名**：Rust 端使用 `snake_case`；enum 透過 `#[serde(rename_all = "snake_case")]` 序列化，例如 `WorkMode::InPerson` → `"in_person"`。
- **主鍵策略**：以 UUID v4 而非 MongoDB ObjectId，方便跨服務追蹤與前端 URL 使用。
- **回傳 DTO**：每個 collection 都有對應的 `*Response` 結構（例如 `UserResponse` 不含 `password_hash`、`CompetitionResponse` 以 `registration_count` 取代完整 `registrations[]`），避免敏感或冗餘欄位外流。
- **參照關係**：以字串 ID 互相參照（如 `projects.team[].user_id` 指向 `users._id`），不使用 MongoDB DBRef；一致性由應用層維護。