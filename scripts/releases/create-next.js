#!/usr/bin/env node

import github from '@actions/github';
import semver from 'semver';
import { writeFileSync, readFileSync } from 'fs';

const RELEASE_BRANCH_REGEX = /^release-(\d+\.\d+)$/;

const octokit = github.getOctokit(process.env.GITHUB_TOKEN);
const context = github.context;

const BRANCH_NAME = (context.ref || '').replace('refs/heads/', '');
const [OWNER, REPO] = (context.payload.repository.full_name || '').split('/');
const COMMIT_SHA = context.sha;

const panic = (...messages) => {
    console.error(...messages);
    process.exit(1);
}

if (!BRANCH_NAME) {
    panic('Unable to get branch');
}

if (!OWNER || !REPO) {
    panic('Unable to get repo');
}

console.log('Detected repo:', `${OWNER}/${REPO}`);
console.log('Detected branch name:', BRANCH_NAME);

const body = readFileSync('../../RELEASE_NOTES.md', { encoding: 'utf-8' });

const match = BRANCH_NAME.match(RELEASE_BRANCH_REGEX);

if (!match) {
    panic('Not release branch');
}

const [, branchTag] = match;

if (!semver.valid(branchTag + '.0')) {
    panic('Unable to parse semver from branch');
}

const getReleases = async () => {
    let releases = [];

    let len, page = 1

    do {
        try {
            const { data, status } = await octokit.rest.repos.listReleases({
                owner: OWNER,
                repo: REPO,
                per_page: 100,
                page
            });

            if (status !== 200) {
                throw new Error('http error');
            }

            len = data.length;
            releases = releases.concat(data);
        } catch (error) {
            panic('Unable to fetch releases', error);
        }

        page += 1;
    } while (len > 0);

    return releases.sort((a, b) => semver.compare(b.tag_name, a.tag_name));
}

const releases = await getReleases();

console.log('Fetched releases:', releases.length);

const latestRelease = releases[0];
const latestReleaseByBranch = releases.find(release => release.tag_name.startsWith(branchTag));

let tag;

if (latestReleaseByBranch) {
    // making new patch

    const parsed = semver.parse(latestReleaseByBranch.tag_name)

    if (!parsed) {
        panic('Unable to parse semver from API');
    }

    tag = parsed.inc('patch').version;
} else {
    // making new major/minor version
    tag = semver.parse(branchTag + '.0').version;
}

console.log('tag:', tag);

const {
    data: { upload_url: uploadUrl }
} = await octokit.rest.repos.createRelease({
    make_latest: latestRelease ? semver.gt(tag, latestRelease.tag_name) : true,
    owner: OWNER,
    repo: REPO,
    tag_name: tag,
    name: `Release ${tag}`,
    body: body,
    draft: false,
    prerelease: false,
    target_commitish: COMMIT_SHA
})

writeFileSync('./out.txt', `tag=${tag}\nupload-url=${uploadUrl}`);