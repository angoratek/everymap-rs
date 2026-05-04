#!/usr/bin/env bash
# ==============================================================================
# validate-cli.sh — EveryMap CLI structural validation (no real API keys needed)
#
# Tests arg parsing, validation, edge cases, provider×domain matrix, and error
# paths across all 11 commands and 5 providers. Uses --api-key test (dummy key)
# — tests structural behavior, NOT live API calls.
#
# Usage:
#   ./scripts/validate-cli.sh              # full run
#   ./scripts/validate-cli.sh --quiet      # only show failures
#   ./scripts/validate-cli.sh --json       # output results as JSON
#   ./scripts/validate-cli.sh --no-build   # skip cargo build step
#   ./scripts/validate-cli.sh --help       # show this help
#
# Output:
#   validate-results/<timestamp>/
#     ├── SUMMARY.txt
#     ├── results.json
#     └── details/ (per-test .out, .err, .code files)
# ==============================================================================

set -uo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
RESULTS_DIR="${PROJECT_DIR}/validate-results/${TIMESTAMP}"
DETAILS_DIR="${RESULTS_DIR}/details"
CLI_BIN="${PROJECT_DIR}/target/debug/everymap"

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'
YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'

QUIET=false; JSON_OUT=false; NO_BUILD=false; START_TIME; START_TIME=$(date +%s)

while [[ $# -gt 0 ]]; do
  case "$1" in
    --quiet)    QUIET=true; shift ;;
    --json)     JSON_OUT=true; shift ;;
    --no-build) NO_BUILD=true; shift ;;
    -h|--help)
      sed -n '2,23p' "$0"; exit 0 ;;
    *) echo "Unknown arg: $1"; exit 1 ;;
  esac
done

# Unset env vars that would override our --api-key flag
unset EVERYMAP_API_KEY EVERYMAP_HERE_API_KEY EVERYMAP_GOOGLE_API_KEY
unset EVERYMAP_TOMTOM_API_KEY EVERYMAP_MAPBOX_API_KEY EVERYMAP_RADAR_API_KEY

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

if ! $NO_BUILD; then
  printf "${CYAN}Building CLI binary...${NC}\n"
  cargo build -q -p everymap-cli 2>&1 || { echo "Build failed"; exit 1; }
  printf "  Binary: %s\n\n" "$CLI_BIN"
fi

EVERYMAP="cargo run -q -p everymap-cli --"
EVERYMAP_BIN="$CLI_BIN"
mkdir -p "$DETAILS_DIR"

# ---------------------------------------------------------------------------
# Test framework
# ---------------------------------------------------------------------------

# Counters stored in files so subshell assertions can increment them atomically
PASS_FILE="${RESULTS_DIR}/.pass";  echo 0 > "$PASS_FILE"
FAIL_FILE="${RESULTS_DIR}/.fail";  echo 0 > "$FAIL_FILE"
TOTAL_FILE="${RESULTS_DIR}/.total"; echo 0 > "$TOTAL_FILE"

inc_pass() { local v; v=$(cat "$PASS_FILE");  echo $((v+1)) > "$PASS_FILE"; }
inc_fail() { local v; v=$(cat "$FAIL_FILE");  echo $((v+1)) > "$FAIL_FILE"; }
inc_total() { local v; v=$(cat "$TOTAL_FILE"); echo $((v+1)) > "$TOTAL_FILE"; }

elapsed_ms() {
  local now; now=$(perl -MTime::HiRes=time -e 'print int(time*1000)' 2>/dev/null || \
    python3 -c 'import time; print(int(time.time()*1000))' 2>/dev/null || echo 0)
  echo $((now - TEST_START_MS))
}

contains() { echo "$1" | grep -q -F -- "$2"; }

pass() {
  inc_pass; inc_total
  local ms; ms=$(elapsed_ms)
  local info=""; [[ -n "${1:-}" ]] && info="  $1"
  ! $QUIET && printf "  ${GREEN}OK${NC}  [%6dms] %s%s\n" "$ms" "${TEST_DESC:-}" "$info"
}

fail() {
  inc_fail; inc_total
  local ms; ms=$(elapsed_ms)
  printf "  ${RED}FAIL${NC} [%6dms] %s\n" "$ms" "${TEST_DESC:-}"
  printf "         expected: '%s'\n" "${1:-}"
  printf "         got:      '%s'\n" "${2:-}"
}

_save_details() {
  local safe_name="$1"; shift
  local out="$1"; shift
  local exit_code="$1"; shift
  echo "$out"       > "${DETAILS_DIR}/${safe_name}.out"
  echo "$exit_code" > "${DETAILS_DIR}/${safe_name}.code"
}

# assert_out: command succeeds, stdout+stderr contain expected text
assert_out() {
  TEST_DESC="$1"; local expected="$2"; shift 2
  local out rc safe_name
  safe_name=$(echo "$TEST_DESC" | tr ' /()' '____' | tr -dc 'a-zA-Z0-9_-')
  out=$($EVERYMAP "$@" 2>&1); rc=$?
  _save_details "$safe_name" "$out" "$rc"
  if [[ "$rc" -eq 0 ]]; then
    if contains "$out" "$expected"; then
      pass
    else
      fail "$expected" "$(echo "$out" | tr '\n' ' ' | cut -c1-200)"
    fi
  else
    fail "$expected" "exit=${rc}: $(echo "$out" | tr '\n' ' ' | cut -c1-200)"
  fi
}

# assert_err: command may succeed or fail, but output must contain expected text
assert_err() {
  TEST_DESC="$1"; local expected="$2"; shift 2
  local out rc safe_name
  safe_name=$(echo "$TEST_DESC" | tr ' /()' '____' | tr -dc 'a-zA-Z0-9_-')
  out=$($EVERYMAP "$@" 2>&1); rc=$?
  _save_details "$safe_name" "$out" "$rc"
  if contains "$out" "$expected"; then
    pass "exit=${rc}"
  else
    fail "$expected" "$(echo "$out" | tr '\n' ' ' | cut -c1-200)"
  fi
}

# assert_fail: command MUST fail AND output must contain expected text
assert_fail() {
  TEST_DESC="$1"; local expected="$2"; shift 2
  local out rc safe_name
  safe_name=$(echo "$TEST_DESC" | tr ' /()' '____' | tr -dc 'a-zA-Z0-9_-')
  out=$($EVERYMAP "$@" 2>&1); rc=$?
  _save_details "$safe_name" "$out" "$rc"
  if [[ "$rc" -ne 0 ]]; then
    if contains "$out" "$expected"; then
      pass
    else
      fail "$expected" "$(echo "$out" | tr '\n' ' ' | cut -c1-200)"
    fi
  else
    fail "expected non-zero exit but got 0; msg: $expected" "$(echo "$out" | tr '\n' ' ' | cut -c1-200)"
  fi
}

section() {
  echo ""
  printf "${YELLOW}─── %s ───${NC}\n" "$1"
}

API="--api-key test"
TEST_START_MS=0

# ---------------------------------------------------------------------------
# S1: Global flags
# ---------------------------------------------------------------------------
section "S1: Global flags"

assert_out "help shows --provider"    "--provider"    --help
assert_out "help shows --api-key"    "--api-key"     --help
assert_out "help shows --output"     "--output"      --help
assert_out "help shows --verbose"    "--verbose"     --help
assert_out "help shows --version"    "--version"     --help
assert_out "version flag"            "everymap"      --version

assert_fail "unsupported provider 'invalid'"    "Unsupported provider"    --provider invalid $API geocode Berlin
assert_fail "unsupported provider 'xyz'"        "Unsupported provider"    --provider xyz $API geocode Berlin
assert_fail "unsupported provider empty"        "Unsupported provider"    --provider "" $API geocode Berlin
assert_out  "valid provider 'here'"             ""      --provider here   $API geocode Berlin
assert_out  "valid provider 'google'"           ""      --provider google $API geocode Berlin
assert_out  "valid provider 'tomtom'"           ""      --provider tomtom $API geocode Berlin
assert_out  "valid provider 'mapbox'"           ""      --provider mapbox $API geocode Berlin
assert_out  "valid provider 'radar'"            ""      --provider radar  $API geocode Berlin

# Missing API key
out=$(HOME=/tmp/everymap-test-no-config $EVERYMAP_BIN --provider here geocode Berlin 2>&1) && rc=0 || rc=$?
TEST_DESC="missing api key"
if [ "$rc" -ne 0 ] && contains "$out" "API key required"; then
  pass
else
  fail "API key required" "$(echo "$out" | tr '\n' ' ')"
fi

assert_out "output json"             ""  --output json    $API geocode Berlin
assert_out "output pretty"           ""  --output pretty  $API geocode Berlin
assert_out "output summary"          ""  --output summary $API geocode Berlin
assert_out "verbose flag"            ""  --verbose $API geocode Berlin
assert_out "verbose short -v"        ""  -v $API geocode Berlin

# ---------------------------------------------------------------------------
# S2: Geocode
# ---------------------------------------------------------------------------
section "S2: Geocode"

assert_fail "geocode no args"          "required"   $API geocode
assert_out  "geocode Berlin"           ""           $API geocode Berlin
assert_out  "geocode 'New York'"       ""           $API geocode "New York"
assert_out  "geocode Tokyo"            ""           $API geocode Tokyo
assert_out  "geocode Paris"            ""           $API geocode Paris
assert_out  "geocode 'Buenos Aires'"   ""           $API geocode "Buenos Aires"
assert_out  "geocode 'Cape Town'"      ""           $API geocode "Cape Town"
assert_out  "geocode Sydney"           ""           $API geocode Sydney
assert_out  "geocode Dubai"            ""           $API geocode Dubai
assert_out  "geocode Mumbai"           ""           $API geocode Mumbai
assert_out  "geocode Singapore"        ""           $API geocode Singapore

# Accented / special
assert_out "geocode 'München'"         ""           $API geocode "München"
assert_out "geocode 'São Paulo'"       ""           $API geocode "São Paulo"
assert_out "geocode 'København'"       ""           $API geocode "København"
assert_out "geocode with comma"        ""           $API geocode "Berlin, Germany"
assert_out "geocode long query"        ""           $API geocode "1600 Amphitheatre Parkway Mountain View California"

for p in here google tomtom mapbox radar; do
  assert_out "geocode --provider $p" "" --provider $p $API geocode Berlin
done

# ---------------------------------------------------------------------------
# S3: Reverse geocode
# ---------------------------------------------------------------------------
section "S3: Reverse geocode"

assert_fail "reverse no args"          "required"   $API reverse-geocode
assert_fail "reverse no --lat"         "required"   $API reverse-geocode --lng 13.3
assert_fail "reverse no --lng"         "required"   $API reverse-geocode --lat 52.5

assert_out "reverse Berlin"            ""  $API reverse-geocode --lat 52.5 --lng 13.3
assert_out "reverse NYC"               ""  $API reverse-geocode --lat 40.71 --lng=-74.01
assert_out "reverse Tokyo"             ""  $API reverse-geocode --lat 35.68 --lng=139.76
assert_out "reverse Sydney"            ""  $API reverse-geocode --lat=-33.87 --lng=151.21
assert_out "reverse BuenosAires"       ""  $API reverse-geocode --lat=-34.60 --lng=-58.38
assert_out "reverse Cape Town"         ""  $API reverse-geocode --lat=-33.92 --lng=18.42
assert_out "reverse Dubai"             ""  $API reverse-geocode --lat 25.20 --lng=55.27
assert_out "reverse Mumbai"            ""  $API reverse-geocode --lat 19.08 --lng=72.88
assert_out "reverse Singapore"         ""  $API reverse-geocode --lat 1.35 --lng=103.82
assert_out "reverse London"            ""  $API reverse-geocode --lat 51.51 --lng=-0.13
assert_out "reverse Moscow"            ""  $API reverse-geocode --lat 55.75 --lng=37.62

# Extreme coordinates
assert_out "reverse equator (0,0)"     ""  $API reverse-geocode --lat 0.0 --lng=0.0
assert_out "reverse near pole"         ""  $API reverse-geocode --lat 89.9 --lng=0.0
assert_out "reverse date line +180"    ""  $API reverse-geocode --lat 0.0 --lng=180.0
assert_out "reverse date line -180"    ""  $API reverse-geocode --lat 0.0 --lng=-180.0

for p in here google tomtom mapbox radar; do
  assert_out "reverse --provider $p" "" --provider $p $API reverse-geocode --lat 52.5 --lng 13.3
done

# ---------------------------------------------------------------------------
# S4: Route
# ---------------------------------------------------------------------------
section "S4: Route"

assert_fail "route no args"                  "required"   $API route
assert_fail "route no --origin"              "required"   $API route --destination 52.5,13.3
assert_fail "route no --destination"         "required"   $API route --origin 52.5,13.3

assert_out "route Berlin→Paris"              ""  $API route --origin 52.5,13.3 --destination 48.8,2.3
assert_out "route NYC→Boston"                ""  $API route --origin 40.71,-74.01 --destination 42.36,-71.06
assert_out "route Tokyo→Osaka"               ""  $API route --origin 35.68,139.76 --destination 34.69,135.50
assert_out "route Sydney→Melbourne"          ""  $API route --origin=-33.87,151.21 --destination=-37.81,144.96
assert_out "route London→Edinburgh"          ""  $API route --origin 51.51,-0.13 --destination 55.95,-3.19
assert_out "route Dubai→AbuDhabi"            ""  $API route --origin 25.20,55.27 --destination 24.45,54.38
assert_out "route BuenosAires→Santiago"      ""  $API route --origin=-34.60,-58.38 --destination=-33.45,-70.65
assert_out "route CapeTown→Johannesburg"     ""  $API route --origin=-33.92,18.42 --destination=-26.20,28.05
assert_out "route Mumbai→Delhi"              ""  $API route --origin 19.08,72.88 --destination 28.61,77.23
assert_out "route Singapore→KualaLumpur"     ""  $API route --origin 1.35,103.82 --destination 3.14,101.69
assert_out "route Moscow→StPetersburg"       ""  $API route --origin 55.75,37.62 --destination 59.93,30.34

# Coordinate validation
assert_fail "route invalid origin 'abc'"     "Invalid origin"        $API route --origin abc --destination 52.5,13.3
assert_fail "route invalid origin 'abc,def'" "Invalid origin"        $API route --origin abc,def --destination 52.5,13.3
assert_fail "route invalid dest 'bad'"       "Invalid destination"   $API route --origin 52.5,13.3 --destination bad
assert_fail "route single comma origin"      "Invalid origin"        $API route --origin "52.5," --destination 52.5,13.3
assert_fail "route single comma dest"        "Invalid destination"   $API route --origin 52.5,13.3 --destination "48.8,"
assert_out  "route negative lng in coords"   ""                      $API route --origin 40.71,-74.01 --destination 42.36,-71.06

# Transport modes — all 7
for t in car truck pedestrian bicycle scooter bus taxi; do
  assert_out "route --transport $t" "" $API route --origin 52.5,13.3 --destination 48.8,2.3 --transport $t
done
assert_out "route unknown transport falls back" "" $API route --origin 52.5,13.3 --destination 48.8,2.3 --transport unicycle

for p in here google tomtom mapbox radar; do
  assert_out "route --provider $p" "" --provider $p $API route --origin 52.5,13.3 --destination 48.8,2.3
done

# ---------------------------------------------------------------------------
# S5: Traffic
# ---------------------------------------------------------------------------
section "S5: Traffic"

assert_fail "traffic no args"          "required"   $API traffic
assert_fail "traffic no --lat"         "required"   $API traffic --lng 13.3
assert_fail "traffic no --lng"         "required"   $API traffic --lat 52.5

assert_out "traffic Berlin"            ""  $API traffic --lat 52.5 --lng 13.3
assert_out "traffic with --radius"     ""  $API traffic --lat 52.5 --lng 13.3 --radius 5000
assert_out "traffic --radius 0"        ""  $API traffic --lat 52.5 --lng 13.3 --radius 0
assert_out "traffic --radius 100000"   ""  $API traffic --lat 52.5 --lng 13.3 --radius 100000
assert_out "traffic with incidents"    ""  $API traffic --lat 52.5 --lng 13.3 --include-incidents true
assert_out "traffic no incidents"      ""  $API traffic --lat 52.5 --lng 13.3 --include-incidents false
assert_out "traffic radius+incidents"  ""  $API traffic --lat 52.5 --lng 13.3 --radius 10000 --include-incidents true

assert_out "traffic NYC"               ""  $API traffic --lat 40.71 --lng=-74.01
assert_out "traffic Tokyo"             ""  $API traffic --lat 35.68 --lng=139.76
assert_out "traffic Sydney"            ""  $API traffic --lat=-33.87 --lng=151.21
assert_out "traffic London"            ""  $API traffic --lat 51.51 --lng=-0.13

# Unsupported
for p in google mapbox radar; do
  assert_err "traffic unsupported $p" "does not support traffic" --provider $p $API traffic --lat 52.5 --lng 13.3
done
for p in here tomtom; do
  assert_out "traffic supported $p" "" --provider $p $API traffic --lat 52.5 --lng 13.3
done

# ---------------------------------------------------------------------------
# S6: Position
# ---------------------------------------------------------------------------
section "S6: Position"

assert_out "position HERE"    ""  --provider here   $API position
assert_out "position Google"  ""  --provider google $API position

for p in tomtom mapbox radar; do
  assert_err "position unsupported $p" "does not support positioning" --provider $p $API position
done

# ---------------------------------------------------------------------------
# S7: Isoline
# ---------------------------------------------------------------------------
section "S7: Isoline"

assert_fail "isoline no args"          "required"   $API isoline
assert_fail "isoline no --lat"         "required"   $API isoline --lng 13.3
assert_fail "isoline no --lng"         "required"   $API isoline --lat 52.5

assert_out "isoline default range"     ""  $API isoline --lat 52.5 --lng 13.3
assert_out "isoline range 500"         ""  $API isoline --lat 52.5 --lng 13.3 --range 500
assert_out "isoline range 5000"        ""  $API isoline --lat 52.5 --lng 13.3 --range 5000
assert_out "isoline range 0"           ""  $API isoline --lat 52.5 --lng 13.3 --range 0
assert_out "isoline range 100000"      ""  $API isoline --lat 52.5 --lng 13.3 --range 100000

# Range types
assert_out "isoline range-type distance" ""  $API isoline --lat 52.5 --lng 13.3 --range-type distance
assert_out "isoline range-type time"     ""  $API isoline --lat 52.5 --lng 13.3 --range-type time
assert_out "isoline range-type garbage"  ""  $API isoline --lat 52.5 --lng 13.3 --range-type garbage

# Transport modes for isoline — all 7
for t in car truck pedestrian bicycle scooter bus taxi; do
  assert_out "isoline --transport $t" "" $API isoline --lat 52.5 --lng 13.3 --transport $t
done

# Global
assert_out "isoline NYC"               ""  $API isoline --lat 40.71 --lng=-74.01
assert_out "isoline Tokyo"             ""  $API isoline --lat 35.68 --lng=139.76
assert_out "isoline Sydney"            ""  $API isoline --lat=-33.87 --lng=151.21
assert_out "isoline London"            ""  $API isoline --lat 51.51 --lng=-0.13
assert_out "isoline Singapore"         ""  $API isoline --lat 1.35 --lng=103.82

# Combined flags
assert_out "isoline range+transport+type" ""  $API isoline --lat 52.5 --lng 13.3 --range 5000 --transport pedestrian --range-type time

# Unsupported
for p in google radar; do
  assert_err "isoline unsupported $p" "does not support isoline" --provider $p $API isoline --lat 52.5 --lng 13.3
done
for p in here tomtom mapbox; do
  assert_out "isoline supported $p" "" --provider $p $API isoline --lat 52.5 --lng 13.3
done

# ---------------------------------------------------------------------------
# S8: Match-route
# ---------------------------------------------------------------------------
section "S8: Match-route"

assert_fail "match-route no args"         "required"               $API match-route
assert_fail "match-route empty trace"     "No valid coordinates"   $API match-route --trace ""
assert_fail "match-route invalid trace"   "No valid coordinates"   $API match-route --trace "invalid;alsobad"

assert_out "match-route 2-point"          ""  $API match-route --trace "52.5,13.3;52.6,13.4"
assert_out "match-route 3-point"          ""  $API match-route --trace "52.5,13.3;52.6,13.4;52.7,13.5"
assert_out "match-route 5-point"          ""  $API match-route --trace "52.5,13.3;52.6,13.4;52.7,13.5;52.8,13.6;52.9,13.7"
assert_out "match-route single point"     ""  $API match-route --trace "52.5,13.3"
assert_out "match-route many points (10)" ""  $API match-route --trace "52.5,13.3;52.51,13.31;52.52,13.32;52.53,13.33;52.54,13.34;52.55,13.35;52.56,13.36;52.57,13.37;52.58,13.38;52.59,13.39"

# Mixed valid/invalid
assert_out "match-route mixed valid" ""  $API match-route --trace "52.5,13.3;invalid;53.0,14.0"

# Transport modes — all 7
for t in car truck pedestrian bicycle scooter bus taxi; do
  assert_out "match-route --transport $t" "" $API match-route --trace "52.5,13.3;52.6,13.4" --transport $t
done

# Global traces
assert_out "match-route NYC"     ""  $API match-route --trace "40.71,-74.01;40.72,-74.00;40.73,-73.99"
assert_out "match-route London"  ""  $API match-route --trace "51.51,-0.13;51.52,-0.12;51.53,-0.11"
assert_out "match-route Tokyo"   ""  $API match-route --trace "35.68,139.76;35.69,139.77;35.70,139.78"
assert_out "match-route Sydney"  ""  $API match-route --trace=-33.87,151.21\;-33.88,151.22\;-33.89,151.23

for p in here google tomtom mapbox radar; do
  assert_out "match-route --provider $p" "" --provider $p $API match-route --trace "52.5,13.3;52.6,13.4"
done

# ---------------------------------------------------------------------------
# S9: Tour
# ---------------------------------------------------------------------------
section "S9: Tour"

assert_fail "tour no args"             "required"         $API tour
assert_fail "tour 1 stop"              "At least 2 stops" $API tour --stops 52.5,13.3
assert_fail "tour all invalid"         "At least 2 stops" $API tour --stops bad alsobad
assert_fail "tour 1 valid 1 invalid"   "At least 2 stops" $API tour --stops 52.5,13.3 badformat

assert_out "tour 2 stops"              ""  $API tour --stops 52.5,13.3 48.8,2.3
assert_out "tour 3 stops"              ""  $API tour --stops 52.5,13.3 48.8,2.3 40.71,-74.01
assert_out "tour 5 stops"              ""  $API tour --stops 52.5,13.3 48.8,2.3 40.71,-74.01 35.68,139.76 --stops=-33.87,151.21

# Transport modes
for t in car truck pedestrian bicycle; do
  assert_out "tour --transport $t" "" $API tour --stops 52.5,13.3 48.8,2.3 --transport $t
done

# Departure time
assert_out "tour with departure ISO"   ""  $API tour --stops 52.5,13.3 48.8,2.3 --departure "2026-04-21T08:00:00Z"
assert_out "tour with departure 'now'" ""  $API tour --stops 52.5,13.3 48.8,2.3 --departure "now"
assert_out "tour with departure empty" ""  $API tour --stops 52.5,13.3 48.8,2.3 --departure ""

# Global
assert_out "tour NYC"                  ""  $API tour --stops 40.71,-74.01 42.36,-71.06
assert_out "tour Tokyo"                ""  $API tour --stops 35.68,139.76 34.69,135.50 35.17,136.91
assert_out "tour Australia"            ""  $API tour --stops=-33.87,151.21 --stops=-37.81,144.96 --stops=-27.47,153.03
assert_out "tour Europe"               ""  $API tour --stops 51.51,-0.13 48.8,2.3 52.5,13.3 41.39,2.17
assert_out "tour multi-continent"      ""  $API tour --stops 51.51,-0.13 --stops=-33.87,151.21 --stops 35.68,139.76 --stops 40.71,-74.01 --stops 52.5,13.3

for p in here tomtom mapbox radar; do
  assert_out "tour --provider $p" "" --provider $p $API tour --stops 52.5,13.3 48.8,2.3
done
assert_err "tour unsupported google" "does not support tour" --provider google $API tour --stops 52.5,13.3 48.8,2.3

# ---------------------------------------------------------------------------
# S10: Tile
# ---------------------------------------------------------------------------
section "S10: Tile"

assert_fail "tile no args"     "required"   $API tile
assert_fail "tile no --z"      "required"   $API tile --x 8800 --y 5374
assert_fail "tile no --x"      "required"   $API tile --z 14 --y 5374
assert_fail "tile no --y"      "required"   $API tile --z 14 --x 8800

assert_out "tile z14"          ""  $API tile --z 14 --x 8800 --y 5374
assert_out "tile z0"           ""  $API tile --z 0 --x 0 --y 0
assert_out "tile z10"          ""  $API tile --z 10 --x 550 --y 335
assert_out "tile z18"          ""  $API tile --z 18 --x 140800 --y 85984
assert_out "tile z22"          ""  $API tile --z 22 --x 4000000 --y 3000000

# Layer
assert_out "tile --layer base"     ""  $API tile --z 14 --x 8800 --y 5374 --layer base
assert_out "tile --layer hybrid"   ""  $API tile --z 14 --x 8800 --y 5374 --layer hybrid
assert_out "tile --layer labels"   ""  $API tile --z 14 --x 8800 --y 5374 --layer labels
assert_out "tile --layer empty"    ""  $API tile --z 14 --x 8800 --y 5374 --layer ""

# Custom output file
OUT="/tmp/everymap-validate-tile.omv"
rm -f "$OUT"
assert_out "tile --output-file"    ""  $API tile --z 14 --x 8800 --y 5374 --output-file "$OUT"
rm -f "$OUT"

# Global tiles
assert_out "tile NYC"              ""  $API tile --z 12 --x 1206 --y 1539
assert_out "tile London"           ""  $API tile --z 12 --x 2048 --y 1361
assert_out "tile Tokyo"            ""  $API tile --z 12 --x 3637 --y 1612
assert_out "tile Sydney"           ""  $API tile --z 12 --x 3776 --y 2379
assert_out "tile HERE scheme"      ""  $API tile --z 14 --x 4494 --y 2832

for p in here tomtom mapbox; do
  assert_out "tile --provider $p" "" --provider $p $API tile --z 14 --x 8800 --y 5374
done
for p in google radar; do
  assert_err "tile unsupported $p" "does not support tiling" --provider $p $API tile --z 14 --x 8800 --y 5374
done

# ---------------------------------------------------------------------------
# S11: Attributes
# ---------------------------------------------------------------------------
section "S11: Attributes"

assert_out "attributes no bbox HERE"    ""  --provider here   $API attributes
assert_out "attributes no bbox Google"  ""  --provider google $API attributes

assert_out "attributes bbox HERE"       ""  --provider here   $API attributes --bbox "52.4,13.2;52.6,13.5"
assert_out "attributes bbox Google"     ""  --provider google $API attributes --bbox "40.7,-74.1;40.8,-73.9"

# Layer
assert_out "attributes --layer roads"       ""  --provider here $API attributes --bbox "52.4,13.2;52.6,13.5" --layer roads
assert_out "attributes --layer segments"    ""  --provider here $API attributes --bbox "52.4,13.2;52.6,13.5" --layer segments
assert_out "attributes --layer adminAreas"  ""  --provider here $API attributes --bbox "52.4,13.2;52.6,13.5" --layer adminAreas

# Format
assert_out "attributes --format json"     ""  --provider here $API attributes --bbox "52.4,13.2;52.6,13.5" --format json
assert_out "attributes --format geojson"  ""  --provider here $API attributes --bbox "52.4,13.2;52.6,13.5" --format geojson

# IDs (fixed: now takes multiple values)
assert_out "attributes --ids multi"     ""  --provider here $API attributes --bbox "52.4,13.2;52.6,13.5" --ids id1 id2 id3
assert_out "attributes --ids single"    ""  --provider here $API attributes --bbox "52.4,13.2;52.6,13.5" --ids onlyone
assert_out "attributes --ids empty"     ""  --provider here $API attributes --bbox "52.4,13.2;52.6,13.5" --ids

# Include
assert_out "attributes --include one"   ""  --provider here $API attributes --bbox "52.4,13.2;52.6,13.5" --include "speedLimit"
assert_out "attributes --include many"  ""  --provider here $API attributes --bbox "52.4,13.2;52.6,13.5" --include "speedLimit,roadClass,functionalClass"

# All options combined
assert_out "attributes all options" "" --provider here $API attributes \
  --bbox "52.4,13.2;52.6,13.5" --layer roads --format geojson --ids id1 id2 id3 --include "speedLimit,roadClass,functionalClass"

# Global bboxes
assert_out "attributes NYC"     ""  --provider here $API attributes --bbox "40.7,-74.1;40.8,-73.9"
assert_out "attributes London"  ""  --provider here $API attributes --bbox "51.4,-0.2;51.6,-0.05"
assert_out "attributes Tokyo"   ""  --provider here $API attributes --bbox "35.6,139.7;35.7,139.8"

# Unsupported
for p in tomtom mapbox radar; do
  assert_err "attributes unsupported $p" "does not support attributes" --provider $p $API attributes --bbox "52.4,13.2;52.6,13.5"
done

# ---------------------------------------------------------------------------
# S12: Map-image
# ---------------------------------------------------------------------------
section "S12: Map-image"

assert_fail "map-image no args"       "required"   $API map-image
assert_fail "map-image no --lat"      "required"   $API map-image --lng 13.3
assert_fail "map-image no --lng"      "required"   $API map-image --lat 52.5

assert_out "map-image Berlin z14"     ""  $API map-image --lat 52.5 --lng 13.3
assert_out "map-image Berlin zoom 14" ""  $API map-image --lat 52.5 --lng 13.3 --zoom 14
assert_out "map-image z1"             ""  $API map-image --lat 52.5 --lng 13.3 --zoom 1
assert_out "map-image z10"            ""  $API map-image --lat 52.5 --lng 13.3 --zoom 10
assert_out "map-image z18"            ""  $API map-image --lat 52.5 --lng 13.3 --zoom 18

# Custom output file
OUT="/tmp/everymap-validate-image.png"
rm -f "$OUT"
assert_out "map-image --output-file"  ""  $API map-image --lat 52.5 --lng 13.3 --output-file "$OUT"
rm -f "$OUT"

# Global
assert_out "map-image NYC"          ""  $API map-image --lat 40.71 --lng=-74.01 --zoom 14
assert_out "map-image Tokyo"        ""  $API map-image --lat 35.68 --lng=139.76 --zoom 14
assert_out "map-image Sydney"       ""  $API map-image --lat=-33.87 --lng=151.21 --zoom 14
assert_out "map-image London"       ""  $API map-image --lat 51.51 --lng=-0.13 --zoom 14
assert_out "map-image Singapore"    ""  $API map-image --lat 1.35 --lng=103.82 --zoom 14
assert_out "map-image Dubai"        ""  $API map-image --lat 25.20 --lng=55.27 --zoom 14
assert_out "map-image CapeTown"     ""  $API map-image --lat=-33.92 --lng=18.42 --zoom 14
assert_out "map-image Reykjavik"    ""  $API map-image --lat 64.15 --lng=-21.94 --zoom 14
assert_out "map-image Ushuaia"      ""  $API map-image --lat=-54.80 --lng=-68.30 --zoom 14

for p in here google tomtom mapbox; do
  assert_out "map-image --provider $p" "" --provider $p $API map-image --lat 52.5 --lng 13.3
done
assert_err "map-image unsupported radar" "does not support imaging" --provider radar $API map-image --lat 52.5 --lng 13.3

# ---------------------------------------------------------------------------
# S13: Subcommand help
# ---------------------------------------------------------------------------
section "S13: Subcommand help"

assert_out "help geocode"                  "<QUERY>"          geocode --help
assert_out "help reverse --lat"            "--lat"            reverse-geocode --help
assert_out "help reverse --lng"            "--lng"            reverse-geocode --help
assert_out "help route --origin"           "--origin"         route --help
assert_out "help route --destination"      "--destination"    route --help
assert_out "help route --transport"        "--transport"      route --help
assert_out "help traffic --lat"            "--lat"            traffic --help
assert_out "help traffic --radius"         "--radius"         traffic --help
assert_out "help position"                 "position"         position --help
assert_out "help isoline --range"          "--range"          isoline --help
assert_out "help isoline --range-type"     "--range-type"     isoline --help
assert_out "help match-route --trace"      "--trace"          match-route --help
assert_out "help match-route --transport"  "--transport"      match-route --help
assert_out "help tour --stops"             "--stops"          tour --help
assert_out "help tour --departure"         "--departure"      tour --help
assert_out "help tile --z"                 "--z"              tile --help
assert_out "help tile --layer"             "--layer"          tile --help
assert_out "help tile --output-file"       "--output-file"    tile --help
assert_out "help attributes --bbox"        "--bbox"           attributes --help
assert_out "help attributes --layer"       "--layer"          attributes --help
assert_out "help attributes --format"      "--format"         attributes --help
assert_out "help attributes --ids"         "--ids"            attributes --help
assert_out "help attributes --include"     "--include"        attributes --help
assert_out "help map-image --zoom"         "--zoom"           map-image --help
assert_out "help map-image --output-file"  "--output-file"    map-image --help

# ---------------------------------------------------------------------------
# S14: Output formats
# ---------------------------------------------------------------------------
section "S14: Output formats"

assert_out "output json geocode"         ""  --output json     $API geocode Berlin
assert_out "output pretty geocode"       ""  --output pretty   $API geocode Berlin
assert_out "output summary geocode"      ""  --output summary  $API geocode Berlin
assert_out "output summary route"        ""  --output summary  $API route --origin 52.5,13.3 --destination 48.8,2.3
assert_out "output summary traffic"      ""  --output summary  --provider here $API traffic --lat 52.5 --lng 13.3
assert_out "output summary isoline"      ""  --output summary  $API isoline --lat 52.5 --lng 13.3
assert_out "output summary tour"         ""  --output summary  $API tour --stops 52.5,13.3 48.8,2.3
assert_out "output summary match-route"  ""  --output summary  $API match-route --trace "52.5,13.3;52.6,13.4"
assert_out "output summary attributes"   ""  --output summary  --provider here $API attributes --bbox "52.4,13.2;52.6,13.5"
assert_out "output summary position"     ""  --output summary  --provider here $API position
assert_out "output invalid xml"          ""  --output xml      $API geocode Berlin
assert_out "output empty string"         ""  --output ""       $API geocode Berlin

# ---------------------------------------------------------------------------
# S15: Combined flags & edge cases
# ---------------------------------------------------------------------------
section "S15: Combined flags & edge cases"

assert_out "verbose+json"              ""  --verbose --output json    $API geocode Berlin
assert_out "verbose+pretty"            ""  --verbose --output pretty  $API geocode Berlin
assert_out "verbose+summary"           ""  -v --output summary        $API geocode Berlin

# --api-key-param overrides
assert_out "api-key-param custom"      ""  --api-key-param custom_key  $API geocode Berlin
assert_out "api-key-param google"      ""  --provider google --api-key-param myparam $API geocode Berlin
assert_out "api-key-param mapbox"      ""  --provider mapbox --api-key-param token $API geocode Berlin
assert_out "api-key-param empty"       ""  --api-key-param ""  $API geocode Berlin

# Cross-equator and hemisphere
assert_out "route cross-equator"       ""  $API route --origin 1.35,103.82 --destination=-33.92,18.42
assert_out "route hemispheres"         ""  $API route --origin 51.51,-0.13 --destination=-33.87,151.21

# Tour combined
assert_out "tour multi-continent" "" $API tour --stops 51.51,-0.13 --stops=-33.87,151.21 --stops 35.68,139.76 --stops 40.71,-74.01 --stops 52.5,13.3

# ---------------------------------------------------------------------------
# S16: Negative longitude --lng= syntax
# ---------------------------------------------------------------------------
section "S16: Negative longitude syntax"

assert_out "reverse --lng= neg"       ""  $API reverse-geocode --lat 40.71 --lng=-74.01
assert_out "traffic --lng= neg"       ""  $API traffic --lat 40.71 --lng=-74.01
assert_out "isoline --lng= neg"       ""  $API isoline --lat 40.71 --lng=-74.01
assert_out "map-image --lng= neg"     ""  $API map-image --lat 40.71 --lng=-74.01 --zoom 14
assert_out "route neg lng in coords"  ""  $API route --origin 40.71,-74.01 --destination 42.36,-71.06

# ---------------------------------------------------------------------------
# S17: Provider×Domain matrix — every provider, every command
# ---------------------------------------------------------------------------
section "S17: Provider×Domain matrix"

# HERE: all 11 supported
for cmd in \
  "geocode Berlin" \
  "reverse-geocode --lat 52.5 --lng 13.3" \
  "route --origin 52.5,13.3 --destination 48.8,2.3" \
  "traffic --lat 52.5 --lng 13.3" \
  "position" \
  "isoline --lat 52.5 --lng 13.3" \
  "match-route --trace 52.5,13.3;52.6,13.4" \
  "tour --stops 52.5,13.3 48.8,2.3" \
  "tile --z 14 --x 8800 --y 5374" \
  "attributes --bbox 52.4,13.2;52.6,13.5" \
  "map-image --lat 52.5 --lng 13.3"; do
  assert_out "HERE: $cmd" "" --provider here $API $cmd
done

# Google: 7 supported, 4 unsupported
for cmd in \
  "geocode Berlin" \
  "reverse-geocode --lat 52.5 --lng 13.3" \
  "route --origin 52.5,13.3 --destination 48.8,2.3" \
  "position" \
  "match-route --trace 52.5,13.3;52.6,13.4" \
  "attributes --bbox 52.4,13.2;52.6,13.5" \
  "map-image --lat 52.5 --lng 13.3"; do
  assert_out "Google: $cmd" "" --provider google $API $cmd
done
for cmd in \
  "traffic --lat 52.5 --lng 13.3" \
  "isoline --lat 52.5 --lng 13.3" \
  "tour --stops 52.5,13.3 48.8,2.3" \
  "tile --z 14 --x 8800 --y 5374"; do
  assert_err "Google unsupported: $cmd" "does not support" --provider google $API $cmd
done

# TomTom: 9 supported, 2 unsupported
for cmd in \
  "geocode Berlin" \
  "reverse-geocode --lat 52.5 --lng 13.3" \
  "route --origin 52.5,13.3 --destination 48.8,2.3" \
  "traffic --lat 52.5 --lng 13.3" \
  "isoline --lat 52.5 --lng 13.3" \
  "match-route --trace 52.5,13.3;52.6,13.4" \
  "tour --stops 52.5,13.3 48.8,2.3" \
  "tile --z 14 --x 8800 --y 5374" \
  "map-image --lat 52.5 --lng 13.3"; do
  assert_out "TomTom: $cmd" "" --provider tomtom $API $cmd
done
for cmd in \
  "position" \
  "attributes --bbox 52.4,13.2;52.6,13.5"; do
  assert_err "TomTom unsupported: $cmd" "does not support" --provider tomtom $API $cmd
done

# MapBox: 8 supported, 3 unsupported
for cmd in \
  "geocode Berlin" \
  "reverse-geocode --lat 52.5 --lng 13.3" \
  "route --origin 52.5,13.3 --destination 48.8,2.3" \
  "isoline --lat 52.5 --lng 13.3" \
  "match-route --trace 52.5,13.3;52.6,13.4" \
  "tour --stops 52.5,13.3 48.8,2.3" \
  "tile --z 14 --x 8800 --y 5374" \
  "map-image --lat 52.5 --lng 13.3"; do
  assert_out "MapBox: $cmd" "" --provider mapbox $API $cmd
done
for cmd in \
  "traffic --lat 52.5 --lng 13.3" \
  "position" \
  "attributes --bbox 52.4,13.2;52.6,13.5"; do
  assert_err "MapBox unsupported: $cmd" "does not support" --provider mapbox $API $cmd
done

# Radar: 5 supported, 6 unsupported
for cmd in \
  "geocode Berlin" \
  "reverse-geocode --lat 52.5 --lng 13.3" \
  "route --origin 52.5,13.3 --destination 48.8,2.3" \
  "match-route --trace 52.5,13.3;52.6,13.4" \
  "tour --stops 52.5,13.3 48.8,2.3"; do
  assert_out "Radar: $cmd" "" --provider radar $API $cmd
done
for cmd in \
  "traffic --lat 52.5 --lng 13.3" \
  "position" \
  "isoline --lat 52.5 --lng 13.3" \
  "tile --z 14 --x 8800 --y 5374" \
  "attributes --bbox 52.4,13.2;52.6,13.5" \
  "map-image --lat 52.5 --lng 13.3"; do
  assert_err "Radar unsupported: $cmd" "does not support" --provider radar $API $cmd
done

# ---------------------------------------------------------------------------
# Summary report
# ---------------------------------------------------------------------------
section "Results"

PASS=$(cat "$PASS_FILE"); FAIL=$(cat "$FAIL_FILE"); TOTAL=$(cat "$TOTAL_FILE")
END_TIME=$(date +%s); ELAPSED=$((END_TIME - START_TIME))

# Generate summary
SUMMARY_FILE="${RESULTS_DIR}/SUMMARY.txt"
{
  printf "EveryMap CLI Validation Report\n"
  printf "==============================\n"
  printf "Date:       %s\n" "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  printf "Binary:     %s\n" "$CLI_BIN"
  printf "Duration:   %ds\n" "$ELAPSED"
  printf "Tests:      %d total\n" "$TOTAL"
  printf "Passed:     %d\n" "$PASS"
  printf "Failed:     %d\n" "$FAIL"
  if [[ "$TOTAL" -gt 0 ]]; then
    printf "Pass rate:  %.1f%%\n" "$(echo "scale=1; $PASS * 100 / $TOTAL" | bc)"
  fi
} > "$SUMMARY_FILE"

cat "$SUMMARY_FILE"

# Generate JSON results (always, for CI consumption)
PASS_RATE="N/A"
[[ "$TOTAL" -gt 0 ]] && PASS_RATE=$(echo "scale=1; $PASS * 100 / $TOTAL" | bc)

cat > "${RESULTS_DIR}/results.json" <<JSONEOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "binary": "$CLI_BIN",
  "duration_s": $ELAPSED,
  "total": $TOTAL,
  "passed": $PASS,
  "failed": $FAIL,
  "pass_rate": $PASS_RATE
}
JSONEOF

# Print JSON to stdout if requested
if $JSON_OUT; then
  cat "${RESULTS_DIR}/results.json"
fi

printf "\nDetails:  %s/\n" "$DETAILS_DIR"
printf "Report:   %s\n" "$SUMMARY_FILE"
echo ""

if [[ "$FAIL" -gt 0 ]]; then
  exit 1
fi
exit 0
