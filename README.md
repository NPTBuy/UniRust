# UniRust

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20x64-lightgrey.svg)]()
[![Binary Size](https://img.shields.io/badge/Size-~177%20KB-success.svg)]()
[![Tests](https://img.shields.io/badge/Tests-19%2F19%20Passing-brightgreen.svg)]()

**[Tiếng Việt](#tiếng-việt) | [English](#english)**

---

<a name="tiếng-việt"></a>
## Tiếng Việt

> [!IMPORTANT]
> **Tuyên bố về mục đích học tập, tính phi thương mại & cam kết an toàn bảo mật:**
>
> 1. **Mục đích học tập & thử nghiệm cá nhân:** UniRust là một dự án nghiên cứu và thử nghiệm cá nhân được tạo ra với mục tiêu học tập, thực hành xây dựng và vận hành ngôn ngữ lập trình **Rust**, đồng thời thử nghiệm ứng dụng quy trình làm việc [REA (Reverse-Engineer-Anything)](https://github.com/teonett/Reverse-Engineer-Anything) thông qua trợ lý tác nhân AI (AI Agent).
> 2. **Tính phi thương mại & phi cạnh tranh:** Dự án này **hoàn toàn phi thương mại**, tuyệt đối không mang mục đích cạnh tranh, so bì hay thay thế bộ gõ **UniKey** (hoặc bất kỳ bộ gõ tiếng Việt nào khác) và không có bất kỳ mưu đồ thương mại nào. Người dùng phổ thông được khuyến nghị tiếp tục sử dụng bản phát hành chính thức của bộ gõ [UniKey](https://www.unikey.org) của tác giả Phạm Kim Long để phục vụ công việc hàng ngày một cách ổn định nhất.
> 3. **Cam kết minh bạch & an toàn (Không mã độc / Không keylogger):** Do UniRust sử dụng cơ chế Windows Low-Level Keyboard Hook (`WH_KEYBOARD_LL`) để chặn và phân tích các phím gõ Telex, các công cụ bảo mật hoặc người dùng có thể băn khoăn về cơ chế này. Dự án đưa ra những cam kết kỹ thuật minh bạch và hoàn toàn có thể tự kiểm chứng:
>    - **100% Offline (Hoạt động độc lập không mạng):** Mã nguồn không chứa bất kỳ dependency mạng nào, không có socket, không có mã HTTP/truyền tải dữ liệu, không có cơ chế thu thập thống kê (telemetry). Tuyệt đối không có bất kỳ byte dữ liệu nào rời khỏi máy tính của bạn.
>    - **Zero Keystroke Logging (Tuyệt đối không ghi phím):** Không bao giờ ghi lại phím bấm của người dùng vào tệp tin, ổ đĩa hay bất kỳ bộ nhớ lưu trữ cố định nào. Bộ nhớ đệm phân tích âm tiết và lịch sử từ ngắn (tối đa 8 từ) chỉ tồn tại tạm thời trong bộ nhớ RAM khả biến (volatile RAM) và được xóa sạch lập tức khi gặp ranh giới từ (Space/Enter), cú nhấp chuột, chuyển cửa sổ hoặc phím tắt.
>    - **Mã nguồn mở có thể kiểm định 100% (GNU GPL v3.0):** Toàn bộ mã nguồn được công khai minh bạch. Người dùng được khuyến khích tự đọc mã nguồn Rust và tự biên dịch tệp thực thi bằng lệnh `cargo build --release` thay vì tải tệp nhị phân có sẵn nếu có bất kỳ nghi ngại nào.

---

### 1. Tổng quan dự án (Project Overview)

**UniRust** là bộ gõ tiếng Việt (Telex) mã nguồn mở, được viết hoàn toàn bằng ngôn ngữ lập trình **Rust** cho hệ điều hành Windows x64. Dự án hướng tới tiêu chí: **siêu nhẹ, tốc độ phản hồi cực nhanh, không phụ thuộc thư viện ngoài (zero external DLL/C-runtime dependencies)** và xuất chuẩn ký tự **Unicode dựng sẵn (NFC)** nguyên bản.

- **Kích thước siêu nhỏ gọn:** Tệp thực thi độc lập (standalone `.exe`) chỉ khoảng **~177 KB** (181.248 bytes ở bản release), không cần cài đặt, không cần .NET Framework hay Visual C++ Redistributable.
- **Bộ nhớ siêu nhẹ & Ổn định (RAM ~2–4 MB):** Khởi tạo lười UI Automation (Lazy UIA) với bộ lọc nhanh Win32 class `GetClassNameW` (chỉ kích hoạt UIA khi người dùng thực sự focus vào Chrome/Edge/Firefox), hot path tái sử dụng bộ đệm zero-heap-allocation, và cơ chế One-Shot Working Set Trim tự động thu hồi bộ nhớ sau 2.5s khởi động mà không cần phụ thuộc `psapi.dll`.
- **Không inject DLL:** Chạy hoàn toàn dưới dạng tiến trình nền độc lập thông qua Win32 Low-Level Hooks (`WH_KEYBOARD_LL`, `WH_MOUSE_LL`).
- **Unicode NFC chuẩn mực (Zero AI Token Bloat):** Tạo trực tiếp các ký tự tiếng Việt dựng sẵn (pre-composed NFC), tuyệt đối không sinh ký tự tổ hợp rời rạc (NFD combining marks `U+0300..=U+036F`) hay các ký tự ẩn độ rộng bằng không (`U+200B`, `U+2060`, `U+FEFF`). Điều này giúp tránh triệt để tình trạng phình to token (token bloat) hoặc lỗi xử lý ngữ cảnh khi nhập liệu vào các mô hình AI/LLM (ChatGPT, Claude, Gemini, Cursor, Copilot).

---

### 2. Nguồn gốc, Bản quyền & Lời tri ân đến UniKey (Attribution & Respect for UniKey)

UniRust là bản tái hiện thuần Rust (pure-Rust re-implementation) được phát triển dựa trên việc nghiên cứu, tham chiếu và kế thừa tinh hoa giải thuật từ bộ gõ tiếng Việt huyền thoại **UniKey** do tác giả **Phạm Kim Long** sáng lập ([https://www.unikey.org](https://www.unikey.org)).

Dự án đã khảo sát và đối chiếu trực tiếp các tệp mã nguồn chính thức từ bản phát hành **x-unikey-1.0.4** (được cấp phép theo GNU GPL/LGPL v2.0-or-later):
- `ukengine.cpp` & `ukengine.h`: Máy trạng thái xử lý âm tiết tiếng Việt, kiểm tra phụ âm đầu, vần, quy tắc biến âm và hoàn tác phím kép.
- `data.cpp`: Bảng tra cứu âm tiết, chuyển đổi dấu thanh và ánh xạ 134 ký tự tiếng Việt Unicode dựng sẵn (NFC).
- `vnlexi.h`: Định nghĩa ngữ pháp, danh sách tổ hợp nguyên âm và phụ âm hợp lệ.

#### Các quy tắc cốt lõi đã được chuyển thể hoàn chỉnh sang Rust:
1. **Bảng ánh xạ 134 ký tự Unicode NFC:** Hỗ trợ đầy đủ toàn bộ bảng chữ cái tiếng Việt với 5 thanh điệu (sắc, huyền, hỏi, ngã, nặng) và các dấu phụ mũ (`â, ê, ô`), móc (`ư, ơ`), trăng (`ă`).
2. **Quy tắc phụ âm đầu đặc biệt (`qu`, `gi`):**
   - Với `qu-`: `u` được coi là một phần của phụ âm đầu, do đó dấu thanh được đặt chính xác vào nguyên âm sau (`quả`, `quý`, `quốc`).
   - Với `gi-`: Phân biệt rõ giữa các từ ghép (`già`, `giá` đặt dấu vào `a`) và từ đơn đặc biệt (`gì` đặt dấu vào `i`).
3. **Quy tắc gõ phím `w` và nguyên âm ghép:**
   - `uo + w -> ươ` (ví dụ: *trường, nước, hươu, rượu*).
   - `ua + w -> ưa` (ví dụ: *sửa, mưa, cửa* - không bị nhầm thành `uă`).
   - Phím `w` đứng đầu từ tự động chuyển thành `ư` (*ừ, ứ*); gõ lặp lại `ww` sẽ hoàn tác trả về ký tự `w` nguyên bản.
   - `u + w -> ư`, gõ thêm `w` (`uw + w`) sẽ hoàn tác về `uw`.
4. **Quy tắc phụ âm cuối hạn chế thanh điệu (Stop-consonants):**
   - Các từ có phụ âm cuối tắc vô thanh (`c`, `ch`, `p`, `t`) chỉ cho phép mang thanh Sắc (`s`) hoặc Nặng (`j`) (ví dụ: *bác, bạc, sách, sạch, tháp, thạp, cát, cạt*).
5. **Bảo toàn từ tiếng Anh chứa `w`:**
   - Các từ tiếng Anh phổ biến như *new, view, crew, draw, show* không bị biến âm sai thành `ư`/`ơ`.
6. **Đặt dấu thanh tự do (Free-tone placement):**
   - Người dùng có thể gõ dấu thanh ở bất kỳ thời điểm nào trong từ; bộ gõ tự động phân tích cấu trúc nguyên âm đôi/nguyên âm ba và phụ âm cuối để đặt dấu đúng chuẩn ngữ pháp chính tả tiếng Việt (*hoà/hòa, thuỷ/thủy, người, tiếng, toán, hoàng*).
7. **Hoàn tác phím kép bảo toàn chữ hoa/thường (Double-key undo):**
   - Gõ lặp lại phím biến âm hoặc phím dấu (`aa, ee, oo, dd, ww, ss, ff, rr, xx, jj`) sẽ hoàn tác đúng ký tự/dấu đó về nguyên bản mà vẫn giữ nguyên trạng thái hoa/thường (*Trường, ĐẮK LẮK, VIỆT NAM*).
8. **Quy tắc phím `z` xóa dấu & hoàn tác chuẩn UniKey:**
   - Phím `z` chỉ kích hoạt xóa dấu khi âm tiết hiện tại đang thực sự mang thanh điệu (`ras` -> `rá`, `+ z` -> `ra`). Nếu từ chưa có dấu (`raz`, `zoo`, `quiz`), ký tự `z` được phát ra nguyên bản.
   - Hoàn tác phím kép `zz`: Nhấn `z` lần thứ hai sau khi vừa xóa dấu sẽ khôi phục lại thanh điệu và chèn `z` (`raszz` -> `ráz`).
   - Khi xuất hiện `z` nguyên bản, từ được tự động nhận diện là từ ngoại lai (tiếng Anh) để ngăn chặn các phím Telex tiếp theo biến âm ngược (`razer`, `azure`, `size`, `crazy`, `frozen`, `pizza`, `buzz`).

---

### 3. Tác giả & Quy trình phát triển cùng AI (Author & AI Workflow)

- **Chủ sở hữu dự án & Người yêu cầu (Project Owner / Requester):** **Phạm Ngọc Trí (NPTBuy)**
  - GitHub: [https://github.com/NPTBuy](https://github.com/NPTBuy)
  - Email: [tri.phamngoc2006@gmail.com](mailto:tri.phamngoc2006@gmail.com)
  - **Phân định vai trò:** Chủ trì quá trình học hỏi và nghiên cứu Rust, định hướng ý tưởng, xác lập yêu cầu kỹ thuật chi tiết, trực tiếp thực hiện gõ thực tế trên máy tính để phát hiện các ca biên (edge-case typing tests) và kiểm định, đánh giá toàn bộ kết quả đầu ra.
- **Quy trình phát triển hỗ trợ bởi AI (AI-Assisted Workflow):**
  - **Gemini:** Hỗ trợ lên ý tưởng, phân tích mã nguồn UniKey, truy tìm nguyên nhân gốc rễ (root-cause debugging) của các lỗi xung đột phím gõ nhanh và thiết kế kiến trúc máy trạng thái âm tiết.
  - **Antigravity:** Môi trường thực thi quy trình [REA (Reverse-Engineer-Anything)](https://github.com/teonett/Reverse-Engineer-Anything) với tác nhân AI để lập trình mã nguồn Rust, tái cấu trúc (refactoring) Win32 hooks, tối ưu hóa atomic batching, xây dựng bộ kiểm thử tự động (test harness) và đóng gói bản phát hành release.

---

### 4. Kiến trúc hệ thống & Tính năng nổi bật (Architecture & Key Features)

```
┌────────────────────────────────────────────────────────────────────────┐
│                        UniRust Architecture                           │
├────────────────────────────────────────────────────────────────────────┤
│  [Physical Keystroke]                                                  │
│          │                                                             │
│          ▼                                                             │
│  [WH_KEYBOARD_LL Hook] ──(Loopback filter: dwExtraInfo == 0x554E494B)  │
│          │                                                             │
│          ├─► Desync Guards (Mouse click, Focus change, Ctrl/Alt, Nav) │
│          │                                                             │
│          ▼                                                             │
│  [Syllable Engine (telex.rs & unicode_map.rs)]                         │
│    - Initial Consonant + Rhyme + Tone + Keystroke History              │
│    - Smart Backspace Popping & Word History Stack (Cap 8 words)        │
│          │                                                             │
│          ├──────────────────────────┬──────────────────────────┐       │
│          ▼                          ▼                          ▼       │
│    PassThrough                    Reset                    Transform   │
│   (Send to OS)                (Clear State)                    │       │
│                                                                ▼       │
│                                                   [IS_OMNIBOX_FOCUSED] │
│                                                   (AtomicBool / UIA)   │
│                                                    ├─────────┴───────┐ │
│                                                    ▼                 ▼ │
│                                              Normal Inputs        Omnibox
│                                            (Atomic Batch:        (Cancel
│                                            Zero VK_SPACE,       VK_SPACE+
│                                             Zero Sleep)         VK_BACK)
└────────────────────────────────────────────────────────────────────────┘
```

1. **Khay hệ thống Win32 GDI vẽ trong bộ nhớ (In-memory Tray Icon):**
   - Biểu tượng chữ **V** (tiếng Việt, nền đỏ) hoặc **E** (tiếng Anh, nền xanh) được vẽ trực tiếp bằng Win32 GDI vào bộ nhớ bitmap và gán cho `Shell_NotifyIconW`.
   - Không cần nạp tệp `.ico` từ ổ đĩa; khởi động tức thì và tiết kiệm tài nguyên tuyệt đối.
2. **Cơ chế phục hồi thông minh phím xóa lùi (Smart Backspace Popping):**
   - Khi bấm phím `Backspace` vật lý, nếu từ hiện tại đang được gõ dở, bộ gõ sẽ pop ký tự/phím gõ thô gần nhất và tự động phân tích lại cấu trúc âm tiết còn lại trên màn hình thay vì xóa sạch trạng thái.
   - *Ví dụ:* Gõ `ddesn` -> hiển thị `đén`. Bấm `Backspace` -> hiển thị `đé`. Gõ tiếp `en` -> tự động biến đổi thành `đến`.
3. **Bộ nhớ lịch sử từ chống lệch trạng thái (Desync-Safe Word History Buffer):**
   - Duy trì ngăn xếp lưu trữ tối đa 8 từ vừa gõ xong. Bấm `Backspace` sau dấu cách sẽ phục hồi lại từ trước vào bộ nhớ để tiếp tục sửa dấu (*Cross-space restoration*).
   - Tự động xóa sạch bộ đệm (hard reset) ngay khi phát hiện bất kỳ hành vi di chuyển con trỏ nào ngoài luồng tuyến tính: nhấp chuột (`WH_MOUSE_LL`), đổi cửa sổ làm việc (`GetForegroundWindow`), phím điều hướng (`Left/Right/Up/Down/Home/End`), ngắt dòng (`Enter/Tab/Esc`), hoặc các phím tắt chọn/xóa văn bản hàng loạt (`Ctrl+A`, `Ctrl+Z`, `Ctrl+V`, `Alt+Tab`).
4. **Gửi phím nguyên tử không khoảng trắng rác (Atomic SendInput Batching):**
   - Với tất cả ô nhập liệu thông thường (trình soạn thảo, IDE, website thi gõ như Monkeytype / 10FastFingers, khung chat AI), lệnh thay thế chữ được gom thành **1 mảng duy nhất** gồm `N` phím `VK_BACK` và các mã Unicode `KEYEVENTF_UNICODE`, gửi đi bằng 1 lệnh gọi `SendInput` nguyên tử.
   - Tuyệt đối **không chèn `VK_SPACE`**, **không dùng `std::thread::sleep`**, loại trừ triệt để tình trạng nhảy từ sớm khi gõ nhanh.
5. **Cách ly thanh địa chỉ trình duyệt bất đồng bộ & Tiền lọc Win32 Class (Lazy UIA + Fast Pre-filtering):**
   - Luồng nền theo dõi `EVENT_OBJECT_FOCUS` trước tiên kiểm tra lớp cửa sổ gốc bằng `GetClassNameW` trên bộ đệm stack `[u16; 64]` (hoàn toàn không tốn heap). Chỉ khi phát hiện đúng lớp trình duyệt (`Chrome_WidgetWin_1` hoặc `MozillaWindowClass`), `IUIAutomation` mới được khởi tạo lười để kiểm tra Omnibox (`OmniboxViewViews`, `urlbar-input`).
   - Mọi cửa sổ ngoài trình duyệt (Notepad, VS Code, Word, Terminal, Game) lập tức đặt `IS_OMNIBOX_FOCUSED = false` mà không chạm vào COM hay UIA, duy trì ứng dụng ở mức chỉ 3 threads nền.
   - Biến cờ `IS_OMNIBOX_FOCUSED: AtomicBool` được kiểm tra ở mức sub-nanosecond trong hook. Mẹo hủy gợi ý inline (`VK_SPACE` + `VK_BACK`) chỉ được áp dụng duy nhất khi người dùng đang thao tác trên Omnibox.
6. **Tối ưu hóa bộ nhớ RAM & Không phân mảnh Heap (Zero-Allocation Hot Path & One-Shot Trim):**
   - Toàn bộ hot path gõ phím và thuật toán bỏ dấu tiếng Việt sử dụng thread-local reusable buffer và mảng cố định trên stack (`build_normal_inputs_into`), triệt tiêu hoàn toàn việc cấp phát heap liên tục khi gõ nhanh.
   - Sau khi khởi động 2.5 giây, một timer duy nhất kích hoạt gọi `SetProcessWorkingSetSizeEx` từ `kernel32.dll` rồi tự hủy vĩnh viễn, thu hồi các trang nhớ khởi động đưa RAM hiển thị về ~2–4 MB mà không cần phụ thuộc `psapi.dll`.
7. **Nhúng siêu dữ liệu PE Version Resource chuẩn Windows (PE Version Info):**
   - Tệp nhúng sẵn siêu dữ liệu định danh trực tiếp vào Portable Executable (`build.rs` kết hợp `winresource`): hiển thị chi tiết tên sản phẩm (`UniRust`), mô tả file, tác giả (`Phạm Ngọc Trí (NPTBuy)`), bản quyền GNU GPL v3.0 và phiên bản đồng bộ `0.1.3` trong Windows File Properties -> Details và Task Manager mà không cần chứng chỉ số tự ký.

---

### 5. Hướng dẫn sử dụng & Phím tắt (Controls & Shortcuts)

- **Bật / Tắt chế độ gõ tiếng Việt:** Nhấn tổ hợp phím **`Ctrl + Shift`** ở bất kỳ đâu trên Windows.
- **Biểu tượng khay hệ thống (System Tray):**
  - Biểu tượng chữ **V** (màu đỏ): Đang bật chế độ tiếng Việt.
  - Biểu tượng chữ **E** (màu xanh): Đang ở chế độ tiếng Anh (bộ gõ tạm dừng xử lý).
- **Menu chuột phải khay hệ thống:**
  - Nhấp chuột phải vào biểu tượng để bật menu ngữ cảnh: bật/tắt nhanh chế độ tiếng Việt hoặc chọn **Exit UniRust** để đóng ứng dụng hoàn toàn.

---

### 6. Hướng dẫn biên dịch & Kiểm thử (Build & Test Instructions)

#### Yêu cầu môi trường:
- Windows 10 / 11 (x64)
- Rust Toolchain 1.75+ (`rustc` & `cargo` với MSVC target `x86_64-pc-windows-msvc`)

#### Chạy kiểm thử tự động (Test Suite):
```powershell
cargo test
```
Toàn bộ **19 bài test** (18 unit tests kiểm tra bảng mã, cấu trúc âm tiết, bộ đệm lịch sử, hoàn tác, tiền lọc trình duyệt, tái sử dụng buffer, xử lý chuẩn phím `z`, độ sạch NFC và 1 integration test kiểm thử E2E luồng gõ) sẽ chạy và báo cáo kết quả:
```text
running 18 tests (src/lib.rs)
test engine::telex::tests::test_circumflex_and_undo ... ok
test engine::telex::tests::test_cross_space_restoration ... ok
test engine::telex::tests::test_case_preservation ... ok
test engine::telex::tests::test_mid_word_repair ... ok
test engine::telex::tests::test_desync_hard_reset ... ok
test engine::telex::tests::test_real_vietnamese_syllables_batch_2 ... ok
test engine::telex::tests::test_real_vietnamese_syllables_batch_3 ... ok
test engine::telex::tests::test_word_history_cap_at_8 ... ok
test engine::telex::tests::test_required_cases_batch_1 ... ok
test hook::tests::test_browser_class_prefilter_constants ... ok
test hook::tests::test_normal_inputs_zero_space_and_atomic ... ok
test hook::tests::test_reusable_input_buffer_zero_alloc ... ok
test hook::tests::test_corpus_replacement_batching_zero_space ... ok
test hook::tests::test_omnibox_cancellation_inputs ... ok
test engine::telex::tests::test_telex_z_specification ... ok
test engine::telex::tests::test_telex_z_backspace ... ok
test engine::telex::tests::test_unikey_synced_edge_cases ... ok
test engine::telex::tests::test_nfc_and_token_cleanliness_corpus ... ok

running 1 test (tests/e2e_typing_test.rs)
test test_e2e_typing_test_harness ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; finished in 0.00s
```

#### Biên dịch bản Release tối ưu:
```powershell
cargo build --release
```
Tệp nhị phân thực thi độc lập sẽ được tạo tại:
```text
target\release\unirust.exe
target\release\unikey-rs.exe
```

---

### 7. Độ ổn định & Giới hạn đã biết (Stability & Known Limitations)

UniRust được thiết kế theo tư duy kỹ thuật tối giản (minimalist, low-overhead), giải quyết triệt để bài toán gõ tiếng Việt tốc độ cao mà không làm nặng hệ thống. Tuy nhiên, người dùng cần lưu ý một số giới hạn kỹ thuật sau:

1. **Phạm vi kiểu gõ & bảng mã:** UniRust hiện tại chỉ tập trung tối ưu hóa cho **kiểu gõ Telex** và **bảng mã chuẩn Quốc tế Unicode dựng sẵn (NFC)**. Ứng dụng chưa hỗ trợ kiểu gõ VNI, VIQR, các bảng mã cũ (TCVN3, VNI Windows) hay bảng từ gõ tắt (Macro).
2. **Ứng dụng chạy quyền Quản trị cao (UAC / UIPI):** Theo cơ chế bảo mật cô lập giao diện của Windows (User Interface Privilege Isolation - UIPI), một tiến trình chạy ở quyền thông thường không được phép hook hay can thiệp phím vào các cửa sổ chạy quyền Administrator (Elevated). Do đó, để gõ tiếng Việt trong các ứng dụng Elevated (như Command Prompt/PowerShell mở bằng *Run as Administrator*, Task Manager, Registry Editor), bạn chỉ cần khởi chạy `unirust.exe` dưới quyền Administrator.
3. **Cơ chế hook mức thấp thay vì Windows TSF:** UniRust hoạt động độc lập bằng Win32 `WH_KEYBOARD_LL` và gửi phím thay thế `VK_BACK` + `KEYEVENTF_UNICODE`, không can thiệp sâu vào Text Services Framework (TSF) dạng COM DLL. Vì vậy, nếu bạn dùng chuột bấm vào giữa một từ đã gõ từ trước bên ngoài ngăn xếp lịch sử từ, bộ gõ sẽ không thể tự động tái cấu trúc từ đó trên màn hình để bỏ dấu lại.
4. **Thanh tìm kiếm tùy biến:** Tính năng cô lập thanh địa chỉ Omnibox dựa vào UI Automation để phát hiện các class chuẩn (`OmniboxViewViews`, `urlbar-input`). Các trình duyệt hoặc ứng dụng Electron sử dụng thanh tìm kiếm vẽ tay hoàn toàn không theo chuẩn Accessibility có thể cần bổ sung thêm nhận diện ID nếu xảy ra hiện tượng gợi ý đè chữ.
5. **Tổ hợp viết tắt không chuẩn từ điển:** Khi gõ các cụm nguyên âm viết tắt không có trong ngữ pháp tiếng Việt thông dụng, bộ quy tắc đặt dấu tự do sẽ ưu tiên đặt dấu vào nguyên âm đầu trừ khi có phụ âm cuối đi sau.

---

### 8. Lộ trình phát triển (Roadmap)

- [ ] Bổ sung tùy chọn kiểu gõ **VNI**.
- [ ] Bổ sung mục **"Khởi động cùng Windows" (Start with Windows)** trực tiếp trên menu chuột phải khay hệ thống.
- [ ] Tính năng định nghĩa bảng gõ tắt (Macro / Shorthand expansion).
- [ ] Mở rộng bảng nhận diện UI Automation cho các ứng dụng Electron và thanh tìm kiếm tùy biến.
- [ ] Nghiên cứu giải pháp module Windows Text Services Framework (TSF) COM cho khả năng tích hợp sâu hơn trong tương lai.

---

### 9. Giấy phép mã nguồn mở (License)

UniRust là phần mềm mã nguồn mở được phát hành theo các điều khoản của **[Giấy phép Công cộng GNU phiên bản 3.0 (GNU General Public License v3.0 - GPL-3.0)](LICENSE)**.

Dự án tuân thủ nghiêm ngặt và trân trọng quyền tác giả đối với mã nguồn gốc của gói **x-unikey-1.0.4** do tác giả **Phạm Kim Long** phát hành ([https://www.unikey.org/source.html](https://www.unikey.org/source.html)).

---

<a name="english"></a>
## English

> [!IMPORTANT]
> **Disclaimer on Educational Purpose, Non-Commercial Stance & Security Transparency:**
>
> 1. **Educational & Experimental Purpose:** UniRust is strictly a personal learning and experimental project created to learn, build, and run modern **Rust**, and to experiment with executing the [REA (Reverse-Engineer-Anything)](https://github.com/teonett/Reverse-Engineer-Anything) workflow using an AI Agent.
> 2. **Non-Commercial & Non-Competitive Stance:** This project has **zero commercial purpose**, is not intended to compete with, replace, or supersede **UniKey** (or any other established Vietnamese IME), and carries no hidden agenda. Everyday users are strongly encouraged to continue using the official [UniKey](https://www.unikey.org) created by Phạm Kim Long for daily productivity and rock-solid stability.
> 3. **Security & Transparency Commitment (Not Malware / Not a Keylogger):** Because UniRust relies on a Windows Low-Level Keyboard Hook (`WH_KEYBOARD_LL`) to intercept and process Telex keystrokes, antivirus heuristics or security-conscious users may naturally question its behavior. We provide the following verifiable technical guarantees:
>    - **100% Offline:** Zero networking dependencies, zero socket/HTTP code, zero telemetry, and no data ever leaves the local machine.
>    - **Zero Keystroke Logging:** Typed characters are never written to disk or any log file. The active syllable buffer and short word history (capped at 8 words) live strictly in volatile RAM and are instantly wiped upon word boundaries (Space/Enter), mouse clicks, window switches, navigation keys, or bulk shortcuts.
>    - **Fully Auditable Open Source (GNU GPL v3.0):** The entire codebase is completely open for public audit. Users are actively encouraged to inspect the Rust source code and compile the executable themselves via `cargo build --release` rather than blindly trusting prebuilt binaries.

---

### 1. Project Overview

**UniRust** is an open-source, lightweight, high-performance Vietnamese Input Method Editor (IME) written purely in **Rust** for Windows x64. The project is engineered for: **minimal binary footprint, blazing-fast input response, zero external DLL or C-runtime dependencies**, and **strict pre-composed NFC Unicode output**.

- **Ultra-Compact Footprint:** Standalone binary size is only **~177 KB** (181,248 bytes in release build), requiring no installer, no .NET Framework, and no Visual C++ Redistributable runtime.
- **Ultra-Lean Memory Footprint (RAM ~2–4 MB):** Lazy UI Automation with fast Win32 class pre-filtering (`GetClassNameW`), zero-heap-allocation reusable hot-path buffers, and a clean one-shot Working Set trim via `kernel32.dll` 2.5s post-startup (zero `psapi.dll` dependency).
- **Zero DLL Injection:** Operates entirely as a clean, in-process background application using standard Win32 Low-Level Hooks (`WH_KEYBOARD_LL`, `WH_MOUSE_LL`).
- **Strict Pre-composed NFC Unicode (Zero AI Token Bloat):** Generates true pre-composed NFC Unicode code points directly. It strictly avoids decomposed combining marks (NFD `U+0300..=U+036F`) and zero-width invisible characters (`U+200B`, `U+2060`, `U+FEFF`), eliminating token bloat and context corruption when prompting AI/LLM tools (ChatGPT, Claude, Gemini, Cursor, Copilot).

---

### 2. Origin, Attribution & Respect for UniKey

UniRust is a pure-Rust re-implementation inspired by, referencing, and honoring the foundational algorithms of the classic **UniKey Vietnamese Input Method** created by **Phạm Kim Long** ([https://www.unikey.org](https://www.unikey.org)).

This project inspected and cross-referenced official reference source code from the **x-unikey-1.0.4** distribution (released under GNU GPL/LGPL v2.0-or-later):
- `ukengine.cpp` & `ukengine.h`: Syllable parser state machine, initial consonant detection, vowel cluster rules, tone transitions, and double-key undo logic.
- `data.cpp`: Character lookup tables, tone diacritic mappings, and the 134 pre-composed NFC Unicode mappings.
- `vnlexi.h`: Lexical syntax, valid vowel clusters, and consonant combinations.

#### Key Rules Ported into Pure Rust:
1. **134 Pre-composed NFC Unicode Mappings:** Complete coverage of all Vietnamese vowels with 5 tone marks (acute, grave, hook, tilde, dot) and diacritics (circumflex `â, ê, ô`, horn `ư, ơ`, breve `ă`).
2. **Initial Consonant Edge Cases (`qu`, `gi`):**
   - In `qu-`: `u` is treated as part of the initial consonant; tone marks are placed onto the subsequent vowel (`quả`, `quý`, `quốc`).
   - In `gi-`: Accurately distinguishes diphthongs (`già`, `giá` tone on `a`) from the standalone word (`gì` tone on `i`).
3. **Compound Vowels & Horn/Breve Rules (`w`):**
   - `uo + w -> ươ` (e.g., *trường, nước, hươu, rượu*).
   - `ua + w -> ưa` (e.g., *sửa, mưa, cửa* — never mistakenly converted to `uă`).
   - Standalone `w` at syllable start produces `ư` (*ừ, ứ*); pressing `ww` cleanly reverts to literal `w`.
   - `u + w -> ư`; pressing `w` again (`uw + w`) cleanly reverts to literal `uw`.
4. **Stop-Consonant Tone Restrictions:**
   - Words ending in voiceless stop consonants (`c`, `ch`, `p`, `t`) strictly accept only Acute (`s`) or Dot (`j`) tones (e.g., *bác, bạc, sách, sạch, tháp, thạp, cát, cạt*).
5. **English Word Preservation:**
   - Standard English words containing `w` (*new, view, crew, draw, show*) are preserved without unwanted horn conversions.
6. **Free-Tone Placement Orthography:**
   - Allows users to type tone keys at any point during syllable input; automatically applies standard Vietnamese orthographic placement based on vowel nucleus and final consonants (*hoà/hòa, thuỷ/thủy, người, tiếng, toán, hoàng*).
7. **Case-Preserving Double-Key Undo:**
   - Double-typing modifier keys (`aa, ee, oo, dd, ww, ss, ff, rr, xx, jj`) cleanly undoes that specific diacritic or tone while preserving case states (*Trường, ĐẮK LẮK, VIỆT NAM*).
8. **UniKey-Compliant `z` Tone Removal & Double-`z` Undo:**
   - The `z` key acts as a tone-removal command only when the current syllable actively carries a tone (`ras` -> `rá`, `+ z` -> `ra`). If no tone exists (`raz`, `zoo`, `quiz`), a literal `z` is emitted.
   - Double-`z` undo: Pressing `z` immediately after tone removal restores the removed tone and appends literal `z` (`raszz` -> `ráz`).
   - Once a literal `z` is emitted, the syllable is treated as non-Vietnamese, protecting subsequent letters in English words (`razer`, `azure`, `size`, `crazy`, `frozen`, `pizza`, `buzz`) from retroactive Telex transformations.

---

### 3. Author & AI-Assisted Workflow

- **Project Owner & Requester:** **Phạm Ngọc Trí (NPTBuy)**
  - GitHub: [https://github.com/NPTBuy](https://github.com/NPTBuy)
  - Email: [tri.phamngoc2006@gmail.com](mailto:tri.phamngoc2006@gmail.com)
  - **Role Definition:** Leads the Rust learning and research process, formulates ideas and architectural specifications, conducts real-world typing and edge-case testing, and reviews/verifies all outputs.
- **AI-Assisted Development Workflow:**
  - **Gemini:** Ideation, UniKey source analysis, root-cause debugging of keystroke race conditions, and syllable parser architectural design.
  - **Antigravity:** Agentic execution of the [REA (Reverse-Engineer-Anything)](https://github.com/teonett/Reverse-Engineer-Anything) workflow for Rust code implementation, Win32 hook refactoring, atomic input batching, automated test harness construction, and release compilation.

---

### 4. Architecture & Key Features

```
┌────────────────────────────────────────────────────────────────────────┐
│                        UniRust Architecture                           │
├────────────────────────────────────────────────────────────────────────┤
│  [Physical Keystroke]                                                  │
│          │                                                             │
│          ▼                                                             │
│  [WH_KEYBOARD_LL Hook] ──(Loopback filter: dwExtraInfo == 0x554E494B)  │
│          │                                                             │
│          ├─► Desync Guards (Mouse click, Focus change, Ctrl/Alt, Nav) │
│          │                                                             │
│          ▼                                                             │
│  [Syllable Engine (telex.rs & unicode_map.rs)]                         │
│    - Initial Consonant + Rhyme + Tone + Keystroke History              │
│    - Smart Backspace Popping & Word History Stack (Cap 8 words)        │
│          │                                                             │
│          ├──────────────────────────┬──────────────────────────┐       │
│          ▼                          ▼                          ▼       │
│    PassThrough                    Reset                    Transform   │
│   (Send to OS)                (Clear State)                    │       │
│                                                                ▼       │
│                                                   [IS_OMNIBOX_FOCUSED] │
│                                                   (AtomicBool / UIA)   │
│                                                    ├─────────┴───────┐ │
│                                                    ▼                 ▼ │
│                                              Normal Inputs        Omnibox
│                                            (Atomic Batch:        (Cancel
│                                            Zero VK_SPACE,       VK_SPACE+
│                                             Zero Sleep)         VK_BACK)
└────────────────────────────────────────────────────────────────────────┘
```

1. **In-Memory GDI System Tray Icon:**
   - The tray icon displays a red **V** (Vietnamese mode) or blue **E** (English mode), dynamically rendered in memory using Win32 GDI onto an HBITMAP and passed to `Shell_NotifyIconW`.
   - Requires zero external `.ico` files on disk; instant startup and zero I/O footprint.
2. **Smart Backspace Popping:**
   - Pressing physical `Backspace` mid-word pops the last character/keystroke and re-evaluates the remaining syllable state on screen rather than resetting the engine.
   - *Example:* Typing `ddesn` produces `đén`. Pressing `Backspace` leaves `đé`. Typing `en` transforms seamlessly to `đến`.
3. **Desync-Safe Word History Buffer:**
   - Maintains an 8-word history stack. Pressing `Backspace` after typing a space pops the committed word back into the active buffer for tone correction (*Cross-space restoration*).
   - Instantly wipes both the buffer and word history (hard reset) upon any non-linear cursor action: mouse clicks (`WH_MOUSE_LL`), window focus changes (`GetForegroundWindow`), navigation keys (`Left/Right/Up/Down/Home/End`), line/focus breaks (`Enter/Tab/Esc`), or bulk text selection/deletion shortcuts (`Ctrl+A`, `Ctrl+Z`, `Ctrl+V`, `Alt+Tab`).
4. **Atomic `SendInput` Batching (No Stray Spaces):**
   - For all standard text inputs (editors, IDEs, typing-test sites like Monkeytype and 10FastFingers, and AI chat boxes), deletions and replacement Unicode characters are dispatched in a single atomic array via `SendInput`.
   - **Zero `VK_SPACE` events** and **zero `std::thread::sleep`** in normal replacements, eliminating premature word submission and keystroke races.
5. **Asynchronous Browser Omnibox Isolation & Fast Win32 Pre-filtering:**
   - The background event monitor first inspects the root window class via `GetClassNameW` on a stack-allocated `[u16; 64]` buffer (zero heap allocation). Only if the window belongs to a known browser class (`Chrome_WidgetWin_1` or `MozillaWindowClass`) is `IUIAutomation` lazily initialized to inspect address bar elements (`OmniboxViewViews`, `urlbar-input`).
   - All non-browser applications (Notepad, VS Code, Word, terminals, games) immediately set `IS_OMNIBOX_FOCUSED = false` without touching COM or UIA, keeping thread count at a minimal 3 background threads.
   - Updates a lock-free `IS_OMNIBOX_FOCUSED: AtomicBool` (<1ns check). The inline autocomplete cancellation trick (`VK_SPACE` + `VK_BACK` with 2ms yield) is applied strictly when an address bar has focus.
6. **Zero-Allocation Hot Path & One-Shot Working Set Trim:**
   - Normal typing transforms and syllable tone adjustments use thread-local reusable buffers and stack-allocated arrays (`build_normal_inputs_into`), eliminating runtime heap allocations during fast typing.
   - Exactly 2.5 seconds after startup, a one-shot hidden-window timer triggers `SetProcessWorkingSetSizeEx` from `kernel32.dll` (no `psapi.dll` dependency) and immediately disarms itself, reclaiming one-time startup initialization pages and dropping resident RAM to ~2–4 MB.
7. **Native Windows PE Version Resource Metadata (PE Version Info):**
   - Embedded PE metadata directly in the executable via `build.rs` and `winresource`: surfaces full product details (`UniRust`), file description, author (`Phạm Ngọc Trí (NPTBuy)`), GNU GPL v3.0 copyright, and synced version `0.1.3` in Windows File Properties -> Details and Task Manager without requiring self-signed digital certificates.

---

### 5. Controls & Shortcuts

- **Toggle Vietnamese / English Mode:** Press **`Ctrl + Shift`** anywhere in Windows.
- **System Tray Icon:**
  - Red **V**: Vietnamese Telex input enabled.
  - Blue **E**: English mode enabled (pass-through).
- **Right-Click Context Menu:**
  - Right-click the tray icon to quickly toggle input modes or select **Exit UniRust** to quit.

---

### 6. Build & Test Instructions

#### Prerequisites:
- Windows 10 / 11 (x64)
- Rust Toolchain 1.75+ (`rustc` & `cargo` targeting `x86_64-pc-windows-msvc`)

#### Running the Automated Test Suite:
```powershell
cargo test
```
All **19 automated tests** (18 unit tests covering character mappings, syllable parsing, word history, undo, browser pre-filtering, zero-allocation buffers, UniKey-compliant `z` handling, and clean NFC compliance, plus 1 comprehensive E2E typing-test integration harness) execute and pass:
```text
running 18 tests (src/lib.rs)
test engine::telex::tests::test_circumflex_and_undo ... ok
test engine::telex::tests::test_cross_space_restoration ... ok
test engine::telex::tests::test_case_preservation ... ok
test engine::telex::tests::test_mid_word_repair ... ok
test engine::telex::tests::test_desync_hard_reset ... ok
test engine::telex::tests::test_real_vietnamese_syllables_batch_2 ... ok
test engine::telex::tests::test_real_vietnamese_syllables_batch_3 ... ok
test engine::telex::tests::test_word_history_cap_at_8 ... ok
test engine::telex::tests::test_required_cases_batch_1 ... ok
test hook::tests::test_browser_class_prefilter_constants ... ok
test hook::tests::test_normal_inputs_zero_space_and_atomic ... ok
test hook::tests::test_reusable_input_buffer_zero_alloc ... ok
test hook::tests::test_corpus_replacement_batching_zero_space ... ok
test hook::tests::test_omnibox_cancellation_inputs ... ok
test engine::telex::tests::test_telex_z_specification ... ok
test engine::telex::tests::test_telex_z_backspace ... ok
test engine::telex::tests::test_unikey_synced_edge_cases ... ok
test engine::telex::tests::test_nfc_and_token_cleanliness_corpus ... ok

running 1 test (tests/e2e_typing_test.rs)
test test_e2e_typing_test_harness ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; finished in 0.00s
```

#### Building the Standalone Release Binary:
```powershell
cargo build --release
```
The optimized binaries are generated at:
```text
target\release\unirust.exe
target\release\unikey-rs.exe
```

---

### 7. Stability & Known Limitations

UniRust is built with a minimalist, low-overhead design philosophy. The following technical boundaries should be noted:

1. **Input Method & Charset Scope:** Focuses strictly on **Telex** and pre-composed **NFC Unicode**. Does not support VNI, VIQR, legacy code pages (TCVN3, VNI Windows), or macro shorthand expansion.
2. **Elevated Applications (UAC / UIPI):** Windows User Interface Privilege Isolation (UIPI) blocks standard-integrity hooks from intercepting keystrokes inside elevated (Administrator) windows. To type Vietnamese in Administrator tools (e.g., Command Prompt or PowerShell run as Administrator, Task Manager, Regedit), launch `unirust.exe` as Administrator.
3. **Non-TSF Architecture (`WH_KEYBOARD_LL`):** Operates via low-level keyboard hooks and synthetic `VK_BACK` + `KEYEVENTF_UNICODE` replacement rather than a Windows Text Services Framework (TSF) COM provider. Clicking into the middle of previously committed text outside the linear Word History Buffer cannot retroactively re-compose that word.
4. **Custom Address Bars:** Omnibox focus detection relies on standard UI Automation class names (`OmniboxViewViews`, `urlbar-input`). Electron applications or custom-drawn browser address bars may require tailored UI Automation rules if inline autocomplete highlights interfere.
5. **Non-Standard Abbreviations:** Out-of-dictionary diphthong abbreviations default free-tone placement to the initial vowel nucleus unless followed by a final consonant.

---

### 8. Roadmap

- [ ] Support for **VNI** input method.
- [ ] Add **"Start with Windows"** toggle to the System Tray context menu.
- [ ] Customizable macro and shorthand text expansion tables.
- [ ] Expanded UI Automation heuristics for custom Electron browser search fields.
- [ ] Long-term exploration of a Windows Text Services Framework (TSF) COM component.

---

### 9. Open-Source License

UniRust is open-source software licensed under the **[GNU General Public License v3.0 (GPL-3.0)](LICENSE)**.

This project strictly adheres to and respects the intellectual property and copyright of the original **x-unikey-1.0.4** source code created by **Phạm Kim Long** ([https://www.unikey.org/source.html](https://www.unikey.org/source.html)).
