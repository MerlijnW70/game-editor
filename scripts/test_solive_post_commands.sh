#!/usr/bin/env bash
# Self-test for scripts/solive_post_commands.sh.
#
# Builds a throwaway GitHub-style repo (a local bare "origin" reached through
# url.insteadOf) with fixture specs, then checks every guarantee the generator makes.
# Run from anywhere:  bash scripts/test_solive_post_commands.sh

set -euo pipefail

# Hermetic: the user's global/system git config (signing, push hooks, proxies) must not
# affect a throwaway local repo.
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1

script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/solive_post_commands.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

WALLET=2atXgJgD7AdDiJK6rah7yq4HHJEQQxQCvcKcoxFWTyKm
FAKE_URL=https://github.com/example-owner/example-repo

pass=0
fail=0
ok() { pass=$((pass + 1)); printf 'ok   - %s\n' "$1"; }
bad() { fail=$((fail + 1)); printf 'FAIL - %s\n' "$1"; }

spec() { # spec <n> <budget> <phase-or-empty> <depends-or-empty>
    local f
    f=$(printf 'SOLIVE_JOB_AUDIT_FIX_%02d.md' "$1")
    {
        echo "# SOLIVE Job: Fixture $1"
        echo
        echo "**Budget:** $2 SOL  "
        echo "**Deadline:** 7 days  "
        if [[ -n "$3" ]]; then echo "**Status:** Phase $3 - Fixture  "; else echo "**Status:** Open for claims"; fi
        [[ -n "$4" ]] && echo "**Depends On:** $4"
        echo
        echo "---"
        echo
        echo "**Budget:** 9.99 SOL (body text must be ignored)"
        echo "**Depends On:** Job #3 (body text must be ignored)"
    } >"$f"
}

git init --quiet --bare "$work/origin.git"
git init --quiet -b main "$work/repo"
cd "$work/repo"
git config user.email test@example.com
git config user.name test
git config core.autocrlf false
git remote add origin "$FAKE_URL"
git config "url.file://$work/origin.git.insteadOf" "$FAKE_URL"

spec 1 0.12 "" ""
spec 2 0.13 "" ""
spec 3 0.14 "" ""
spec 5 0.06 2 "Job #3 (spatial grid)"
spec 6 0.06 2 "Job #1 (validation)"
spec 7 0.07 2 "Job #1, #2 (config)"
spec 8 0.08 3 ""
spec 9 0.10 3 "None (independent)"
spec 10 0.12 5 "Jobs #1-3 (foundation)"
mkdir scripts
cp "$script" scripts/
git add -A
git commit --quiet -m fixtures
git push --quiet origin main

run() { # run <args...> ; sets out, err, rc
    set +e
    out=$(bash scripts/solive_post_commands.sh "$@" 2>"$work/err")
    rc=$?
    set -e
    err=$(cat "$work/err")
}

# 1. Happy path: phase 2 with all deps approved.
run --attestor "$WALLET" --approved 1,2,3 phase2
head_sha=$(git rev-parse HEAD)
if ((rc == 0)) && [[ $(grep -c '^solive_wallet_escrow' <<<"$out") == 3 ]]; then
    ok "phase2 emits 3 commands"
else bad "phase2 emits 3 commands (rc=$rc) $err"; fi
grep -q '# Jobs: 3   Total escrow: 0.19 SOL' <<<"$out" && ok "total is summed exactly (0.19)" || bad "total 0.19: $out"
grep -q -- "--amount 0.07" <<<"$out" && ! grep -q "9.99" <<<"$out" \
    && ok "budget read from header only" || bad "budget parsing"
expected_hash=$(git show "HEAD:SOLIVE_JOB_AUDIT_FIX_05.md" | sha256sum | cut -d' ' -f1)
grep -q -- "--spec_hash $expected_hash" <<<"$out" && ok "hash is SHA-256 of committed blob" || bad "hash mismatch"
grep -q "raw.githubusercontent.com/example-owner/example-repo/$head_sha/SOLIVE_JOB_AUDIT_FIX_05.md" <<<"$out" \
    && ok "URL pinned to commit and owner/repo from raw origin URL" || bad "URL pinning: $out"

# 2. Local edits and CRLF in the working tree do not change the hash.
sed -i 's/$/\r/' SOLIVE_JOB_AUDIT_FIX_05.md
echo "local edit" >>SOLIVE_JOB_AUDIT_FIX_05.md
run --attestor "$WALLET" --approved 1,2,3 5
grep -q -- "--spec_hash $expected_hash" <<<"$out" && ok "working-tree edits ignored" || bad "working tree leaked into hash"
git checkout --quiet -- SOLIVE_JOB_AUDIT_FIX_05.md

# 3. Unmet dependencies are refused, and nothing actionable is printed.
run --attestor "$WALLET" --approved 1 phase2
if ((rc == 1)) && [[ -z "$out" ]] && grep -q "job 5 .* #3" <<<"$err" && grep -q "job 7 .* #2" <<<"$err" \
    && ! grep -q "job 6" <<<"$err"; then
    ok "unmet deps refused with every blocked job listed"
else bad "unmet deps (rc=$rc) out=$out err=$err"; fi

# 4. Ranges in Depends On ("Jobs #1-3") expand.
run --attestor "$WALLET" --approved 1,3 10
((rc == 1)) && grep -q "job 10 .* #2" <<<"$err" && ok "dependency range expands" || bad "range deps: $err"

# 5. --ignore-deps emits commands but warns.
run --attestor "$WALLET" --approved none --ignore-deps phase2
((rc == 0)) && grep -q WARNING <<<"$err" && [[ $(grep -c '^solive_wallet_escrow' <<<"$out") == 3 ]] \
    && ok "--ignore-deps warns and emits" || bad "--ignore-deps (rc=$rc)"

# 6. "None" dependencies and missing Depends lines need nothing approved.
run --attestor "$WALLET" --approved none phase3
((rc == 0)) && grep -q '# Jobs: 2   Total escrow: 0.18 SOL' <<<"$out" && ok "None / absent deps" || bad "None deps: $err"

# 7. Re-posting an approved job is refused.
run --attestor "$WALLET" --approved 1,2,3 3
((rc == 1)) && grep -q "already done" <<<"$err" && ok "approved job not re-posted" || bad "re-post guard"

# 8. Bad attestor, missing --approved, unknown job, empty phase.
run --attestor "not-a-wallet!" --approved none 8
((rc == 1)) && ok "invalid attestor rejected" || bad "attestor validation"
run --attestor "$WALLET" 8
((rc == 1)) && grep -q "approved is required" <<<"$err" && ok "--approved required" || bad "--approved required"
run --attestor "$WALLET" --approved none 4
((rc == 1)) && grep -q "no spec for job 4" <<<"$err" && ok "unknown job rejected" || bad "unknown job"
run --attestor "$WALLET" --approved none phase4
((rc == 1)) && grep -q "matched no spec" <<<"$err" && ok "empty phase rejected" || bad "empty phase"

# 9. A ref that is not on origin/main is refused.
git checkout --quiet -b side
echo x >side.txt && git add side.txt && git commit --quiet -m side
run --attestor "$WALLET" --approved none --ref side 8
((rc == 1)) && grep -q "not on origin/main" <<<"$err" && ok "unmerged ref refused" || bad "unmerged ref (rc=$rc)"
git checkout --quiet main

# 10. Malformed budget fails closed.
spec 11 "0.1.2" 4 ""
git add -A && git commit --quiet -m bad && git push --quiet origin main
run --attestor "$WALLET" --approved none 8
((rc == 1)) && grep -q "SOLIVE_JOB_AUDIT_FIX_11.md: unparseable" <<<"$err" && ok "malformed budget fails closed" \
    || bad "malformed budget (rc=$rc) $err"

echo
echo "passed: $pass  failed: $fail"
((fail == 0))
