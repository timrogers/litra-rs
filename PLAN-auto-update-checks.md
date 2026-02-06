# Plan: Auto-Update Checks Using GitHub Releases

## Summary

Implement (and improve) automatic update checks in the `litra` CLI that notify users
when a newer version is available, by querying the GitHub Releases API. The current
codebase already contains a working implementation (see `src/main.rs:1249-1483`). This
plan documents the existing architecture, evaluates its design, and proposes concrete
improvements.

---

## 1. Current Implementation (What Exists Today)

### 1.1 Architecture Overview

```
main()
  |
  +-- check_for_updates()                        # Entry point (main.rs:1365)
  |     |
  |     +-- env var check (LITRA_DISABLE_UPDATE_CHECK)
  |     +-- read_config() -> Config               # Read ~/.litra.toml
  |     +-- should_check_for_updates(&Config)     # Rate limit: once/day
  |     +-- write_config(&Config)                 # Persist last-check timestamp
  |     +-- HTTP GET GitHub Releases API (ureq)   # 2s global timeout
  |     +-- filter releases by age (>=72 hours)
  |     +-- find highest version newer than current
  |     +-- return Option<String>                 # tag name, e.g. "v3.4.0"
  |
  +-- format_update_message(&str) -> String       # main.rs:1474
  |     |
  |     +-- prints colored message to stderr
  |
  +-- Cli::parse() ...                            # Normal CLI execution
```

### 1.2 Key Constants

| Constant                     | Value                                                         | Location       |
| ---------------------------- | ------------------------------------------------------------- | -------------- |
| `CURRENT_VERSION`            | `env!("CARGO_PKG_VERSION")` (compile-time from Cargo.toml)   | main.rs:1249   |
| `GITHUB_API_URL`             | `https://api.github.com/repos/timrogers/litra-rs/releases`   | main.rs:1253   |
| `UPDATE_CHECK_TIMEOUT_SECS`  | 2                                                             | main.rs:1256   |
| `DISABLE_UPDATE_CHECK_ENV`   | `LITRA_DISABLE_UPDATE_CHECK`                                  | main.rs:1355   |
| `CONFIG_FILE_NAME`           | `.litra.toml`                                                 | main.rs:1266   |
| `SECONDS_PER_DAY`            | 86400                                                         | main.rs:1269   |

### 1.3 Data Structures

```rust
struct GitHubRelease {
    tag_name: String,       // e.g. "v3.3.0"
    published_at: String,   // ISO 8601 timestamp
}

struct Config {
    update_check: UpdateCheckConfig,
}

struct UpdateCheckConfig {
    last_check_timestamp: Option<u64>,   // Unix seconds
}
```

### 1.4 Design Decisions Already Made

1. **Rate limiting**: At most one check per 24 hours, tracked in `~/.litra.toml`.
2. **Release age filter**: Only releases >=72 hours old are considered, to avoid
   promoting releases before they're validated by early adopters.
3. **Graceful degradation**: All network errors (including timeouts) are silently
   swallowed; the CLI always continues normally.
4. **2-second global timeout**: Keeps the CLI responsive even on slow networks.
5. **Environment variable opt-out**: `LITRA_DISABLE_UPDATE_CHECK` disables checks
   entirely (useful for CI, scripts, non-interactive use).
6. **stderr output**: Update messages go to stderr so they don't corrupt piped stdout.
7. **Synchronous execution**: The check runs before CLI argument parsing and blocks
   the main thread (up to 2 seconds on first call each day).

### 1.5 Test Coverage (main.rs:1665-1910)

Well-tested functions:
- `is_newer_version()` — major/minor/patch comparisons, edge cases
- `should_check_for_updates()` — rate limiting logic
- `is_release_old_enough()` — timestamp filtering
- `format_update_message()` — output formatting

Not tested (network-dependent):
- `check_for_updates()` — full integration flow
- `read_config()` / `write_config()` — filesystem operations

---

## 2. Proposed Improvements

### 2.1 Non-Blocking Update Check (Priority: High)

**Problem**: The current synchronous check can add up to 2 seconds of latency to every
CLI invocation (once per day). For a hardware control tool where responsiveness matters,
this is noticeable.

**Proposal**: Run the update check in a background thread and print the notification
after the main command completes.

```rust
fn main() -> ExitCode {
    // Spawn the update check on a background thread
    let update_handle = std::thread::spawn(check_for_updates);

    let args = Cli::parse();
    let result = match &args.command { /* ... */ };

    // After command execution, check if the update thread finished
    if let Ok(Some(latest_version)) = update_handle.join() {
        eprintln!("{}", format_update_message(&latest_version));
    }

    // ... exit code handling
}
```

**Trade-offs**:
- Pro: Zero perceived latency for the user.
- Pro: The thread has the full duration of the CLI command to complete.
- Con: If the CLI command is very fast (e.g. `litra devices` with no devices), the
  thread may still be running. We can use `join()` with a short additional timeout or
  simply skip the notification if it isn't ready.

**Files to change**: `src/main.rs` (main function only)

### 2.2 Pre-release / Draft Release Filtering (Priority: Medium)

**Problem**: The current implementation doesn't filter out pre-release or draft releases.
GitHub Releases API can return these, and they shouldn't be recommended to regular users.

**Proposal**: Add `prerelease` and `draft` fields to `GitHubRelease` and filter them out.

```rust
#[derive(serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
    published_at: String,
    prerelease: bool,
    draft: bool,
}
```

Then add to the filter loop:
```rust
if release.prerelease || release.draft {
    continue;
}
```

**Files to change**: `src/main.rs` (struct + filter loop)

### 2.3 Paginated API Response Handling (Priority: Low)

**Problem**: The GitHub Releases API returns at most 30 releases per page by default.
For a project with many releases, the current version might not appear in the first
page. However, since we only care about *newer* releases and the API returns newest
first, this is unlikely to be a practical issue unless there are 30+ releases newer
than the user's version.

**Proposal**: Add `?per_page=10` to the API URL to reduce payload size. Since we only
need the latest release that's older than 72 hours, fetching 10 is more than enough.

```rust
const GITHUB_API_URL: &str =
    "https://api.github.com/repos/timrogers/litra-rs/releases?per_page=10";
```

**Files to change**: `src/main.rs` (constant only)

### 2.4 Testability Improvements (Priority: Medium)

**Problem**: `check_for_updates()` is tightly coupled to `ureq`, filesystem I/O, system
clock, and environment variables, making it impossible to unit test without network access.

**Proposal**: Extract the core logic into pure functions that can be tested independently:

```rust
/// Pure function: given releases, current version, and current time, find the best update.
fn find_best_update(
    releases: &[GitHubRelease],
    current_version: &str,
    now: DateTime<Utc>,
) -> Option<String> { /* ... */ }
```

The existing `check_for_updates()` becomes a thin orchestrator that calls `find_best_update`
after fetching data. The pure function gets comprehensive unit tests.

**Files to change**: `src/main.rs` (refactor + new tests)

### 2.5 Cache the Latest Known Version (Priority: Low)

**Problem**: If the network is unavailable, the user never sees the update notification
even if a previous check found a newer version.

**Proposal**: Store the latest known version in `~/.litra.toml` alongside the timestamp.
On subsequent runs, if the rate limit prevents a new check, still show the notification
if the cached version is newer than the current version.

```rust
struct UpdateCheckConfig {
    last_check_timestamp: Option<u64>,
    latest_known_version: Option<String>,  // NEW
}
```

**Files to change**: `src/main.rs` (config struct, read/write, check logic)

### 2.6 Respect GitHub API Rate Limits (Priority: Low)

**Problem**: Unauthenticated GitHub API requests are limited to 60/hour per IP. The
once-per-day rate limit makes this unlikely to be hit by a single user, but shared
IPs (corporate NATs) could theoretically exhaust the limit.

**Proposal**: Check for HTTP 403/429 responses and handle them gracefully (already
effectively handled by the catch-all error swallowing, but an explicit check would
allow logging a more useful warning).

**Files to change**: `src/main.rs` (error handling in `check_for_updates`)

---

## 3. Implementation Plan (Ordered Steps)

If starting from scratch, here is the recommended implementation order. Steps 1-5 are
the core feature; steps 6-10 are the improvements proposed above.

### Phase 1: Core Feature (Already Complete)

| Step | Task                                                  | Status    |
| ---- | ----------------------------------------------------- | --------- |
| 1    | Add `ureq`, `toml`, `dirs`, `chrono`, `colored` deps | Done      |
| 2    | Define `GitHubRelease`, `Config` structs              | Done      |
| 3    | Implement config read/write (`~/.litra.toml`)         | Done      |
| 4    | Implement rate-limited `check_for_updates()`          | Done      |
| 5    | Wire into `main()`, print to stderr                   | Done      |

### Phase 2: Improvements (Proposed)

| Step | Task                                          | Priority | Complexity | Files           |
| ---- | --------------------------------------------- | -------- | ---------- | --------------- |
| 6    | Make update check non-blocking (background thread) | High | Low        | `src/main.rs`   |
| 7    | Filter out pre-release / draft releases       | Medium   | Low        | `src/main.rs`   |
| 8    | Extract pure `find_best_update()` for testing | Medium   | Medium     | `src/main.rs`   |
| 9    | Cache latest known version in config          | Low      | Low        | `src/main.rs`   |
| 10   | Reduce API payload with `?per_page=10`        | Low      | Trivial    | `src/main.rs`   |

---

## 4. Testing Strategy

### Unit Tests (Existing)

- `test_is_newer_version_*` — semver comparison
- `test_should_check_for_updates_*` — rate limiting
- `test_is_release_old_enough` — timestamp filtering
- `test_format_update_message` — output format

### Unit Tests (To Add)

- `test_find_best_update_*` — pure logic with mock release data
- `test_prerelease_filtering` — pre-release/draft exclusion
- `test_cached_version_notification` — offline notification from cache

### Integration Tests (Manual)

- Verify update message appears on first run of the day
- Verify no message on second run within 24 hours
- Verify `LITRA_DISABLE_UPDATE_CHECK=1` suppresses the check
- Verify timeout handling by pointing to a non-routable IP
- Verify the CLI completes normally when GitHub is unreachable

---

## 5. Risk Assessment

| Risk                                     | Likelihood | Impact | Mitigation                         |
| ---------------------------------------- | ---------- | ------ | ---------------------------------- |
| GitHub API downtime blocks CLI           | Low        | None   | Errors are silently swallowed      |
| Slow network adds latency               | Medium     | Low    | 2s timeout; propose background thread |
| Rate limit exhaustion (shared IP)        | Very Low   | None   | Once-per-day check; graceful fallback |
| Config file corruption                   | Very Low   | Low    | `unwrap_or_default()` on parse     |
| Pre-release recommended to users         | Low        | Low    | Propose filtering (step 7)         |

---

## 6. Dependencies

All dependencies for the core feature are already in `Cargo.toml` under the `cli` feature:

| Crate    | Version | Purpose                        |
| -------- | ------- | ------------------------------ |
| `ureq`   | 3.0.11  | HTTP client for GitHub API     |
| `toml`   | 0.8.20  | Config file serialization      |
| `dirs`   | 6.0.0   | Home directory discovery       |
| `chrono` | 0.4.43  | Release age calculation        |
| `colored`| 2.2     | Terminal output formatting     |
| `serde`  | 1.0.219 | Deserialization of API/config  |

No new dependencies are needed for any of the proposed improvements.
