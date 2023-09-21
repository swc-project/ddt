#!/usr/bin/env bash
set -eu


cargo install --debug --path .


(cd lab/git-study; git merge --abort; git merge react)