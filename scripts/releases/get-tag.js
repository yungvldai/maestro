#!/usr/bin/env node

import github from '@actions/github';
import semver from 'semver';
import { writeFileSync } from 'fs';

const RELEASE_BRANCH_REGEX = /^release-(\d+\.\d+)$/;
const BRANCH_NAME = (process.env.GITHUB_REF || '').replace('refs/heads/', '');

console.log('Detected branch name:', BRANCH_NAME);

const octokit = github.getOctokit(process.env.GITHUB_TOKEN);
const context = github.context;

const panic = (message) => {
    console.error(message);
    process.exit(1);
}

if (!BRANCH_NAME) {
    panic('Unable to get branch');
}

const match = BRANCH_NAME.match(RELEASE_BRANCH_REGEX);

if (!match) {
    panic('Not release branch');
}

const [, versionString] = match;

const versionObject = semver.parse(versionString + '.0');

if (!versionObject) {
    panic('Unable to parse semver');
}

const getReleases = async () => {
    let releases = [];

    let items, page = 1

    do {
        try {
            const { data, status } = await octokit.rest.repos.listReleases({
                owner: context.owner,
                repo: context.repo,
                per_page: 100,
                page
            });

            if (status !== 200) {
                throw new Error('http error');
            }

            const items = data;
            releases = releases.concat(items);
        } catch (error) {
            panic('Unable to fetch releases');
        }

        page += 1;
    } while (items.length > 0);

    return releases.sort((a, b) => semver.compare(b.tag_name, a.tag_name)); // DESC
}

const releases = await getReleases();

console.log('Fetched releases:', releases.length);

const latest = releases.find(release => release.tag_name.startsWith(versionString));

if (latest) {
    versionObject.inc('patch');
}

writeFileSync('./tag.txt', `tag=${versionObject.version}`);