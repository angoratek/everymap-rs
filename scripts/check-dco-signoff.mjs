#!/usr/bin/env node
// check-dco-signoff.mjs
//
// A pull request carries the Developer Certificate of Origin: every commit
// adds a `Signed-off-by:` line that matches the commit's author. Commits the
// maintainer writes directly on `main` carry no trailer.
//
// Usage: node scripts/check-dco-signoff.mjs --base origin/main
//
// The range is `<base>..HEAD`; merge commits are skipped because GitHub writes
// their message and no author signs them, and bot commits (Dependabot) are
// skipped because a machine cannot certify the DCO.

import { execFileSync } from "node:child_process";

const FIELD_SEPARATOR = "\x1f";
const RECORD_SEPARATOR = "\x1e";
const SIGN_OFF_PATTERN = /^signed-off-by:\s*(.+?)\s*<([^<>]+)>\s*$/i;

/**
 * Read the `--base` argument, or explain the usage and exit.
 * @param {string[]} argv
 * @returns {string}
 */
function parseBaseArgument(argv) {
    const baseIndex = argv.indexOf("--base");
    const base = baseIndex === -1 ? undefined : argv[baseIndex + 1];
    if (!base) {
        console.error("usage: node scripts/check-dco-signoff.mjs --base <git-ref>");
        process.exit(2);
    }
    return base;
}

/**
 * Read the commits in `<base>..HEAD` as one record per commit.
 * @param {string} base
 * @returns {{sha: string, authorName: string, authorEmail: string, message: string}[]}
 */
function readCommits(base) {
    const output = execFileSync(
        "git",
        [
            "log",
            "--no-merges",
            `--format=%H${FIELD_SEPARATOR}%an${FIELD_SEPARATOR}%ae${FIELD_SEPARATOR}%B${RECORD_SEPARATOR}`,
            `${base}..HEAD`,
        ],
        { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
    );

    return output
        .split(RECORD_SEPARATOR)
        .map((record) => record.trim())
        .filter((record) => record.length > 0)
        .map((record) => {
            const [sha, authorName, authorEmail, message] = record.split(FIELD_SEPARATOR);
            return { sha, authorName, authorEmail, message };
        });
}

/**
 * Whether a machine wrote the commit, so the DCO does not apply.
 * @param {{authorName: string, authorEmail: string}} commit
 * @returns {boolean}
 */
function isBotAuthor(commit) {
    return commit.authorName.toLowerCase().includes("[bot]")
        || commit.authorEmail.toLowerCase().includes("[bot]");
}

/**
 * Whether the commit carries a sign-off that matches its author.
 * @param {{authorName: string, authorEmail: string, message: string}} commit
 * @returns {boolean}
 */
function isSignedOffByAuthor(commit) {
    const authorEmail = commit.authorEmail.trim().toLowerCase();
    const authorName = commit.authorName.trim().toLowerCase();

    return commit.message.split("\n").some((line) => {
        const match = SIGN_OFF_PATTERN.exec(line.trim());
        if (!match) {
            return false;
        }
        const signOffName = match[1].trim().toLowerCase();
        const signOffEmail = match[2].trim().toLowerCase();
        return signOffName === authorName && signOffEmail === authorEmail;
    });
}

const base = parseBaseArgument(process.argv.slice(2));

let commits;
try {
    commits = readCommits(base);
} catch (error) {
    console.error(`dco-signoff: cannot read ${base}..HEAD: ${error.stderr?.toString().trim() || error.message}`);
    process.exit(2);
}

if (commits.length === 0) {
    console.log(`dco-signoff: no commits in ${base}..HEAD`);
    process.exit(0);
}

const unsignedCommits = commits
    .filter((commit) => !isBotAuthor(commit))
    .filter((commit) => !isSignedOffByAuthor(commit));

if (unsignedCommits.length > 0) {
    console.error("dco-signoff: these pull-request commits carry no sign-off matching their author:");
    for (const commit of unsignedCommits) {
        console.error(`  ${commit.sha.slice(0, 12)}  ${commit.authorName} <${commit.authorEmail}>`);
    }
    console.error("");
    console.error("Amend each commit with `git commit --amend -s` (or rebase with `git rebase --signoff`).");
    process.exit(1);
}

const signedOffCount = commits.filter((commit) => isSignedOffByAuthor(commit)).length;
const botCount = commits.filter((commit) => isBotAuthor(commit)).length;
console.log(
    `dco-signoff: ${signedOffCount} signed commit(s), ${botCount} bot commit(s) skipped, ${commits.length} in ${base}..HEAD.`,
);
