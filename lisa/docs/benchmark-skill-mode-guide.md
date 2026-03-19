# Skill Mode Benchmark Guide

`benchmark-skill-mode.sh` compares SKILL.md mode vs SKILL.toml mode by measuring response time and LLM turn count through the daemon gateway API (`/api/chat`).

## Usage

```bash
# Local benchmark (10 runs, default)
lisa/test/benchmark-skill-mode.sh

# Benchmark on remote target
lisa/test/benchmark-skill-mode.sh --target 192.168.0.10

# Custom run count
lisa/test/benchmark-skill-mode.sh --runs 50

# Both options
lisa/test/benchmark-skill-mode.sh --target 192.168.0.10 --runs 50
```

| Option | Default | Description |
|---|---|---|
| `--target <IP>` | (none = local) | Remote target IP for SSH-based benchmark |
| `--runs <N>` | 10 | Number of measurement runs per skill per mode |

## Prerequisites

- `onboard.sh` completed (binary + skills + config + .env)
- SSH key auth configured (for `--target` usage)
- Mock `luna-send` installed on non-webOS environments (`onboard.sh` handles this automatically via symlink)

## What it measures

| Metric | Method |
|---|---|
| Response time (ms) | `curl -w '%{time_total}'` on gateway request-response cycle |
| LLM turn count | `llm_response` event count from runtime trace JSONL |
| Errors | Content filter / provider exhaustion detection |

### Skills tested

| Skill | Query | Purpose |
|---|---|---|
| weather | Live weather API request for Seoul | Forces external tool call every run |
| tv-control | Alternates volume set to 8 / 10 | Forces tool call by alternating values |

## How it works

```
┌─────────────────────────────────────────────────┐
│  1. Deploy config + enable runtime trace        │
├─────────────────────────────────────────────────┤
│  2. Phase 1: SKILL.md mode                      │
│     - Deploy skills                             │
│     - Remove SKILL.toml files                   │
│     - Start daemon                              │
│     - Warm-up (1 ignored run)                   │
│     - Measure weather × N runs                  │
│     - Measure tv-control × N runs               │
├─────────────────────────────────────────────────┤
│  3. Phase 2: SKILL.toml mode                    │
│     - Redeploy skills (with SKILL.toml)         │
│     - Start daemon                              │
│     - Warm-up (1 ignored run)                   │
│     - Measure weather × N runs                  │
│     - Measure tv-control × N runs               │
├─────────────────────────────────────────────────┤
│  4. Final comparison report                     │
│  5. Restore original config (runtime trace off) │
└─────────────────────────────────────────────────┘
```

## Output

### Per-run detail

Each run prints elapsed time and turn count:

```
  [weather] 10 runs...
    # 1 :  6273 ms  (2 turns)
    # 2 :  7150 ms  (2 turns)
    ...
```

### Final report

Side-by-side comparison table for each skill:

```
  ── weather ──────────────────────────────────────────────────
   Run      SKILL.md    turns    SKILL.toml    turns
  ──────────────────────────────────────────────────────────────
     1     14409 ms  4 turns      6163 ms  2 turns
     2      6273 ms  2 turns      6580 ms  2 turns
     ...
  ──────────────────────────────────────────────────────────────
   avg      8957 ms  2.5 turns      6554 ms  2.0 turns

  ── Comparison ──────────────────────────────────────────────────
  Skill              SKILL.md      turns   SKILL.toml      turns  Diff (ms)
  ────────────────────────────────────────────────────────────────
  weather             8957 ms  2.5 turns      6554 ms  2.0 turns  SKILL.toml faster by 2403ms
  tv-control          6369 ms  2.0 turns      5001 ms  2.0 turns  SKILL.toml faster by 1368ms
```

## Configuration

### Gateway port

The script automatically reads `[gateway] port` from `config.default.toml`. No manual port configuration needed.

### Runtime trace

The script temporarily enables runtime trace in the target config for turn counting, and restores the original config on exit (including on error via trap).

## Local vs Target mode

| | Local | Target (`--target`) |
|---|---|---|
| Daemon runs on | localhost | Remote host via SSH |
| Skills deployed via | `onboard.sh --skills` | `onboard.sh --skills --target <IP>` |
| luna-send | Mock (symlink) | Real webOS binary |
| Gateway access | Direct `curl` | `curl` via SSH |
| mock/ directory | Included in skills | Excluded (target has real luna-send) |

## Troubleshooting

### All turns show 0

Another user's daemon may be occupying the gateway port. Check with:

```bash
curl -sf http://127.0.0.1:42617/health | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['runtime']['pid'])"
ps -p <PID> -o user,cmd
```

Kill the stale daemon and re-run the benchmark.

### Permission denied on /tmp/zeroclaw-*.log

Another user's log file exists. The script uses user-ID-based log filenames (e.g., `/tmp/zeroclaw-bench-sungsik-12345.log`) to avoid collisions, but if it occurs, remove the stale file:

```bash
sudo rm /tmp/zeroclaw-*.log
```
