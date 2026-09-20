# Official Relay v1 deployment

The owner guide is [Relay Server](../../src/apps/relay-server/README.md).
Use this independent Compose project for `/v/1.0.2/`. Keep the `/v/1.0.1/` Compose project
(`openbitfun-relay-v1.0.1`, port `19701`, `/srv/openbitfun-relay-v1.0.1`) running with its own
container, image, database, assets, and proxy location: clients built for 1.0.1 keep using it,
and the two releases do not share a credential database.

Deploy from a committed checkout at `/srv/openbitfun-relay-v1.0.2/app`. Set
`RELAY_GIT_COMMIT` to that checkout's verified full commit. Build mobile web
from the same checkout with `pnpm run build:mobile-web` and stage its `dist`
contents into `/srv/openbitfun-relay-v1.0.2/static`. Create `data` and `assets`
under that root owned by UID/GID 10001 before starting Compose.

The Linux host network plus explicit `127.0.0.1:19702` listener lets the service
verify the immediate proxy peer before trusting its overwritten forwarded IP.
Do not publish this listener on a public interface. Install `nginx-http.conf` as `/etc/nginx/conf.d/relay-v1.0.2.conf` and include
`nginx-location.conf` as `/etc/nginx/relay-v1.0.2-location.conf` in the existing remote server after the container passes
its health check. Keep the existing v1.0.1 includes. The new version uses independent admission zones.
The new location accepts the existing explicit WAF origin
ranges and loopback; direct origin requests from other peers receive 403.
Forwarded client IPs are recursively resolved only for those trusted WAF
peers. Keep the range list synchronized with the WAF control plane. Raise
`worker_connections` to 8192 and retain a file descriptor limit of at least
16384; validate with `nginx -t` before a graceful reload.

Published Pages use the existing official Relay address:
`https://remote.openbitfun.com/v/1.0.2/p/{github_username}/{slug}`.
Compose sets this public base URL and the separate sign-in base URL
`https://auth.openbitfun.com/v/1.0.2`. Users do not configure domains.
Install the versioned Pages sign-in locations from
[`nginx-auth.openbitfun.com.conf`](../miniapp-market/nginx-auth.openbitfun.com.conf)
in the existing auth server as well. Keep its marketplace sign-in routes intact.
These locations forward only Page sign-in and GitHub start/poll endpoints to
Relay, preserve the auth Host, and omit query strings from access logs.
The Page callback and published content remain on the remote origin.

Both base URLs are required: missing configuration returns an explicit 503.
After changing the environment, recreate only `relay-v1` with the verified
existing image (`docker compose up -d --no-build relay-v1`), validate Nginx,
and gracefully reload it. Verify publish and deploy through an authenticated
CLI, fetch both returned URLs, and verify that private-page sign-in redirects
to the auth origin and its client script loads.

Before replacement, back up this version's database and assets and retain the
previous image tag. Roll back only this Compose project and its versioned
location. Never use the legacy relay Compose file to operate this deployment.

## Retiring an older version

Retiring `/v/1.0.1/` (or any earlier prefix) must tell its clients to update
instead of leaving them with a bare `404`/`502`, because those clients cannot
be patched after the fact. Both forms answer the same body:

```json
{"error":"relay_version_retired","message":"…Update OpenBitFun on this device, then sign in again to continue.","update_required":true}
```

The code string, the `update_required` flag and the message are one contract,
pinned by `src/crates/services/relay-service/src/retired_version.rs`.

1. **Relay enforced (while a container still serves the prefix).** Point the
   retired prefix at the *current* relay and announce it:

   ```nginx
   location ^~ /v/1.0.0/ {
       proxy_set_header X-OpenBitFun-Relay-Served-Prefix /v/1.0.0;
       # …the same proxy_pass, real-IP and admission settings as the live prefix
   }
   ```

   Start that relay with `RELAY_RETIRED_VERSION_PREFIXES=/v/1.0.0` so account,
   realtime and Page routes of the announced prefix answer `410` before
   authentication or body buffering. `RELAY_RETIRED=1` retires the whole
   deployment instead, which is what a full shutdown uses. The startup log
   states the resolved switch (`relay answers retired versions …` or `no Relay
   version is retired`), so enabling and rolling back are verifiable before
   traffic arrives. The development deployment retires `/v/1.0.0/` this way,
   through a deployment-local `compose.override.yml`:

   ```yaml
   services:
     relay-v1:
       environment:
         RELAY_RETIRED_VERSION_PREFIXES: /v/1.0.0
   ```

   **Both halves are required.** Proxying a retired prefix without listing it
   serves the live relay under that retired prefix, which silently un-retires
   the version; announcing a prefix that the edge never sends retires nothing.
   A working example of this location is installed as
   `/etc/nginx/relay-retired-v1.0.0-location.conf`.
2. **Edge only (recommended once the container is gone).** Stop the retired
   Compose project, delete its images and volumes, then replace the retired
   prefix's include with `nginx-retired-version.conf`, keep the live version's
   include untouched, validate with `nginx -t`, and gracefully reload. The old
   prefix keeps answering `410` with `no-store`, so no client caches a stale
   success, and the retirement body covers the websocket upgrade path too.

Either form keeps `/health` and static content served: the page that explains
the update still loads, and the health probe keeps working.

Never retire a prefix that current clients still use: an unconfigured relay is
never retired, so an omitted or mistyped variable fails safe. An operator
turnover hazard is the reverse: a prefix listed but no longer proxied stays
retired only for as long as the edge announces it.
