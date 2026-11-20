#!/usr/bin/env node
// browser-smoke.mjs
//
// Serves the wasm-bindgen harness page and its fixture responses from one
// origin, drives the page in headless Chromium through Playwright, and fails
// when a provider's request never reached the server or the browser clock is
// implausible.
//
// Usage: node scripts/browser-smoke.mjs [pkg-dir]
//
// Requires `playwright` and its Chromium build:
//   npm install playwright && npx playwright install chromium

import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const SCRIPT_DIRECTORY = dirname(fileURLToPath(import.meta.url));
const PACKAGE_DIRECTORY = resolve(
    process.argv[2] ?? join(SCRIPT_DIRECTORY, "..", "everymap-browser-smoke", "pkg"),
);

const HARNESS_HTML = `<!doctype html>
<meta charset="utf-8">
<title>EveryMap browser smoke</title>
<script type="module">
  import init, { run_smoke } from "/pkg/everymap_browser_smoke.js";
  try {
    await init();
    window.__smokeResult = JSON.parse(await run_smoke(location.origin));
  } catch (error) {
    window.__smokeResult = { error: String(error) };
  }
</script>
`;

const JSON_FIXTURES = {
    here: { items: [] },
    google: { results: [], status: "OK" },
    tomtom: { results: [] },
    mapbox: { features: [] },
    radar: { addresses: [], meta: { code: 200 } },
    mapquest: { info: { statuscode: 0, messages: [] }, results: [] },
    photon: { features: [] },
    osrm: { code: "Ok", routes: [] },
    valhalla: {
        trip: {
            legs: [],
            summary: { time: 0, length: 0 },
            status: 0,
            units: "kilometers",
            language: "en-US",
        },
        id: "browser-smoke",
    },
};

// A 1x1 transparent PNG.
const PNG_BYTES = Buffer.from(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==",
    "base64",
);

const request_counts = Object.fromEntries(
    [
        "here",
        "google",
        "tomtom",
        "mapbox",
        "radar",
        "mapquest",
        "photon",
        "osrm",
        "valhalla",
        "url-template-raster",
        "url-template-vector",
    ].map((key) => [key, 0]),
);

function fixture_key(pathname) {
    const match = /^\/fixtures\/([^/]+)/.exec(pathname);
    if (!match) {
        return undefined;
    }
    if (match[1] === "tiles") {
        return pathname.includes("/raster/")
            ? "url-template-raster"
            : "url-template-vector";
    }
    return match[1];
}

function fixture_response(pathname) {
    if (pathname.startsWith("/fixtures/tiles/raster/")) {
        return { body: PNG_BYTES, contentType: "image/png" };
    }
    if (pathname.startsWith("/fixtures/tiles/vector/")) {
        return { body: Buffer.from([0x1a, 0x00]), contentType: "application/octet-stream" };
    }
    const key = fixture_key(pathname);
    if (key && JSON_FIXTURES[key]) {
        return {
            body: Buffer.from(JSON.stringify(JSON_FIXTURES[key])),
            contentType: "application/json",
        };
    }
    return undefined;
}

async function serve_asset(pathname) {
    const name = pathname.replace(/^\/pkg\//, "");
    if (!/^[A-Za-z0-9_.-]+$/.test(name)) {
        return undefined;
    }
    try {
        return await readFile(join(PACKAGE_DIRECTORY, name));
    } catch {
        return undefined;
    }
}

const server = createServer(async (request, response) => {
    const url = new URL(request.url ?? "/", "http://127.0.0.1");
    const pathname = url.pathname;

    if (pathname === "/") {
        response.writeHead(200, { "content-type": "text/html; charset=utf-8" });
        response.end(HARNESS_HTML);
        return;
    }

    if (pathname === "/__requests") {
        response.writeHead(200, { "content-type": "application/json" });
        response.end(JSON.stringify(request_counts));
        return;
    }

    if (pathname.startsWith("/pkg/")) {
        const asset = await serve_asset(pathname);
        if (!asset) {
            response.writeHead(404).end("not found");
            return;
        }
        const contentType = pathname.endsWith(".wasm")
            ? "application/wasm"
            : "text/javascript; charset=utf-8";
        response.writeHead(200, { "content-type": contentType });
        response.end(asset);
        return;
    }

    const fixture = fixture_response(pathname);
    if (!fixture) {
        response.writeHead(404).end("not found");
        return;
    }

    const key = fixture_key(pathname);
    if (key && key in request_counts) {
        request_counts[key] += 1;
    }
    response.writeHead(200, {
        "content-type": fixture.contentType,
        "access-control-allow-origin": "*",
    });
    response.end(fixture.body);
});

const port = await new Promise((resolvePort) => {
    server.listen(0, "127.0.0.1", () => resolvePort(server.address().port));
});

const failures = [];
let browser;

try {
    browser = await chromium.launch();
    const page = await browser.newPage();
    page.on("pageerror", (error) => failures.push(`page error: ${error.message}`));

    await page.goto(`http://127.0.0.1:${port}/`);
    await page.waitForFunction(() => window.__smokeResult !== undefined, undefined, {
        timeout: 90_000,
    });
    const result = await page.evaluate(() => window.__smokeResult);
    const counts = await page.evaluate(async () => (await fetch("/__requests")).json());

    if (result.error) {
        failures.push(`the smoke module failed: ${result.error}`);
    } else {
        if (!result.clock_ok) {
            failures.push("the browser clock read is implausible");
        }
        for (const { key, outcome } of result.providers) {
            if (outcome === "transport_error") {
                failures.push(`${key}: the fetch transport failed`);
            }
        }
    }

    for (const [key, count] of Object.entries(counts)) {
        if (count < 1) {
            failures.push(`${key}: no request reached the fixture server`);
        }
    }
    console.log(`browser-smoke: ${Object.keys(counts).length} providers, ${JSON.stringify(counts)}`);
    if (!result.error) {
        console.log(`browser-smoke: outcomes ${JSON.stringify(result.providers)}`);
    }
} finally {
    await browser?.close();
    await new Promise((resolveClose) => server.close(resolveClose));
}

if (failures.length > 0) {
    console.error("browser-smoke: failures:");
    for (const failure of failures) {
        console.error(`  ${failure}`);
    }
    process.exit(1);
}

console.log("browser-smoke: every provider reached the fixture server through fetch.");
