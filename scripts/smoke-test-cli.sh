#!/usr/bin/env bash
# =============================================================================
# smoke-test-cli.sh — Extensive CLI smoke tests across all providers & domains
#
# Tests real API calls with diverse locations, all output formats, verbose mode,
# edge cases, and error paths. Captures every command's stdout, stderr, exit
# code, and timing for post-run analysis.
#
# Usage:
#   ./scripts/smoke-test-cli.sh              # full run (all providers)
#   ./scripts/smoke-test-cli.sh --here       # only HERE
#   ./scripts/smoke-test-cli.sh --google     # only Google
#   ./scripts/smoke-test-cli.sh --tomtom     # only TomTom
#   ./scripts/smoke-test-cli.sh --mapbox     # only MapBox
#   ./scripts/smoke-test-cli.sh --radar      # only Radar
#   ./scripts/smoke-test-cli.sh --quick      # subset: 1 location per domain
#   ./scripts/smoke-test-cli.sh --no-build   # skip cargo build step
#
# Output: smoke-test-results/<timestamp>/
#   ├── <provider>_<command>_<variant>.out    (stdout)
#   ├── <provider>_<command>_<variant>.err   (stderr)
#   ├── <provider>_<command>_<variant>.code  (exit code)
#   ├── <provider>_<command>_<variant>.time  (wall-clock ms)
#   └── SUMMARY.md                           (human-readable report)
# =============================================================================

set -uo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="${PROJECT_DIR}/smoke-test-results/$(date +%Y%m%d_%H%M%S)"
CLI_BIN="${PROJECT_DIR}/target/debug/everymap"
SLEEP_BETWEEN_CALLS="${SMOKE_SLEEP:-0.3}"  # seconds between API calls

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Counters
TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

# Provider filter (empty = all)
FILTER_PROVIDER=""

# Quick mode flag
QUICK_MODE=false

# Skip build flag
NO_BUILD=false

# Parse args
while [[ $# -gt 0 ]]; do
    case "$1" in
        --here)    FILTER_PROVIDER="here";    shift ;;
        --google)  FILTER_PROVIDER="google";  shift ;;
        --tomtom)  FILTER_PROVIDER="tomtom";  shift ;;
        --mapbox)  FILTER_PROVIDER="mapbox";  shift ;;
        --radar)   FILTER_PROVIDER="radar";   shift ;;
        --quick)   QUICK_MODE=true;           shift ;;
        --no-build) NO_BUILD=true;           shift ;;
        -h|--help)
            echo "Usage: $0 [--here|--google|--tomtom|--mapbox|--radar] [--quick] [--no-build]"
            exit 0 ;;
        *) echo "Unknown argument: $1"; exit 1 ;;
    esac
done

# ---------------------------------------------------------------------------
# API keys (from .env file)
# ---------------------------------------------------------------------------

# Source .env if it exists
if [[ -f "${PROJECT_DIR}/.env" ]]; then
    set -a
    source "${PROJECT_DIR}/.env"
    set +a
fi

HERE_KEY="${EVERYMAP_HERE_API_KEY:-${EVERYMAP_API_KEY:-}}"
GOOGLE_KEY="${EVERYMAP_GOOGLE_API_KEY:-${EVERYMAP_API_KEY:-}}"
TOMTOM_KEY="${EVERYMAP_TOMTOM_API_KEY:-${EVERYMAP_API_KEY:-}}"
MAPBOX_KEY="${EVERYMAP_MAPBOX_API_KEY:-${EVERYMAP_API_KEY:-}}"
RADAR_KEY="${EVERYMAP_RADAR_API_KEY:-${EVERYMAP_API_KEY:-}}"

# ---------------------------------------------------------------------------
# Test locations — diverse global coverage
# ---------------------------------------------------------------------------

# Each location: "name|query|lat|lng"
# Using pipe-delimited to avoid array-of-arrays issues in bash

LOCATIONS=(
    "berlin|Berlin|52.52|13.40"
    "tokyo|Tokyo|35.68|139.69"
    "nyc|New York|40.71|-74.01"
    "sydney|Sydney|-33.87|151.21"
    "sao_paulo|São Paulo|-23.55|-46.63"
    "london|London|51.51|-0.13"
    "lagos|Lagos|6.52|3.38"
    "dubai|Dubai|25.20|55.27"
    "sf|San Francisco|37.77|-122.42"
    "paris|Paris|48.86|2.35"
)

# Route pairs: "name|origin_lat,origin_lng|dest_lat,dest_lng"
ROUTES=(
    "berlin_paris|52.52,13.40|48.86,2.35"
    "nyc_boston|40.71,-74.01|42.36,-71.06"
    "tokyo_osaka|35.68,139.69|34.69,135.50"
    "sf_la|37.77,-122.42|34.05,-118.24"
    "london_paris|51.51,-0.13|48.86,2.35"
)

# Dense GPS traces for match-route (semicolons, close points along real roads)
TRACES=(
    "berlin_trace|52.5164,13.3777;52.5170,13.3900;52.5175,13.3950;52.5180,13.4000"
    "nyc_trace|40.7128,-74.0060;40.7135,-74.0050;40.7142,-74.0040;40.7150,-74.0030"
    "london_trace|51.5074,-0.1278;51.5080,-0.1260;51.5085,-0.1245;51.5090,-0.1230"
    "tokyo_trace|35.6812,139.7671;35.6820,139.7680;35.6828,139.7690;35.6835,139.7700"
)

# HERE tile coordinates (HERE's own tiling scheme, NOT web Mercator)
HERE_TILES=(
    "berlin_z14|14|4494|2832"
    "nyc_z14|14|2608|6332"
    "tokyo_z14|14|7278|4390"
)

# Web Mercator tile coordinates (Slippy map — TomTom, MapBox)
WEB_TILES=(
    "berlin_z14|14|8800|5374"
    "nyc_z14|14|4651|6159"
    "tokyo_z14|14|14555|6443"
)

# Bounding boxes: "name|south,west;north,east"
BBOXES=(
    "berlin_bbox|52.4,13.2;52.6,13.5"
    "nyc_bbox|40.6,-74.1;40.8,-74.0"
    "tokyo_bbox|35.6,139.6;35.8,139.8"
)

# ---------------------------------------------------------------------------
# Helper functions
# ---------------------------------------------------------------------------

loc_name()  { echo "$1" | cut -d'|' -f1; }
loc_query() { echo "$1" | cut -d'|' -f2; }
loc_lat()   { echo "$1" | cut -d'|' -f3; }
loc_lng()   { echo "$1" | cut -d'|' -f4; }

route_name() { echo "$1" | cut -d'|' -f1; }
route_orig() { echo "$1" | cut -d'|' -f2; }
route_dest() { echo "$1" | cut -d'|' -f3; }

trace_name()  { echo "$1" | cut -d'|' -f1; }
trace_points() { echo "$1" | cut -d'|' -f2; }

sanitize_name() {
    echo "$1" | tr ' ' '_' | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9_]/_/g'
}

# Run a single CLI command, capture everything
run_cli() {
    local test_name="$1"
    shift

    local safe_name
    safe_name="$(sanitize_name "$test_name")"
    local out_file="${OUTPUT_DIR}/${safe_name}.out"
    local err_file="${OUTPUT_DIR}/${safe_name}.err"
    local code_file="${OUTPUT_DIR}/${safe_name}.code"
    local time_file="${OUTPUT_DIR}/${safe_name}.time"

    TOTAL=$((TOTAL + 1))

    local start_ms
    start_ms="$(python3 -c 'import time; print(int(time.time()*1000))' 2>/dev/null || echo 0)"

    set +e
    "$CLI_BIN" "$@" > "$out_file" 2> "$err_file"
    local exit_code=$?
    set -e

    local end_ms
    end_ms="$(python3 -c 'import time; print(int(time.time()*1000))' 2>/dev/null || echo 0)"
    local elapsed=$((end_ms - start_ms))

    echo "$exit_code" > "$code_file"
    echo "$elapsed" > "$time_file"

    # Classify result
    if [[ "$exit_code" -eq 0 ]]; then
        PASSED=$((PASSED + 1))
        printf "  ${GREEN}PASS${NC} [%5dms] %s\n" "$elapsed" "$test_name"
    else
        FAILED=$((FAILED + 1))
        local err_preview
        err_preview="$(head -c 200 "$err_file" 2>/dev/null | tr '\n' ' ')"
        printf "  ${RED}FAIL${NC} [%5dms] %s  (exit=%d, stderr: %s)\n" "$elapsed" "$test_name" "$exit_code" "$err_preview"
    fi

    # Sleep between calls to respect rate limits
    sleep "$SLEEP_BETWEEN_CALLS"
}

# Run a CLI command that saves a binary file; verify the file exists and is non-empty
run_cli_binary() {
    local test_name="$1"
    local expected_file="$2"
    shift 2

    run_cli "$test_name" "$@"

    local safe_name
    safe_name="$(sanitize_name "$test_name")"
    local code_file="${OUTPUT_DIR}/${safe_name}.code"
    local exit_code
    exit_code="$(cat "$code_file")"

    if [[ "$exit_code" -eq 0 ]]; then
        if [[ -f "$expected_file" ]] && [[ "$(stat -f%z "$expected_file" 2>/dev/null || stat -c%s "$expected_file" 2>/dev/null)" -gt 0 ]]; then
            local fsize
            fsize="$(stat -f%z "$expected_file" 2>/dev/null || stat -c%s "$expected_file" 2>/dev/null)"
            printf "  ${CYAN}FILE${NC} [%7d bytes] %s\n" "$fsize" "$expected_file"
            # Move file info into output
            echo "File: $expected_file ($fsize bytes)" >> "${OUTPUT_DIR}/${safe_name}.out"
        else
            printf "  ${RED}MISS${NC} file missing or empty: %s\n" "$expected_file"
            # Override to FAIL
            PASSED=$((PASSED - 1))
            FAILED=$((FAILED + 1))
            echo "1" > "$code_file"
        fi
    fi
}

should_run_provider() {
    local provider="$1"
    if [[ -z "$FILTER_PROVIDER" ]]; then
        return 0  # run all
    fi
    [[ "$provider" == "$FILTER_PROVIDER" ]]
}

has_key() {
    local provider="$1"
    case "$provider" in
        here)   [[ -n "$HERE_KEY" ]] ;;
        google) [[ -n "$GOOGLE_KEY" ]] ;;
        tomtom) [[ -n "$TOMTOM_KEY" ]] ;;
        mapbox) [[ -n "$MAPBOX_KEY" ]] ;;
        radar)  [[ -n "$RADAR_KEY" ]] ;;
        *)      return 1 ;;
    esac
}

key_for() {
    local provider="$1"
    case "$provider" in
        here)   echo "$HERE_KEY" ;;
        google) echo "$GOOGLE_KEY" ;;
        tomtom) echo "$TOMTOM_KEY" ;;
        mapbox) echo "$MAPBOX_KEY" ;;
        radar)  echo "$RADAR_KEY" ;;
    esac
}

# For negative longitudes, use --lng=VALUE syntax to avoid CLI parsing issues
lng_flag() {
    local lng="$1"
    printf -- "--lng=%s" "$lng"
}

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

printf "${CYAN}═══ EveryMap CLI Smoke Tests ═══${NC}\n\n"

if [[ "$NO_BUILD" == false ]]; then
    printf "Building CLI binary... "
    cargo build -p everymap-cli 2>&1 | tail -1
    if [[ ! -f "$CLI_BIN" ]]; then
        echo "ERROR: Binary not found at $CLI_BIN"
        exit 1
    fi
    echo ""
fi

mkdir -p "$OUTPUT_DIR"
echo "Output directory: $OUTPUT_DIR"
echo ""

# ---------------------------------------------------------------------------
# Test suites
# ---------------------------------------------------------------------------

# ===================== HERE ================================================

test_here() {
    printf "\n${YELLOW}─── HERE Provider ───${NC}\n"
    local key="$HERE_KEY"

    local loc_count=0
    for loc in "${LOCATIONS[@]}"; do
        loc_count=$((loc_count + 1))
        # Quick mode: only first 3 locations
        if [[ "$QUICK_MODE" == true ]] && [[ "$loc_count" -gt 3 ]]; then
            break
        fi

        local name query lat lng
        name="$(loc_name "$loc")"
        query="$(loc_query "$loc")"
        lat="$(loc_lat "$loc")"
        lng="$(loc_lng "$loc")"

        # geocode
        run_cli "here_geocode_${name}" \
            --api-key "$key" --provider here geocode "$query"

        # geocode with pretty output
        if [[ "$QUICK_MODE" == false ]] || [[ "$loc_count" -eq 1 ]]; then
            run_cli "here_geocode_${name}_pretty" \
                --api-key "$key" --provider here --output pretty geocode "$query"
        fi

        # geocode with summary output
        if [[ "$QUICK_MODE" == false ]] || [[ "$loc_count" -eq 1 ]]; then
            run_cli "here_geocode_${name}_summary" \
                --api-key "$key" --provider here --output summary geocode "$query"
        fi

        # reverse-geocode
        run_cli "here_reverse_${name}" \
            --api-key "$key" --provider here reverse-geocode --lat "$lat" $(lng_flag "$lng")

        # reverse-geocode summary
        if [[ "$QUICK_MODE" == false ]] || [[ "$loc_count" -eq 1 ]]; then
            run_cli "here_reverse_${name}_summary" \
                --api-key "$key" --provider here --output summary reverse-geocode --lat "$lat" $(lng_flag "$lng")
        fi

        # New CLI flags: --limit, --language on geocode; --radius on reverse
        if [[ "$QUICK_MODE" == false ]] || [[ "$loc_count" -eq 1 ]]; then
            run_cli "here_geocode_${name}_limit" \
                --api-key "$key" --provider here geocode --limit 3 "$query"

            run_cli "here_geocode_${name}_lang" \
                --api-key "$key" --provider here geocode --language de "$query"

            run_cli "here_reverse_${name}_radius" \
                --api-key "$key" --provider here reverse-geocode --lat "$lat" $(lng_flag "$lng") --radius 500
        fi

        # traffic
        run_cli "here_traffic_${name}" \
            --api-key "$key" --provider here traffic --lat "$lat" $(lng_flag "$lng")

        # traffic summary
        if [[ "$QUICK_MODE" == false ]] || [[ "$loc_count" -eq 1 ]]; then
            run_cli "here_traffic_${name}_summary" \
                --api-key "$key" --provider here --output summary traffic --lat "$lat" $(lng_flag "$lng")
        fi

        # isoline
        run_cli "here_isoline_${name}" \
            --api-key "$key" --provider here isoline --lat "$lat" $(lng_flag "$lng") --range 1000

        # isoline with larger range
        if [[ "$QUICK_MODE" == false ]] && [[ "$loc_count" -le 3 ]]; then
            run_cli "here_isoline_${name}_5000" \
                --api-key "$key" --provider here isoline --lat "$lat" $(lng_flag "$lng") --range 5000
        fi

        # isoline summary
        if [[ "$QUICK_MODE" == false ]] || [[ "$loc_count" -eq 1 ]]; then
            run_cli "here_isoline_${name}_summary" \
                --api-key "$key" --provider here --output summary isoline --lat "$lat" $(lng_flag "$lng") --range 1000
        fi

        # Isoline range-type=time
        if [[ "$QUICK_MODE" == false ]] || [[ "$loc_count" -eq 1 ]]; then
            run_cli "here_isoline_${name}_time" \
                --api-key "$key" --provider here isoline --lat "$lat" $(lng_flag "$lng") --range 600 --range-type time
        fi

        # map-image (saves to temp file)
        local img_file
        img_file="$(mktemp /tmp/everymap_smoke_here_${name}_XXXXX.png)"
        run_cli_binary "here_mapimage_${name}" "$img_file" \
            --api-key "$key" --provider here map-image --lat "$lat" $(lng_flag "$lng") --output-file "$img_file"
        rm -f "$img_file" 2>/dev/null
    done

    # Routes — test different transport modes
    local route_count=0
    for route in "${ROUTES[@]}"; do
        route_count=$((route_count + 1))
        if [[ "$QUICK_MODE" == true ]] && [[ "$route_count" -gt 2 ]]; then
            break
        fi

        local rname orig dest
        rname="$(route_name "$route")"
        orig="$(route_orig "$route")"
        dest="$(route_dest "$route")"

        # Default (car)
        run_cli "here_route_${rname}_car" \
            --api-key "$key" --provider here route --origin "$orig" --destination "$dest" --transport car

        # Pedestrian
        if [[ "$QUICK_MODE" == false ]] || [[ "$route_count" -eq 1 ]]; then
            run_cli "here_route_${rname}_pedestrian" \
                --api-key "$key" --provider here route --origin "$orig" --destination "$dest" --transport pedestrian
        fi

        # Truck
        if [[ "$QUICK_MODE" == false ]] && [[ "$route_count" -le 2 ]]; then
            run_cli "here_route_${rname}_truck" \
                --api-key "$key" --provider here route --origin "$orig" --destination "$dest" --transport truck
        fi

        # Bicycle, scooter, bus, taxi transport modes (HERE supports all)
        if [[ "$QUICK_MODE" == false ]] && [[ "$route_count" -le 2 ]]; then
            run_cli "here_route_${rname}_bicycle" \
                --api-key "$key" --provider here route --origin "$orig" --destination "$dest" --transport bicycle

            run_cli "here_route_${rname}_scooter" \
                --api-key "$key" --provider here route --origin "$orig" --destination "$dest" --transport scooter

            run_cli "here_route_${rname}_bus" \
                --api-key "$key" --provider here route --origin "$orig" --destination "$dest" --transport bus

            run_cli "here_route_${rname}_taxi" \
                --api-key "$key" --provider here route --origin "$orig" --destination "$dest" --transport taxi
        fi

        # Summary output
        run_cli "here_route_${rname}_summary" \
            --api-key "$key" --provider here --output summary route --origin "$orig" --destination "$dest" --transport car

        # New CLI flags: --avoid, --alternatives, --departure-time, --language on route
        if [[ "$QUICK_MODE" == false ]] && [[ "$route_count" -le 1 ]]; then
            run_cli "here_route_${rname}_avoid_tolls" \
                --api-key "$key" --provider here route --origin "$orig" --destination "$dest" --transport car --avoid tolls

            run_cli "here_route_${rname}_alternatives" \
                --api-key "$key" --provider here route --origin "$orig" --destination "$dest" --transport car --alternatives 2

            run_cli "here_route_${rname}_lang_de" \
                --api-key "$key" --provider here route --origin "$orig" --destination "$dest" --transport car --language de
        fi
    done

    # Match-route — test with different traces and transport modes
    for trace in "${TRACES[@]}"; do
        local tname tpoints
        tname="$(trace_name "$trace")"
        tpoints="$(trace_points "$trace")"

        # Default (car)
        run_cli "here_match_${tname}_car" \
            --api-key "$key" --provider here match-route --trace "$tpoints" --transport car

        # Pedestrian
        if [[ "$QUICK_MODE" == false ]]; then
            run_cli "here_match_${tname}_pedestrian" \
                --api-key "$key" --provider here match-route --trace "$tpoints" --transport pedestrian
        fi

        # Summary
        run_cli "here_match_${tname}_summary" \
            --api-key "$key" --provider here --output summary match-route --trace "$tpoints" --transport car
    done

    # Tour — multiple stops
    local tour_count=0
    for loc in "${LOCATIONS[@]}"; do
        tour_count=$((tour_count + 1))
        if [[ "$tour_count" -gt 3 ]]; then break; fi  # Only first 3

        local name lat lng
        name="$(loc_name "$loc")"
        lat="$(loc_lat "$loc")"
        lng="$(loc_lng "$loc")"

        # 3-stop tour near each city
        local stop1="${lat},$(echo "$lng + 0.01" | bc)"
        local stop2="${lat},$(echo "$lng + 0.02" | bc)"
        local stop3="${lat},$(echo "$lng + 0.03" | bc)"

        run_cli "here_tour_${name}" \
            --api-key "$key" --provider here tour --stops "$stop1" "$stop2" "$stop3"

        # Tour summary
        run_cli "here_tour_${name}_summary" \
            --api-key "$key" --provider here --output summary tour --stops "$stop1" "$stop2" "$stop3"
    done

    # Position
    run_cli "here_position" \
        --api-key "$key" --provider here position

    run_cli "here_position_verbose" \
        --api-key "$key" --provider here --verbose position

    # Tiles (HERE tiling scheme)
    for tile in "${HERE_TILES[@]}"; do
        local tname tz tx ty
        tname="$(echo "$tile" | cut -d'|' -f1)"
        tz="$(echo "$tile" | cut -d'|' -f2)"
        tx="$(echo "$tile" | cut -d'|' -f3)"
        ty="$(echo "$tile" | cut -d'|' -f4)"

        local tile_file
        tile_file="$(mktemp /tmp/everymap_smoke_here_tile_${tname}_XXXXX.omv)"

        run_cli_binary "here_tile_${tname}_base" "$tile_file" \
            --api-key "$key" --provider here tile --z "$tz" --x "$tx" --y "$ty" --layer base --output-file "$tile_file"
        rm -f "$tile_file" 2>/dev/null

        # Core layer
        if [[ "$QUICK_MODE" == false ]]; then
            tile_file="$(mktemp /tmp/everymap_smoke_here_tile_${tname}_core_XXXXX.omv)"
            run_cli_binary "here_tile_${tname}_core" "$tile_file" \
                --api-key "$key" --provider here tile --z "$tz" --x "$tx" --y "$ty" --layer core --output-file "$tile_file"
            rm -f "$tile_file" 2>/dev/null
        fi
    done

    # Attributes — various bboxes and layers
    for bbox in "${BBOXES[@]}"; do
        local bname bcoords
        bname="$(echo "$bbox" | cut -d'|' -f1)"
        bcoords="$(echo "$bbox" | cut -d'|' -f2)"

        run_cli "here_attrs_${bname}_roads" \
            --api-key "$key" --provider here attributes --bbox "$bcoords" --layer roads

        if [[ "$QUICK_MODE" == false ]]; then
            run_cli "here_attrs_${bname}_segments" \
                --api-key "$key" --provider here attributes --bbox "$bcoords" --layer segments
        fi
    done

    # Verbose mode on a few commands
    run_cli "here_geocode_berlin_verbose" \
        --api-key "$key" --provider here --verbose geocode "Berlin"

    run_cli "here_route_berlin_paris_verbose" \
        --api-key "$key" --provider here -v route --origin "52.52,13.40" --destination "48.86,2.35" --transport car

    run_cli "here_traffic_berlin_verbose" \
        --api-key "$key" --provider here --verbose traffic --lat 52.52 --lng=13.40

    run_cli "here_isoline_berlin_verbose" \
        --api-key "$key" --provider here --verbose isoline --lat 52.52 --lng=13.40 --range 1000

    run_cli "here_match_berlin_verbose" \
        --api-key "$key" --provider here --verbose match-route --trace "52.5164,13.3777;52.5170,13.3900;52.5175,13.3950;52.5180,13.4000" --transport car

    run_cli "here_tour_berlin_verbose" \
        --api-key "$key" --provider here --verbose tour --stops "52.52,13.41" "52.52,13.42" "52.52,13.43"

    run_cli "here_attrs_berlin_verbose" \
        --api-key "$key" --provider here --verbose attributes --bbox "52.4,13.2;52.6,13.5" --layer roads

}

# ===================== GOOGLE ==============================================

test_google() {
    printf "\n${YELLOW}─── Google Provider ───${NC}\n"
    local key="$GOOGLE_KEY"

    local loc_count=0
    for loc in "${LOCATIONS[@]}"; do
        loc_count=$((loc_count + 1))
        if [[ "$QUICK_MODE" == true ]] && [[ "$loc_count" -gt 3 ]]; then break; fi

        local name query lat lng
        name="$(loc_name "$loc")"
        query="$(loc_query "$loc")"
        lat="$(loc_lat "$loc")"
        lng="$(loc_lng "$loc")"

        run_cli "google_geocode_${name}" \
            --api-key "$key" --provider google geocode "$query"

        run_cli "google_geocode_${name}_summary" \
            --api-key "$key" --provider google --output summary geocode "$query"

        run_cli "google_reverse_${name}" \
            --api-key "$key" --provider google reverse-geocode --lat "$lat" $(lng_flag "$lng")

        run_cli "google_reverse_${name}_summary" \
            --api-key "$key" --provider google --output summary reverse-geocode --lat "$lat" $(lng_flag "$lng")

        # map-image
        local img_file
        img_file="$(mktemp /tmp/everymap_smoke_google_${name}_XXXXX.png)"
        run_cli_binary "google_mapimage_${name}" "$img_file" \
            --api-key "$key" --provider google map-image --lat "$lat" $(lng_flag "$lng") --output-file "$img_file"
        rm -f "$img_file" 2>/dev/null
    done

    # Attributes (Google supports attributes)
    local attr_count=0
    for bbox in "${BBOXES[@]}"; do
        attr_count=$((attr_count + 1))
        if [[ "$QUICK_MODE" == true ]] && [[ "$attr_count" -gt 1 ]]; then break; fi

        local aname acoords
        aname="$(echo "$bbox" | cut -d'|' -f1)"
        acoords="$(echo "$bbox" | cut -d'|' -f2)"

        run_cli "google_attrs_${aname}_roads" \
            --api-key "$key" --provider google attributes --bbox "$acoords" --layer roads

        if [[ "$QUICK_MODE" == false ]]; then
            run_cli "google_attrs_${aname}_segments" \
                --api-key "$key" --provider google attributes --bbox "$acoords" --layer segments
        fi
    done

    # Routes
    local route_count=0
    for route in "${ROUTES[@]}"; do
        route_count=$((route_count + 1))
        if [[ "$QUICK_MODE" == true ]] && [[ "$route_count" -gt 2 ]]; then break; fi

        local rname orig dest
        rname="$(route_name "$route")"
        orig="$(route_orig "$route")"
        dest="$(route_dest "$route")"

        run_cli "google_route_${rname}_car" \
            --api-key "$key" --provider google route --origin "$orig" --destination "$dest" --transport car

        run_cli "google_route_${rname}_summary" \
            --api-key "$key" --provider google --output summary route --origin "$orig" --destination "$dest" --transport car

        if [[ "$QUICK_MODE" == false ]] || [[ "$route_count" -eq 1 ]]; then
            run_cli "google_route_${rname}_pedestrian" \
                --api-key "$key" --provider google route --origin "$orig" --destination "$dest" --transport pedestrian
        fi

        # Bicycle and bus transport modes
        if [[ "$QUICK_MODE" == false ]] && [[ "$route_count" -le 2 ]]; then
            run_cli "google_route_${rname}_bicycle" \
                --api-key "$key" --provider google route --origin "$orig" --destination "$dest" --transport bicycle

            run_cli "google_route_${rname}_bus" \
                --api-key "$key" --provider google route --origin "$orig" --destination "$dest" --transport bus
        fi
    done

    # Match-route
    for trace in "${TRACES[@]}"; do
        local tname tpoints
        tname="$(trace_name "$trace")"
        tpoints="$(trace_points "$trace")"

        run_cli "google_match_${tname}_car" \
            --api-key "$key" --provider google match-route --trace "$tpoints" --transport car
    done

    # Position
    run_cli "google_position" \
        --api-key "$key" --provider google position

    run_cli "google_position_verbose" \
        --api-key "$key" --provider google --verbose position

    # Verbose mode
    run_cli "google_geocode_berlin_verbose" \
        --api-key "$key" --provider google --verbose geocode "Berlin"

    # Unsupported domains
    run_cli "google_traffic_unsupported" \
        --api-key "$key" --provider google traffic --lat 52.52 --lng=13.40

    run_cli "google_isoline_unsupported" \
        --api-key "$key" --provider google isoline --lat 52.52 --lng=13.40 --range 1000

    run_cli "google_tour_unsupported" \
        --api-key "$key" --provider google tour --stops "52.52,13.40" "52.52,13.41" "52.52,13.42"

    run_cli "google_tile_unsupported" \
        --api-key "$key" --provider google tile --z 14 --x 8800 --y 5374

    # New CLI flags: --limit, --language on geocode
    run_cli "google_geocode_berlin_limit" \
        --api-key "$key" --provider google geocode --limit 3 "Berlin"

    run_cli "google_geocode_berlin_lang" \
        --api-key "$key" --provider google geocode --language de "Berlin"
}

# ===================== TOMTOM ==============================================

test_tomtom() {
    printf "\n${YELLOW}─── TomTom Provider ───${NC}\n"
    local key="$TOMTOM_KEY"

    local loc_count=0
    for loc in "${LOCATIONS[@]}"; do
        loc_count=$((loc_count + 1))
        if [[ "$QUICK_MODE" == true ]] && [[ "$loc_count" -gt 3 ]]; then break; fi

        local name query lat lng
        name="$(loc_name "$loc")"
        query="$(loc_query "$loc")"
        lat="$(loc_lat "$loc")"
        lng="$(loc_lng "$loc")"

        run_cli "tomtom_geocode_${name}" \
            --api-key "$key" --provider tomtom geocode "$query"

        run_cli "tomtom_geocode_${name}_summary" \
            --api-key "$key" --provider tomtom --output summary geocode "$query"

        run_cli "tomtom_reverse_${name}" \
            --api-key "$key" --provider tomtom reverse-geocode --lat "$lat" $(lng_flag "$lng")

        run_cli "tomtom_reverse_${name}_summary" \
            --api-key "$key" --provider tomtom --output summary reverse-geocode --lat "$lat" $(lng_flag "$lng")

        run_cli "tomtom_traffic_${name}" \
            --api-key "$key" --provider tomtom traffic --lat "$lat" $(lng_flag "$lng")

        run_cli "tomtom_traffic_${name}_summary" \
            --api-key "$key" --provider tomtom --output summary traffic --lat "$lat" $(lng_flag "$lng")

        # Isoline
        run_cli "tomtom_isoline_${name}" \
            --api-key "$key" --provider tomtom isoline --lat "$lat" $(lng_flag "$lng") --range 1000

        if [[ "$QUICK_MODE" == false ]] || [[ "$loc_count" -eq 1 ]]; then
            run_cli "tomtom_isoline_${name}_summary" \
                --api-key "$key" --provider tomtom --output summary isoline --lat "$lat" $(lng_flag "$lng") --range 1000
        fi

        # Isoline range-type=time (TomTom supports time-based isolines)
        if [[ "$QUICK_MODE" == false ]] || [[ "$loc_count" -eq 1 ]]; then
            run_cli "tomtom_isoline_${name}_time" \
                --api-key "$key" --provider tomtom isoline --lat "$lat" $(lng_flag "$lng") --range 600 --range-type time
        fi

        # map-image
        local img_file
        img_file="$(mktemp /tmp/everymap_smoke_tomtom_${name}_XXXXX.png)"
        run_cli_binary "tomtom_mapimage_${name}" "$img_file" \
            --api-key "$key" --provider tomtom map-image --lat "$lat" $(lng_flag "$lng") --output-file "$img_file"
        rm -f "$img_file" 2>/dev/null
    done

    # Routes
    local route_count=0
    for route in "${ROUTES[@]}"; do
        route_count=$((route_count + 1))
        if [[ "$QUICK_MODE" == true ]] && [[ "$route_count" -gt 2 ]]; then break; fi

        local rname orig dest
        rname="$(route_name "$route")"
        orig="$(route_orig "$route")"
        dest="$(route_dest "$route")"

        run_cli "tomtom_route_${rname}_car" \
            --api-key "$key" --provider tomtom route --origin "$orig" --destination "$dest" --transport car

        run_cli "tomtom_route_${rname}_summary" \
            --api-key "$key" --provider tomtom --output summary route --origin "$orig" --destination "$dest" --transport car

        if [[ "$QUICK_MODE" == false ]] || [[ "$route_count" -eq 1 ]]; then
            run_cli "tomtom_route_${rname}_pedestrian" \
                --api-key "$key" --provider tomtom route --origin "$orig" --destination "$dest" --transport pedestrian
        fi

        # Bicycle, bus, taxi transport modes (TomTom supports all)
        if [[ "$QUICK_MODE" == false ]] && [[ "$route_count" -le 2 ]]; then
            run_cli "tomtom_route_${rname}_bicycle" \
                --api-key "$key" --provider tomtom route --origin "$orig" --destination "$dest" --transport bicycle

            run_cli "tomtom_route_${rname}_bus" \
                --api-key "$key" --provider tomtom route --origin "$orig" --destination "$dest" --transport bus

            run_cli "tomtom_route_${rname}_taxi" \
                --api-key "$key" --provider tomtom route --origin "$orig" --destination "$dest" --transport taxi
        fi
    done

    # Match-route (TomTom uses lon,lat;lon,lat format — but CLI should handle lat,lng input)
    for trace in "${TRACES[@]}"; do
        local tname tpoints
        tname="$(trace_name "$trace")"
        tpoints="$(trace_points "$trace")"

        run_cli "tomtom_match_${tname}_car" \
            --api-key "$key" --provider tomtom match-route --trace "$tpoints" --transport car
    done

    # Tour
    local tour_count=0
    for loc in "${LOCATIONS[@]}"; do
        tour_count=$((tour_count + 1))
        if [[ "$tour_count" -gt 2 ]]; then break; fi

        local name lat lng
        name="$(loc_name "$loc")"
        lat="$(loc_lat "$loc")"
        lng="$(loc_lng "$loc")"

        local stop1="${lat},$(echo "$lng + 0.01" | bc)"
        local stop2="${lat},$(echo "$lng + 0.02" | bc)"
        local stop3="${lat},$(echo "$lng + 0.03" | bc)"

        run_cli "tomtom_tour_${name}" \
            --api-key "$key" --provider tomtom tour --stops "$stop1" "$stop2" "$stop3"
    done

    # Tiles (web Mercator tiling scheme)
    for tile in "${WEB_TILES[@]}"; do
        local tname tz tx ty
        tname="$(echo "$tile" | cut -d'|' -f1)"
        tz="$(echo "$tile" | cut -d'|' -f2)"
        tx="$(echo "$tile" | cut -d'|' -f3)"
        ty="$(echo "$tile" | cut -d'|' -f4)"

        local tile_file
        tile_file="$(mktemp /tmp/everymap_smoke_tomtom_tile_${tname}_XXXXX.pbf)"

        run_cli_binary "tomtom_tile_${tname}" "$tile_file" \
            --api-key "$key" --provider tomtom tile --z "$tz" --x "$tx" --y "$ty" --output-file "$tile_file"
        rm -f "$tile_file" 2>/dev/null
    done

    # Verbose mode
    run_cli "tomtom_geocode_berlin_verbose" \
        --api-key "$key" --provider tomtom --verbose geocode "Berlin"

    run_cli "tomtom_traffic_berlin_verbose" \
        --api-key "$key" --provider tomtom --verbose traffic --lat 52.52 --lng=13.40

    run_cli "tomtom_isoline_berlin_verbose" \
        --api-key "$key" --provider tomtom --verbose isoline --lat 52.52 --lng=13.40 --range 1000

    run_cli "tomtom_tour_berlin_verbose" \
        --api-key "$key" --provider tomtom --verbose tour --stops "52.52,13.41" "52.52,13.42" "52.52,13.43"

    # Unsupported domains
    run_cli "tomtom_position_unsupported" \
        --api-key "$key" --provider tomtom position

    run_cli "tomtom_attributes_unsupported" \
        --api-key "$key" --provider tomtom attributes --bbox "52.4,13.2;52.6,13.5" --layer roads

}

# ===================== MAPBOX ===============================================

test_mapbox() {
    printf "\n${YELLOW}─── MapBox Provider ───${NC}\n"
    local key="$MAPBOX_KEY"

    local loc_count=0
    for loc in "${LOCATIONS[@]}"; do
        loc_count=$((loc_count + 1))
        if [[ "$QUICK_MODE" == true ]] && [[ "$loc_count" -gt 3 ]]; then break; fi

        local name query lat lng
        name="$(loc_name "$loc")"
        query="$(loc_query "$loc")"
        lat="$(loc_lat "$loc")"
        lng="$(loc_lng "$loc")"

        run_cli "mapbox_geocode_${name}" \
            --api-key "$key" --provider mapbox geocode "$query"

        run_cli "mapbox_geocode_${name}_summary" \
            --api-key "$key" --provider mapbox --output summary geocode "$query"

        run_cli "mapbox_reverse_${name}" \
            --api-key "$key" --provider mapbox reverse-geocode --lat "$lat" $(lng_flag "$lng")

        run_cli "mapbox_reverse_${name}_summary" \
            --api-key "$key" --provider mapbox --output summary reverse-geocode --lat "$lat" $(lng_flag "$lng")

        # Isoline (MapBox uses minutes for range)
        run_cli "mapbox_isoline_${name}" \
            --api-key "$key" --provider mapbox isoline --lat "$lat" $(lng_flag "$lng") --range 30

        # Isoline range-type=time
        if [[ "$QUICK_MODE" == false ]] || [[ "$loc_count" -eq 1 ]]; then
            run_cli "mapbox_isoline_${name}_time" \
                --api-key "$key" --provider mapbox isoline --lat "$lat" $(lng_flag "$lng") --range 30 --range-type time
        fi

        # map-image
        local img_file
        img_file="$(mktemp /tmp/everymap_smoke_mapbox_${name}_XXXXX.png)"
        run_cli_binary "mapbox_mapimage_${name}" "$img_file" \
            --api-key "$key" --provider mapbox map-image --lat "$lat" $(lng_flag "$lng") --output-file "$img_file"
        rm -f "$img_file" 2>/dev/null
    done

    # Routes
    local route_count=0
    for route in "${ROUTES[@]}"; do
        route_count=$((route_count + 1))
        if [[ "$QUICK_MODE" == true ]] && [[ "$route_count" -gt 2 ]]; then break; fi

        local rname orig dest
        rname="$(route_name "$route")"
        orig="$(route_orig "$route")"
        dest="$(route_dest "$route")"

        run_cli "mapbox_route_${rname}_car" \
            --api-key "$key" --provider mapbox route --origin "$orig" --destination "$dest" --transport car

        run_cli "mapbox_route_${rname}_summary" \
            --api-key "$key" --provider mapbox --output summary route --origin "$orig" --destination "$dest" --transport car

        if [[ "$QUICK_MODE" == false ]] || [[ "$route_count" -eq 1 ]]; then
            run_cli "mapbox_route_${rname}_pedestrian" \
                --api-key "$key" --provider mapbox route --origin "$orig" --destination "$dest" --transport pedestrian
        fi

        # Bicycle transport mode
        if [[ "$QUICK_MODE" == false ]] && [[ "$route_count" -le 2 ]]; then
            run_cli "mapbox_route_${rname}_bicycle" \
                --api-key "$key" --provider mapbox route --origin "$orig" --destination "$dest" --transport bicycle
        fi
    done

    # Match-route
    for trace in "${TRACES[@]}"; do
        local tname tpoints
        tname="$(trace_name "$trace")"
        tpoints="$(trace_points "$trace")"

        run_cli "mapbox_match_${tname}_car" \
            --api-key "$key" --provider mapbox match-route --trace "$tpoints" --transport car
    done

    # Tour — multiple variants
    local name="berlin"
    local lat="52.52"
    local lng="13.40"
    run_cli "mapbox_tour_${name}_car" \
        --api-key "$key" --provider mapbox tour --stops "52.52,13.41" "52.52,13.42" "52.52,13.43"

    run_cli "mapbox_tour_${name}_bicycle" \
        --api-key "$key" --provider mapbox tour --transport bicycle --stops "52.52,13.41" "52.52,13.42" "52.52,13.43"

    run_cli "mapbox_tour_${name}_walking" \
        --api-key "$key" --provider mapbox tour --transport pedestrian --stops "52.52,13.41" "52.52,13.42" "52.52,13.43"

    run_cli "mapbox_tour_${name}_summary" \
        --api-key "$key" --provider mapbox --output summary tour --stops "52.52,13.41" "52.52,13.42" "52.52,13.43"

    # Tiles (web Mercator tiling scheme)
    for tile in "${WEB_TILES[@]}"; do
        local tname tz tx ty
        tname="$(echo "$tile" | cut -d'|' -f1)"
        tz="$(echo "$tile" | cut -d'|' -f2)"
        tx="$(echo "$tile" | cut -d'|' -f3)"
        ty="$(echo "$tile" | cut -d'|' -f4)"

        local tile_file
        tile_file="$(mktemp /tmp/everymap_smoke_mapbox_tile_${tname}_XXXXX.pbf)"

        run_cli_binary "mapbox_tile_${tname}" "$tile_file" \
            --api-key "$key" --provider mapbox tile --z "$tz" --x "$tx" --y "$ty" --output-file "$tile_file"
        rm -f "$tile_file" 2>/dev/null
    done

    # Verbose mode
    run_cli "mapbox_geocode_berlin_verbose" \
        --api-key "$key" --provider mapbox --verbose geocode "Berlin"

    run_cli "mapbox_route_berlin_paris_verbose" \
        --api-key "$key" --provider mapbox --verbose route --origin "52.52,13.40" --destination "48.86,2.35" --transport car

    run_cli "mapbox_isoline_berlin_verbose" \
        --api-key "$key" --provider mapbox --verbose isoline --lat 52.52 --lng=13.40 --range 30

    run_cli "mapbox_match_berlin_verbose" \
        --api-key "$key" --provider mapbox --verbose match-route --trace "52.5164,13.3777;52.5170,13.3900;52.5175,13.3950;52.5180,13.4000" --transport car

    # Unsupported domains
    run_cli "mapbox_traffic_unsupported" \
        --api-key "$key" --provider mapbox traffic --lat 52.52 --lng=13.40

    run_cli "mapbox_position_unsupported" \
        --api-key "$key" --provider mapbox position

    run_cli "mapbox_attributes_unsupported" \
        --api-key "$key" --provider mapbox attributes --bbox "52.4,13.2;52.6,13.5" --layer roads

}

# ===================== RADAR ================================================

test_radar() {
    printf "\n${YELLOW}─── Radar Provider ───${NC}\n"
    local key="$RADAR_KEY"

    # Radar works best with US locations
    local radar_locations=(
        "nyc|New York|40.71|-74.01"
        "sf|San Francisco|37.77|-122.42"
        "boston|Boston|42.36|-71.06"
        "chicago|Chicago|41.88|-87.63"
        "la|Los Angeles|34.05|-118.24"
    )

    local loc_count=0
    for loc in "${radar_locations[@]}"; do
        loc_count=$((loc_count + 1))
        if [[ "$QUICK_MODE" == true ]] && [[ "$loc_count" -gt 2 ]]; then break; fi

        local name query lat lng
        name="$(loc_name "$loc")"
        query="$(loc_query "$loc")"
        lat="$(loc_lat "$loc")"
        lng="$(loc_lng "$loc")"

        run_cli "radar_geocode_${name}" \
            --api-key "$key" --provider radar geocode "$query"

        run_cli "radar_geocode_${name}_summary" \
            --api-key "$key" --provider radar --output summary geocode "$query"

        run_cli "radar_reverse_${name}" \
            --api-key "$key" --provider radar reverse-geocode --lat "$lat" $(lng_flag "$lng")

        run_cli "radar_reverse_${name}_summary" \
            --api-key "$key" --provider radar --output summary reverse-geocode --lat "$lat" $(lng_flag "$lng")

    done

    # Routes
    run_cli "radar_route_nyc_boston_car" \
        --api-key "$key" --provider radar route --origin "40.71,-74.01" --destination "42.36,-71.06" --transport car

    run_cli "radar_route_nyc_boston_summary" \
        --api-key "$key" --provider radar --output summary route --origin "40.71,-74.01" --destination "42.36,-71.06" --transport car

    run_cli "radar_route_nyc_boston_pedestrian" \
        --api-key "$key" --provider radar route --origin "40.71,-74.01" --destination "42.36,-71.06" --transport pedestrian

    run_cli "radar_route_sf_la_car" \
        --api-key "$key" --provider radar route --origin "37.77,-122.42" --destination "34.05,-118.24" --transport car

    # Additional route variants
    run_cli "radar_route_nyc_boston_bicycle" \
        --api-key "$key" --provider radar route --origin "40.71,-74.01" --destination "42.36,-71.06" --transport bicycle

    run_cli "radar_route_boston_chicago_car" \
        --api-key "$key" --provider radar route --origin "42.36,-71.06" --destination "41.88,-87.63" --transport car

    # Match-route
    run_cli "radar_match_nyc_car" \
        --api-key "$key" --provider radar match-route --trace "40.7128,-74.0060;40.7135,-74.0050;40.7142,-74.0040;40.7150,-74.0030" --transport car

    run_cli "radar_match_sf_car" \
        --api-key "$key" --provider radar match-route --trace "37.7749,-122.4194;37.7755,-122.4180;37.7760,-122.4165;37.7765,-122.4150" --transport car

    # Tour
    run_cli "radar_tour_nyc" \
        --api-key "$key" --provider radar tour --stops "40.71,-74.01" "40.75,-73.99" "40.78,-73.96"

    run_cli "radar_tour_nyc_summary" \
        --api-key "$key" --provider radar --output summary tour --stops "40.71,-74.01" "40.75,-73.99" "40.78,-73.96"

    run_cli "radar_tour_nyc_bicycle" \
        --api-key "$key" --provider radar tour --transport bicycle --stops "40.71,-74.01" "40.75,-73.99" "40.78,-73.96"

    run_cli "radar_tour_sf" \
        --api-key "$key" --provider radar tour --stops "37.77,-122.42" "37.78,-122.41" "37.79,-122.40"

    # Verbose mode
    run_cli "radar_geocode_nyc_verbose" \
        --api-key "$key" --provider radar --verbose geocode "New York"

    run_cli "radar_route_nyc_boston_verbose" \
        --api-key "$key" --provider radar --verbose route --origin "40.71,-74.01" --destination "42.36,-71.06" --transport car

    # Unsupported domains
    run_cli "radar_traffic_unsupported" \
        --api-key "$key" --provider radar traffic --lat 40.71 --lng=-74.01

    run_cli "radar_isoline_unsupported" \
        --api-key "$key" --provider radar isoline --lat 40.71 --lng=-74.01 --range 1000

    run_cli "radar_tile_unsupported" \
        --api-key "$key" --provider radar tile --z 14 --x 4651 --y 6159

    run_cli "radar_attributes_unsupported" \
        --api-key "$key" --provider radar attributes --bbox "40.6,-74.1;40.8,-74.0" --layer roads

    run_cli "radar_position_unsupported" \
        --api-key "$key" --provider radar position

    run_cli "radar_mapimage_unsupported" \
        --api-key "$key" --provider radar map-image --lat 40.71 --lng=-74.01
}

# ---------------------------------------------------------------------------
# Error path tests (no API key needed)
# ---------------------------------------------------------------------------

test_error_paths() {
    printf "\n${YELLOW}─── Error Path Tests ───${NC}\n"

    # Missing API key
    local old_home="$HOME"
    HOME="/tmp/everymap-smoke-no-config"
    run_cli "error_no_api_key" geocode "Berlin"
    HOME="$old_home"

    # Invalid provider
    run_cli "error_invalid_provider" \
        --provider invalid --api-key test geocode "Berlin"
}

# ---------------------------------------------------------------------------
# Run all test suites
# ---------------------------------------------------------------------------

printf "\n${CYAN}Running smoke tests...${NC}\n\n"

test_error_paths

if should_run_provider "here" && has_key "here"; then
    test_here
elif should_run_provider "here"; then
    printf "  ${YELLOW}SKIP${NC} HERE (no API key)\n"
fi

if should_run_provider "google" && has_key "google"; then
    test_google
elif should_run_provider "google"; then
    printf "  ${YELLOW}SKIP${NC} Google (no API key)\n"
fi

if should_run_provider "tomtom" && has_key "tomtom"; then
    test_tomtom
elif should_run_provider "tomtom"; then
    printf "  ${YELLOW}SKIP${NC} TomTom (no API key)\n"
fi

if should_run_provider "mapbox" && has_key "mapbox"; then
    test_mapbox
elif should_run_provider "mapbox"; then
    printf "  ${YELLOW}SKIP${NC} MapBox (no API key)\n"
fi

if should_run_provider "radar" && has_key "radar"; then
    test_radar
elif should_run_provider "radar"; then
    printf "  ${YELLOW}SKIP${NC} Radar (no API key)\n"
fi

# ---------------------------------------------------------------------------
# Generate summary report
# ---------------------------------------------------------------------------

generate_summary() {
    local summary_file="${OUTPUT_DIR}/SUMMARY.md"
    local total_calls="$TOTAL"
    local pass="$PASSED"
    local fail="$FAILED"

    cat > "$summary_file" <<EOF
# EveryMap CLI Smoke Test Report

**Date**: $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Binary**: \`$CLI_BIN\`
**Total commands**: $total_calls
**Passed**: $pass
**Failed**: $fail
**Pass rate**: $(echo "scale=1; $pass * 100 / $total_calls" | bc 2>/dev/null || echo "N/A")%

## Failed Commands

EOF

    # List all failures
    for code_file in "${OUTPUT_DIR}"/*.code; do
        local exit_code
        exit_code="$(cat "$code_file")"
        if [[ "$exit_code" -ne 0 ]]; then
            local base
            base="$(basename "$code_file" .code)"
            local stderr_preview
            stderr_preview="$(head -c 300 "${OUTPUT_DIR}/${base}.err" 2>/dev/null | tr '\n' ' ')"
            echo "- **${base}** (exit=${exit_code}): ${stderr_preview}" >> "$summary_file"
        fi
    done

    cat >> "$summary_file" <<EOF

## Timing Statistics

| Metric | Value |
|--------|-------|
| Fastest | $(find "${OUTPUT_DIR}" -name '*.time' -exec cat {} \; | sort -n | head -1)ms |
| Slowest | $(find "${OUTPUT_DIR}" -name '*.time' -exec cat {} \; | sort -n | tail -1)ms |
| Median  | $(find "${OUTPUT_DIR}" -name '*.time' -exec cat {} \; | sort -n | awk '{a[NR]=$0} END {if(NR%2==1) print a[(NR+1)/2]; else print (a[NR/2]+a[NR/2+1])/2}')ms |
| Total   | $(find "${OUTPUT_DIR}" -name '*.time' -exec cat {} \; | paste -sd+ | bc)ms |

## Provider Breakdown

EOF

    # Count per provider
    for provider in here google tomtom mapbox radar error; do
        local count
        count="$(find "${OUTPUT_DIR}" -name "${provider}_*" -name '*.code' 2>/dev/null | wc -l | tr -d ' ')"
        local provider_fail
        provider_fail="$(find "${OUTPUT_DIR}" -name "${provider}_*" -name '*.code' -exec cat {} \; 2>/dev/null | grep -cv '^0$' || echo 0)"
        if [[ "$count" -gt 0 ]]; then
            echo "| ${provider} | ${count} | ${provider_fail} |" >> "$summary_file"
        fi
    done

    echo "" >> "$summary_file"
    echo "Full outputs in: \`${OUTPUT_DIR}/\`" >> "$summary_file"
}

printf "\n\n${CYAN}═══ Generating Summary ═══${NC}\n\n"
generate_summary

# Print final results
printf "\n${CYAN}═══ Results ═══${NC}\n"
printf "  Total:  %d\n" "$TOTAL"
printf "  ${GREEN}Passed: %d${NC}\n" "$PASSED"
printf "  ${RED}Failed: %d${NC}\n" "$FAILED"
printf "\n  Report: %s/SUMMARY.md\n" "$OUTPUT_DIR"
printf "  Outputs: %s/\n\n" "$OUTPUT_DIR"

# Exit with failure if any test failed
if [[ "$FAILED" -gt 0 ]]; then
    exit 1
fi
exit 0