#!/usr/bin/env bash

# =============================================================================
# icepick.sh
#
# Create a compact repository snapshot for LLM analysis.
#
# Usage:
#   ./icepick.sh
#   ./icepick.sh snapshot.txt
#   ./icepick.sh snapshot.txt /path/to/repo
#
# Options:
#   -o, --output FILE       Output file
#   -r, --root DIR          Repository root
#   -l, --max-lines N       Max lines per normal file
#   -s, --max-size-kb N     Large-file threshold
#   -d, --max-depth N       Tree depth
#   -g, --git               Include Git metadata
#       --hidden            Include hidden files
#       --full              Never truncate text files
#       --no-tree           Do not include directory tree
#       --dry-run           Print files that would be included
#       --include EXT       Force extension inclusion
#       --exclude GLOB      Additional exclusion
#   -q, --quiet             Suppress status messages
#   -h, --help              Show help
#
# =============================================================================

set -u
set -o pipefail

OUTPUT="icepick_snapshot.txt"
ROOT="."

MAX_LINES=300
MAX_SIZE_KB=128
MAX_DEPTH=20

INCLUDE_GIT=0
INCLUDE_HIDDEN=0
FULL_MODE=0
INCLUDE_TREE=1
DRY_RUN=0
QUIET=0

EXTRA_EXTENSIONS=()
EXTRA_EXCLUDES=()

# -----------------------------------------------------------------------------
# Helpers
# -----------------------------------------------------------------------------

log() {
    if (( QUIET == 0 )); then
        printf '%s\n' "$*"
    fi
}

die() {
    printf 'icepick: error: %s\n' "$*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Icepick — repository snapshot generator for LLM analysis

Usage:

    ./icepick.sh [OUTPUT [ROOT]]
    ./icepick.sh [OPTIONS]

Options:

    -o, --output FILE
    -r, --root DIR
    -l, --max-lines N
    -s, --max-size-kb N
    -d, --max-depth N
    -g, --git
        --hidden
        --full
        --no-tree
        --dry-run
        --include EXT
        --exclude GLOB
    -q, --quiet
    -h, --help

Examples:

    ./icepick.sh

    ./icepick.sh snapshot.txt

    ./icepick.sh snapshot.txt ~/physiome

    ./icepick.sh \
        --root ~/physiome \
        --output physiome.txt \
        --git

    ./icepick.sh \
        --include rs \
        --include tex \
        --max-lines 500
EOF
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# -----------------------------------------------------------------------------
# Arguments
# -----------------------------------------------------------------------------

POSITIONAL=()

while (( $# > 0 )); do
    case "$1" in

        -o|--output)
            [[ $# -ge 2 ]] || die "$1 requires an argument"
            OUTPUT="$2"
            shift 2
            ;;

        -r|--root)
            [[ $# -ge 2 ]] || die "$1 requires an argument"
            ROOT="$2"
            shift 2
            ;;

        -l|--max-lines)
            [[ $# -ge 2 ]] || die "$1 requires an argument"
            MAX_LINES="$2"
            shift 2
            ;;

        -s|--max-size-kb)
            [[ $# -ge 2 ]] || die "$1 requires an argument"
            MAX_SIZE_KB="$2"
            shift 2
            ;;

        -d|--max-depth)
            [[ $# -ge 2 ]] || die "$1 requires an argument"
            MAX_DEPTH="$2"
            shift 2
            ;;

        -g|--git)
            INCLUDE_GIT=1
            shift
            ;;

        --hidden)
            INCLUDE_HIDDEN=1
            shift
            ;;

        --full)
            FULL_MODE=1
            shift
            ;;

        --no-tree)
            INCLUDE_TREE=0
            shift
            ;;

        --dry-run)
            DRY_RUN=1
            shift
            ;;

        --include)
            [[ $# -ge 2 ]] || die "$1 requires an extension"
            EXTRA_EXTENSIONS+=("${2#.}")
            shift 2
            ;;

        --exclude)
            [[ $# -ge 2 ]] || die "$1 requires a glob"
            EXTRA_EXCLUDES+=("$2")
            shift 2
            ;;

        -q|--quiet)
            QUIET=1
            shift
            ;;

        -h|--help)
            usage
            exit 0
            ;;

        --)
            shift
            while (( $# > 0 )); do
                POSITIONAL+=("$1")
                shift
            done
            ;;

        -*)
            die "unknown option: $1"
            ;;

        *)
            POSITIONAL+=("$1")
            shift
            ;;
    esac
done

if (( ${#POSITIONAL[@]} >= 1 )); then
    OUTPUT="${POSITIONAL[0]}"
fi

if (( ${#POSITIONAL[@]} >= 2 )); then
    ROOT="${POSITIONAL[1]}"
fi

if (( ${#POSITIONAL[@]} > 2 )); then
    die "too many positional arguments"
fi

[[ "$MAX_LINES" =~ ^[0-9]+$ ]] ||
    die "MAX_LINES must be an integer"

[[ "$MAX_SIZE_KB" =~ ^[0-9]+$ ]] ||
    die "MAX_SIZE_KB must be an integer"

[[ "$MAX_DEPTH" =~ ^[0-9]+$ ]] ||
    die "MAX_DEPTH must be an integer"

[[ -d "$ROOT" ]] ||
    die "directory does not exist: $ROOT"

ROOT="$(cd "$ROOT" && pwd -P)"

# -----------------------------------------------------------------------------
# Resolve output path
# -----------------------------------------------------------------------------

if [[ "$OUTPUT" = /* ]]; then
    OUTPUT_ABS="$OUTPUT"
else
    OUTPUT_ABS="$(pwd -P)/$OUTPUT"
fi

mkdir -p "$(dirname "$OUTPUT_ABS")"

OUTPUT_DIR="$(cd "$(dirname "$OUTPUT_ABS")" && pwd -P)"
OUTPUT_ABS="$OUTPUT_DIR/$(basename "$OUTPUT_ABS")"

# -----------------------------------------------------------------------------
# Temporary file list
# -----------------------------------------------------------------------------

TMP_FILE="$(mktemp)"

cleanup() {
    rm -f "$TMP_FILE"
}

trap cleanup EXIT INT TERM

# -----------------------------------------------------------------------------
# File classification
# -----------------------------------------------------------------------------

is_self() {
    local file="$1"
    local candidate

    candidate="$(cd "$(dirname "$file")" 2>/dev/null && pwd -P)/$(basename "$file")"

    [[ "$candidate" == "$OUTPUT_ABS" ]]
}

is_excluded_directory_path() {
    local file="$1"

    case "$file" in
        */.git/*)          return 0 ;;
        */.svn/*)          return 0 ;;
        */.hg/*)           return 0 ;;
        */node_modules/*)  return 0 ;;
        */target/*)        return 0 ;;
        */dist/*)          return 0 ;;
        */build/*)         return 0 ;;
        */coverage/*)      return 0 ;;
        */.next/*)         return 0 ;;
        */.nuxt/*)         return 0 ;;
        */.venv/*)         return 0 ;;
        */venv/*)          return 0 ;;
        */__pycache__/*)   return 0 ;;
        */.cache/*)        return 0 ;;
        */.pytest_cache/*) return 0 ;;
        */.mypy_cache/*)   return 0 ;;
        */.ruff_cache/*)   return 0 ;;
    esac

    return 1
}

is_hidden() {
    local relative

    relative="${1#"$ROOT"/}"

    [[ "$relative" == .* || "$relative" == */.* ]]
}

is_user_excluded() {
    local relative pattern

    relative="${1#"$ROOT"/}"

    for pattern in "${EXTRA_EXCLUDES[@]}"; do
        if [[ "$relative" == $pattern ]]; then
            return 0
        fi
    done

    return 1
}

is_forced_extension() {
    local file="$1"
    local ext

    for ext in "${EXTRA_EXTENSIONS[@]}"; do
        if [[ "$file" == *."$ext" ]]; then
            return 0
        fi
    done

    return 1
}

is_binary_extension() {
    case "$1" in
        *.pdf|\
        *.mp3|*.wav|*.ogg|*.flac|*.aac|*.m4a|\
        *.mp4|*.webm|*.mov|*.avi|*.mkv|\
        *.png|*.jpg|*.jpeg|*.gif|*.bmp|*.webp|*.ico|\
        *.zip|*.tar|*.gz|*.bz2|*.xz|*.7z|*.rar|\
        *.so|*.dll|*.dylib|*.exe|*.bin|*.o|*.a|\
        *.pyc|*.pyo|*.class)
            return 0
            ;;
    esac

    return 1
}

is_generated_file() {
    case "$1" in
        *.aux|\
        *.log|\
        *.out|\
        *.toc|\
        *.synctex.gz|\
        *.fdb_latexmk|\
        *.fls|\
        *.bbl|\
        *.bcf|\
        *.blg)
            return 0
            ;;
    esac

    return 1
}

is_text() {
    local file="$1"

    if is_forced_extension "$file"; then
        return 0
    fi

    if [[ ! -s "$file" ]]; then
        return 0
    fi

    # grep -I is deliberately used here instead of depending entirely
    # on MIME classifications from `file`.
    LC_ALL=C grep -Iq . "$file" 2>/dev/null
}

include_full() {
    local file="$1"
    local base

    base="$(basename "$file")"

    case "$base" in
        README|README.*|\
        LICENSE|LICENSE.*|\
        COPYING|COPYING.*|\
        Makefile|\
        Dockerfile|\
        Containerfile|\
        Cargo.toml|\
        Cargo.lock|\
        package.json|\
        pyproject.toml|\
        setup.py|\
        setup.cfg)
            return 0
            ;;
    esac

    case "$file" in
        *.md|*.markdown|*.toml|*.yaml|*.yml|*.json)
            return 0
            ;;
    esac

    return 1
}

# -----------------------------------------------------------------------------
# Build list
# -----------------------------------------------------------------------------

log "Scanning: $ROOT"

: > "$TMP_FILE"

while IFS= read -r -d '' file; do

    if is_self "$file"; then
        continue
    fi

    if is_excluded_directory_path "$file"; then
        continue
    fi

    if (( INCLUDE_HIDDEN == 0 )) && is_hidden "$file"; then
        continue
    fi

    if is_user_excluded "$file"; then
        continue
    fi

    if ! is_forced_extension "$file"; then

        if is_binary_extension "$file"; then
            continue
        fi

        if is_generated_file "$file"; then
            continue
        fi
    fi

    if ! is_text "$file"; then
        continue
    fi

    printf '%s\0' "$file" >> "$TMP_FILE"

done < <(
    find "$ROOT" -type f -print0
)

# -----------------------------------------------------------------------------
# Count files
# -----------------------------------------------------------------------------

FILE_COUNT=0

while IFS= read -r -d '' file; do
    FILE_COUNT=$((FILE_COUNT + 1))
done < "$TMP_FILE"

log "Eligible files: $FILE_COUNT"

if (( FILE_COUNT == 0 )); then
    printf '\nWARNING: no eligible text files were found.\n' >&2
    printf 'Try: ./icepick.sh --hidden --dry-run\n' >&2
fi

# -----------------------------------------------------------------------------
# Dry run
# -----------------------------------------------------------------------------

if (( DRY_RUN == 1 )); then

    printf '\nFiles that would be included:\n\n'

    while IFS= read -r -d '' file; do
        printf '%s\n' "${file#"$ROOT"/}"
    done < "$TMP_FILE"

    exit 0
fi

# -----------------------------------------------------------------------------
# Metadata
# -----------------------------------------------------------------------------

write_header() {
    echo "===== ICEPICK SNAPSHOT ====="
    echo
    echo "Generated: $(date)"
    echo "Root: $ROOT"
    echo "Files selected: $FILE_COUNT"
    echo

    if command_exists git &&
       git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then

        echo "Git repository: yes"
        echo "Branch: $(git -C "$ROOT" branch --show-current 2>/dev/null)"
        echo "Commit: $(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null)"
    else
        echo "Git repository: no"
    fi

    echo
}

write_git() {
    if (( INCLUDE_GIT == 0 )); then
        return
    fi

    echo "===== GIT METADATA ====="
    echo

    if ! command_exists git; then
        echo "[git unavailable]"
        echo
        return
    fi

    if ! git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "[not a Git repository]"
        echo
        return
    fi

    echo "--- Status ---"
    git -C "$ROOT" status --short || true
    echo

    echo "--- Recent commits ---"
    git -C "$ROOT" log \
        -10 \
        --date=short \
        --pretty=format:'%h %ad %s' || true

    echo
    echo
}

write_tree() {
    if (( INCLUDE_TREE == 0 )); then
        return
    fi

    echo "===== DIRECTORY TREE ====="
    echo

    if command_exists tree; then

        tree \
            -a \
            -L "$MAX_DEPTH" \
            -I '.git|.svn|.hg|node_modules|target|dist|build|coverage|.next|.nuxt|.venv|venv|__pycache__|.cache' \
            "$ROOT" || true

    else

        echo "[tree unavailable; using find]"
        echo

        find "$ROOT" \
            -maxdepth "$MAX_DEPTH" \
            ! -path '*/.git/*' \
            ! -path '*/node_modules/*' \
            ! -path '*/target/*' \
            ! -path '*/build/*' \
            ! -path '*/dist/*' \
            | sort
    fi

    echo
}

# -----------------------------------------------------------------------------
# Write individual file
# -----------------------------------------------------------------------------

write_file() {
    local file="$1"
    local relative
    local size_bytes
    local size_kb
    local lines

    relative="${file#"$ROOT"/}"

    size_bytes="$(wc -c < "$file" | tr -d '[:space:]')"
    size_kb=$(( (size_bytes + 1023) / 1024 ))

    lines="$(wc -l < "$file" | tr -d '[:space:]')"

    echo
    echo "----- FILE: $relative -----"
    echo "[size: ${size_bytes} bytes | lines: ${lines}]"
    echo

    if (( FULL_MODE == 1 )); then

        cat "$file"

    elif include_full "$file"; then

        cat "$file"

    elif (( size_kb > MAX_SIZE_KB )); then

        echo "[[ LARGE FILE: ${size_kb} KB ]]"
        echo "[[ FIRST ${MAX_LINES} LINES ]]"
        echo

        head -n "$MAX_LINES" "$file"

    elif (( lines > MAX_LINES )); then

        echo "[[ TRUNCATED: ${lines} LINES ]]"
        echo "[[ FIRST ${MAX_LINES} LINES ]]"
        echo

        head -n "$MAX_LINES" "$file"

    else

        cat "$file"

    fi

    echo
    echo "----- END FILE: $relative -----"
}

# -----------------------------------------------------------------------------
# Generate
# -----------------------------------------------------------------------------

log "Writing: $OUTPUT_ABS"

{
    write_header
    write_git
    write_tree

    echo "===== FILE CONTENTS ====="

    while IFS= read -r -d '' file; do
        write_file "$file"
    done < "$TMP_FILE"

    echo
    echo "===== SNAPSHOT STATISTICS ====="
    echo
    echo "Files included: $FILE_COUNT"
    echo
    echo "===== END SNAPSHOT ====="

} > "$OUTPUT_ABS"

# -----------------------------------------------------------------------------
# Done
# -----------------------------------------------------------------------------

SNAPSHOT_SIZE="$(du -h "$OUTPUT_ABS" | cut -f1)"

log
log "Snapshot complete."
log "Files: $FILE_COUNT"
log "Size:  $SNAPSHOT_SIZE"
log "Path:  $OUTPUT_ABS"