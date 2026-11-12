#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const cargoPath = path.join(root, 'riftcore-native', 'Cargo.toml');
const cargo = fs.readFileSync(cargoPath, 'utf8');
const versionMatch = cargo.match(/(\[package\][\s\S]*?^version\s*=\s*")([^\"]+)(")/m);
if (!versionMatch) throw new Error('Could not find the package version in riftcore-native/Cargo.toml');
const oldVersion = versionMatch[2];
const target = (process.argv[2] || 'patch').replace(/^v/, '').toLowerCase();
const semver = /^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?$/;
const parts = oldVersion.match(semver);
if (!parts) throw new Error(`Current version is not semver: ${oldVersion}`);

let nextVersion;
if (target === 'patch') nextVersion = `${parts[1]}.${parts[2]}.${Number(parts[3]) + 1}`;
else if (target === 'minor') nextVersion = `${parts[1]}.${Number(parts[2]) + 1}.0`;
else if (target === 'major') nextVersion = `${Number(parts[1]) + 1}.0.0`;
else if (semver.test(target)) nextVersion = target;
else throw new Error(`Invalid version: ${target}`);

const nextCargo = cargo.replace(versionMatch[0], `${versionMatch[1]}${nextVersion}${versionMatch[3]}`);
fs.writeFileSync(cargoPath, nextCargo, 'utf8');

const buildPath = path.join(root, 'build_all.bat');
fs.writeFileSync(buildPath, fs.readFileSync(buildPath, 'utf8').replace(/set "APP_VERSION=[^"]+"/, `set "APP_VERSION=${nextVersion}"`), 'utf8');

const nsiPath = path.join(root, 'installer', 'installer.nsi');
fs.writeFileSync(nsiPath, fs.readFileSync(nsiPath, 'utf8').replace(/!define PRODUCT_VERSION "[^"]+"/, `!define PRODUCT_VERSION "${nextVersion}"`), 'utf8');

const notesPath = path.join(root, 'installer', 'UPDATE_NOTES.txt');
fs.writeFileSync(notesPath, fs.readFileSync(notesPath, 'utf8').replace(/RIFTCORE COMPANION - NATIVE RUST UI/, `RIFTCORE COMPANION - UPDATE v${nextVersion}`), 'utf8');

console.log(`RiftCore version bumped: ${oldVersion} -> ${nextVersion}`);
console.log('Updated Cargo.toml, build_all.bat, installer definition and update notes.');
