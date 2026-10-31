#!/usr/bin/env bash
# Generate SOLIVE escrow commands for job specs, with hashes and immutable spec URLs.
#
# This script NEVER posts anything and never touches a wallet. It prints the exact
# commands to run in the session that has the SOLIVE tools and the wallet.
#
# Why it exists (each point is a way a hand-written posting goes wrong):
#   * spec_hash must be the SHA-256 of the bytes a worker downloads. The script hashes
#     the blob committed on origin/main (`git show`), not the working-tree file, so local
#     edits or Windows CRLF conversion cannot produce a hash that no download matches.
#   * spec_url is pinned to that commit (raw.githubusercontent.com/<owner>/<repo>/<sha>/…),
#     so a later push to main cannot change the spec out from under the posted hash.
#   * Amount, phase and dependencies are read from each spec's header — the spec is the
#     single source of truth, nothing is duplicated here.
#   * A job whose "Depends On" jobs are not yet approved is refused by default, because its
#     spec is written against code those jobs have not delivered yet.
#
# Usage:
#   scripts/solive_post_commands.sh --attestor <WALLET> --approved <LIST> [options] <SELECTOR>...
#
#   --attestor <WALLET>  Base58 wallet that attests deliveries (required).
#   --approved <LIST>    Comma-separated job numbers already approved AND merged to main,
#                        e.g. 1,2,3,4 — or "none" (required, so the choice is explicit).
#   --ref <REF>          Commit to pin specs to (default: origin/main after a fetch).
#                        Must be an ancestor of origin/main.
#   --ignore-deps        Emit commands even for jobs with unapproved dependencies.
#                        Each such job is listed with a warning.
#   SELECTOR             A job number (7), a range (5-7), or a phase (phase2).
#
# Example:
#   scripts/solive_post_commands.sh --attestor 2atXgJgD7AdDiJK6rah7yq4HHJEQQxQCvcKcoxFWTyKm \
#       --approved 1,2,3,4 phase2

set -euo pipefail

die() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

usage() {
    sed -n '/^# Usage:/,/^# Example:/p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
}

LAMPORTS_PER_SOL=1000000000

# "0.06" -> 60000000 (integer lamports; avoids floating-point sums).
sol_to_lamports() {
    local sol="$1" int frac
    [[ "$sol" =~ ^([0-9]+)(\.([0-9]{1,9}))?$ ]] || return 1
    int="${BASH_REMATCH[1]}"
    frac="${BASH_REMATCH[3]}"
    while ((${#frac} < 9)); do frac="${frac}0"; done
    echo $((10#$int * LAMPORTS_PER_SOL + 10#$frac))
}

# 60000000 -> "0.06"
lamports_to_sol() {
    local l="$1" int frac
    int=$((l / LAMPORTS_PER_SOL))
    frac=$(printf '%09d' $((l % LAMPORTS_PER_SOL)))
    frac="${frac%"${frac##*[!0]}"}"
    if [[ -z "$frac" ]]; then echo "$int"; else echo "$int.$frac"; fi
}

contains() {
    local needle="$1" x
    shift
    for x in "$@"; do [[ "$x" == "$needle" ]] && return 0; done
    return 1
}

# ---- arguments ------------------------------------------------------------------------

attestor=""
approved_raw=""
ref=""
ignore_deps=0
selectors=()

while (($#)); do
    case "$1" in
        --attestor) (($# >= 2)) || usage; attestor="$2"; shift 2 ;;
        --approved) (($# >= 2)) || usage; approved_raw="$2"; shift 2 ;;
        --ref) (($# >= 2)) || usage; ref="$2"; shift 2 ;;
        --ignore-deps) ignore_deps=1; shift ;;
        -h | --help) usage ;;
        -*) die "unknown option: $1" ;;
        *) selectors+=("$1"); shift ;;
    esac
done

[[ -n "$attestor" ]] || die "--attestor is required"
[[ "$attestor" =~ ^[1-9A-HJ-NP-Za-km-z]{32,44}$ ]] || die "--attestor is not a base58 Solana address: $attestor"
[[ -n "$approved_raw" ]] || die "--approved is required (use --approved none if no job is approved yet)"
((${#selectors[@]})) || die "no jobs selected (give job numbers, ranges like 5-7, or phases like phase2)"

approved=()
if [[ "$approved_raw" != "none" ]]; then
    IFS=',' read -r -a parts <<<"$approved_raw"
    for p in "${parts[@]}"; do
        [[ "$p" =~ ^[0-9]+$ ]] || die "--approved entry is not a job number: '$p'"
        approved+=($((10#$p)))
    done
fi

# ---- repository -----------------------------------------------------------------------

repo_root=$(git rev-parse --show-toplevel 2>/dev/null) || die "not inside a git repository"
cd "$repo_root"

# Raw config value: `git remote get-url` would apply url.*.insteadOf rewrites.
origin_url=$(git config --get remote.origin.url) || die "no 'origin' remote configured"
if [[ "$origin_url" =~ github\.com[:/]([^/]+)/([^/]+)$ ]]; then
    gh_owner="${BASH_REMATCH[1]}"
    gh_repo="${BASH_REMATCH[2]%.git}"
else
    die "origin is not a GitHub repository: $origin_url"
fi

git fetch --quiet origin main || die "could not fetch origin/main (specs must be pinned to what is on GitHub)"

if [[ -z "$ref" ]]; then
    ref="origin/main"
fi
commit=$(git rev-parse --verify --quiet "${ref}^{commit}") || die "unknown ref: $ref"
git merge-base --is-ancestor "$commit" origin/main \
    || die "$ref ($commit) is not on origin/main — merge the spec changes first"

# ---- parse every spec at the pinned commit ---------------------------------------------

declare -A TITLE BUDGET LAMPORTS PHASE DEPS FILE HASH
all_jobs=()

while IFS= read -r path; do
    [[ "$path" =~ ^SOLIVE_JOB_AUDIT_FIX_([0-9]+)\.md$ ]] || continue
    n=$((10#${BASH_REMATCH[1]}))
    content=$(git show "$commit:$path")

    # Only the header (everything before the first '---' line) carries metadata.
    header=$(printf '%s\n' "$content" | sed '/^---[[:space:]]*$/q')

    title=$(printf '%s\n' "$header" | sed -n 's/^# SOLIVE Job: *//p' | head -n1)
    [[ -n "$title" ]] || die "$path: missing '# SOLIVE Job: <title>' line"

    budget=$(printf '%s\n' "$header" | sed -n 's/^\*\*Budget:\*\* *\([^ ]*\) *SOL.*/\1/p' | head -n1)
    lamports=$(sol_to_lamports "$budget") || die "$path: unparseable '**Budget:** <n> SOL' line (got '$budget')"
    ((lamports > 0)) || die "$path: budget must be greater than zero"

    phase=$(printf '%s\n' "$header" | sed -n 's/^\*\*Status:\*\* *Phase \([0-9][0-9]*\).*/\1/p' | head -n1)

    deps=""
    dep_line=$(printf '%s\n' "$header" | sed -n 's/^\*\*Depends On:\*\* *//p' | head -n1)
    if [[ -n "$dep_line" ]]; then
        dep_spec="${dep_line%%(*}"
        if [[ ! "$dep_spec" =~ ^[[:space:]]*[Nn]one ]]; then
            rest="$dep_spec"
            while [[ "$rest" =~ \#([0-9]+)(-([0-9]+))?(.*)$ ]]; do
                lo=$((10#${BASH_REMATCH[1]}))
                hi=$lo
                [[ -n "${BASH_REMATCH[3]}" ]] && hi=$((10#${BASH_REMATCH[3]}))
                rest="${BASH_REMATCH[4]}"
                ((lo <= hi)) || die "$path: descending dependency range in '$dep_line'"
                for ((d = lo; d <= hi; d++)); do deps+="$d "; done
            done
            [[ -n "$deps" ]] || die "$path: unparseable '**Depends On:**' line: '$dep_line'"
        fi
    fi

    [[ -z "${FILE[$n]:-}" ]] || die "two specs claim job number $n"
    FILE[$n]="$path"
    TITLE[$n]="$title"
    BUDGET[$n]="$budget"
    LAMPORTS[$n]="$lamports"
    PHASE[$n]="$phase"
    DEPS[$n]="${deps% }"
    HASH[$n]=$(git show "$commit:$path" | sha256sum | cut -d' ' -f1)
    all_jobs+=("$n")
done < <(git ls-tree --name-only "$commit")

((${#all_jobs[@]})) || die "no SOLIVE_JOB_AUDIT_FIX_*.md specs found at $commit"

for a in "${approved[@]}"; do
    [[ -n "${FILE[$a]:-}" ]] || die "--approved lists job $a, but no spec exists for it"
done

# ---- resolve selectors -----------------------------------------------------------------

selected=()
add_job() {
    [[ -n "${FILE[$1]:-}" ]] || die "no spec for job $1"
    contains "$1" "${selected[@]}" || selected+=("$1")
}

for s in "${selectors[@]}"; do
    if [[ "$s" =~ ^[0-9]+$ ]]; then
        add_job $((10#$s))
    elif [[ "$s" =~ ^([0-9]+)-([0-9]+)$ ]]; then
        lo=$((10#${BASH_REMATCH[1]}))
        hi=$((10#${BASH_REMATCH[2]}))
        ((lo <= hi)) || die "descending range: $s"
        for ((j = lo; j <= hi; j++)); do add_job "$j"; done
    elif [[ "$s" =~ ^phase([0-9]+)$ ]]; then
        want="${BASH_REMATCH[1]}"
        matched=0
        for j in "${all_jobs[@]}"; do
            if [[ "${PHASE[$j]}" == "$want" ]]; then add_job "$j"; matched=1; fi
        done
        ((matched)) || die "selector $s matched no spec (no '**Status:** Phase $want' header)"
    else
        die "unrecognised selector: $s"
    fi
done

mapfile -t selected < <(printf '%s\n' "${selected[@]}" | sort -n)

# ---- checks ----------------------------------------------------------------------------

blocked=()
for j in "${selected[@]}"; do
    contains "$j" "${approved[@]}" && die "job $j is listed in --approved; it is already done, do not post it again"
    unmet=""
    for d in ${DEPS[$j]}; do
        contains "$d" "${approved[@]}" || unmet+="#$d "
    done
    [[ -n "$unmet" ]] && blocked+=("job $j (${TITLE[$j]}) depends on unapproved ${unmet% }")
done

if ((${#blocked[@]})); then
    if ((ignore_deps)); then
        {
            echo "WARNING: --ignore-deps given. These jobs' specs assume work that is not approved yet:"
            printf '  - %s\n' "${blocked[@]}"
            echo
        } >&2
    else
        {
            echo "error: refusing to generate commands; unmet dependencies:"
            printf '  - %s\n' "${blocked[@]}"
            echo "Post these after their dependencies are approved and merged, or pass --ignore-deps"
            echo "to post anyway (workers will build against code that does not exist yet)."
        } >&2
        exit 1
    fi
fi

# ---- output ----------------------------------------------------------------------------

total=0
for j in "${selected[@]}"; do total=$((total + LAMPORTS[$j])); done

echo "# SOLIVE escrow commands — generated, NOT executed"
echo "# Specs pinned to ${gh_owner}/${gh_repo}@${commit}"
echo "# Jobs: ${#selected[@]}   Total escrow: $(lamports_to_sol "$total") SOL"
echo "#"
printf '# %-4s %-6s %-9s %s\n' "Job" "Phase" "Amount" "Title"
for j in "${selected[@]}"; do
    printf '# %-4s %-6s %-9s %s\n' "$j" "${PHASE[$j]:--}" "${BUDGET[$j]}" "${TITLE[$j]}"
done
echo

for j in "${selected[@]}"; do
    echo "# Job $j: ${TITLE[$j]}"
    echo "solive_wallet_escrow \\"
    echo "  --spec_url https://raw.githubusercontent.com/${gh_owner}/${gh_repo}/${commit}/${FILE[$j]} \\"
    echo "  --spec_hash ${HASH[$j]} \\"
    echo "  --amount ${BUDGET[$j]} \\"
    echo "  --attestor ${attestor}"
    echo
done
