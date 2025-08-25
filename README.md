# Songparser
A service that edits the metadata of a queued song and populates it with data.


## Getting started
This service can run as a regular service or a docker image. The easiest way to get quickly
started is to run it with docker. Copy `.env.docker.sample` as `.env`. Ensure that 
`ROOT_DIRECTORY` is pointing to a directory that exists, not for the local filesystem, but
for the filesystem on the docker image. The current directory that is listed will work, but
it can be changed.

The `SERVICE_PASSPHRASE` env variable should not be changed, but it could be changed. The 
value for this variable should match the value in the `passphrase` table. This would be
found in the `icarus_auth` project.

Ensure that the URLs for the two APIs are correctly set for the respective env variables
`ICARUS_BASE_API_URL` for Icarus API and `ICARUS_AUTH_BASE_API_URL` for `icarus_auth`.

If the values are properly set, next is to build the image. The docker image should be
built from the main icarus web API.
