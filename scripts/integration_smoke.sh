#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER_DIR="$ROOT/server"
DB_NAME="domus_smoke_$(date +%s)"
DB_USER="${DOMUS_SMOKE_DB_USER:-domus_smoke}"
DB_PASS="${DOMUS_SMOKE_DB_PASS:-domus_smoke_pass}"
DB_URL="postgres://${DB_USER}:${DB_PASS}@localhost:5432/${DB_NAME}"
MEDIA_DIR="/tmp/domus-smoke-media"
PORT="${DOMUS_SMOKE_PORT:-43003}"
OAUTH_PORT="${DOMUS_SMOKE_OAUTH_PORT:-43111}"
BASE="http://127.0.0.1:${PORT}/api"
SERVER_PID=""
OAUTH_PID=""

cleanup() {
  if [[ -n "${OAUTH_PID}" ]] && kill -0 "${OAUTH_PID}" 2>/dev/null; then
    kill "${OAUTH_PID}" || true
    wait "${OAUTH_PID}" || true
  fi
  if [[ -n "${SERVER_PID}" ]] && kill -0 "${SERVER_PID}" 2>/dev/null; then
    kill "${SERVER_PID}" || true
    wait "${SERVER_PID}" || true
  fi
  su - postgres -c "dropdb --if-exists ${DB_NAME}" >/dev/null
  rm -rf "${MEDIA_DIR}" /tmp/domus-smoke-*.png /tmp/domus-smoke-*.out
}
trap cleanup EXIT

su - postgres -c "psql -tAc \"SELECT 1 FROM pg_roles WHERE rolname='${DB_USER}'\"" | rg -q 1 \
  || su - postgres -c "psql -c \"CREATE ROLE ${DB_USER} LOGIN PASSWORD '${DB_PASS}'\"" >/dev/null
su - postgres -c "createdb -O ${DB_USER} ${DB_NAME}"
rm -rf "${MEDIA_DIR}"
mkdir -p "${MEDIA_DIR}"
magick -size 64x64 xc:'#3366cc' /tmp/domus-smoke-upload.png
magick -size 64x64 xc:'#cc6633' /tmp/domus-smoke-old.png
magick -size 64x64 xc:'#33aa66' /tmp/domus-smoke-live.jpg
magick -size 64x64 xc:'#6633aa' /tmp/domus-smoke-raw-source.png
magick -size 64x64 xc:'#226644' /tmp/domus-smoke-storage.png
cp /tmp/domus-smoke-old.png /tmp/domus-smoke-live.mov
cp /tmp/domus-smoke-raw-source.png /tmp/domus-smoke-raw.NEF

(
  cd "${SERVER_DIR}"
  DB_URL="${DB_URL}" HOST=127.0.0.1 PORT="${PORT}" DOMUS_MEDIA_LOCATION="${MEDIA_DIR}" cargo run -p domus-server
) >/tmp/domus-smoke-server.out 2>&1 &
SERVER_PID="$!"

for _ in $(seq 1 60); do
  if curl -fsS "${BASE}/server/ping" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
curl -fsS "${BASE}/server/ping" >/dev/null
python3 - "${PORT}" <<'PY'
import json
import socket
import sys

port = int(sys.argv[1])
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.settimeout(2)
sock.sendto(b"DOMUS_DISCOVER_V1", ("127.0.0.1", port))
data, _ = sock.recvfrom(2048)
body = json.loads(data.decode("utf-8"))
assert body["url"].endswith(f":{port}"), body
assert body["api"].endswith(f":{port}/api"), body
PY

python3 - "${OAUTH_PORT}" <<'PY' >/tmp/domus-smoke-oauth.out 2>&1 &
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

port = int(sys.argv[1])

class Handler(BaseHTTPRequestHandler):
    def _json(self, status, body):
        payload = json.dumps(body).encode("utf-8")
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def do_GET(self):
        if self.path.startswith("/userinfo"):
            self._json(200, {
                "sub": "oauth-sub-1",
                "email": "oauth@example.com",
                "name": "OAuth User",
            })
        else:
            self._json(404, {"error": "not_found"})

    def do_POST(self):
        if self.path.startswith("/token"):
            self._json(200, {
                "access_token": "mock-access",
                "token_type": "Bearer",
            })
        else:
            self._json(404, {"error": "not_found"})

    def log_message(self, *_):
        pass

ThreadingHTTPServer(("127.0.0.1", port), Handler).serve_forever()
PY
OAUTH_PID="$!"
for _ in $(seq 1 30); do
  if curl -fsS "http://127.0.0.1:${OAUTH_PORT}/userinfo" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
curl -fsS "http://127.0.0.1:${OAUTH_PORT}/userinfo" >/dev/null

EMAIL="smoke-$(date +%s)@example.com"
PASS='SmokePass123!'
curl -fsS -X POST "${BASE}/auth/admin-sign-up" \
  -H 'content-type: application/json' \
  -d "{\"email\":\"${EMAIL}\",\"password\":\"${PASS}\",\"name\":\"Smoke Admin\"}" >/dev/null
TOKEN="$(curl -fsS -X POST "${BASE}/auth/login" \
  -H 'content-type: application/json' \
  -d "{\"email\":\"${EMAIL}\",\"password\":\"${PASS}\"}" | jq -r '.accessToken')"

upload_asset() {
  local device_asset_id="$1"
  local file_path="${2:-/tmp/domus-smoke-upload.png}"
  local created_at="${3:-2026-07-14T01:00:00.000Z}"
  local live_photo_video_id="${4:-}"
  local curl_args=(
    -fsS -X POST "${BASE}/assets"
    -H "Authorization: Bearer ${TOKEN}" \
    -F "assetData=@${file_path}" \
    -F "deviceAssetId=${device_asset_id}" \
    -F deviceId=smoke-device \
    -F "fileCreatedAt=${created_at}" \
    -F "fileModifiedAt=${created_at}" \
    -F isFavorite=false
  )
  if [[ -n "${live_photo_video_id}" ]]; then
    curl_args+=(-F "livePhotoVideoId=${live_photo_video_id}")
  fi
  curl "${curl_args[@]}"
}

FIRST="$(upload_asset smoke-asset-1)"
ASSET_ID="$(jq -r '.id' <<<"${FIRST}")"
SECOND="$(upload_asset smoke-asset-2)"
jq -e --arg id "${ASSET_ID}" '.status == "duplicate" and .id == $id' <<<"${SECOND}" >/dev/null
OLD="$(upload_asset smoke-asset-old /tmp/domus-smoke-old.png 2025-07-14T01:00:00.000Z)"
OLD_ASSET_ID="$(jq -r '.id' <<<"${OLD}")"
RAW="$(upload_asset smoke-asset-raw /tmp/domus-smoke-raw.NEF 2026-07-14T02:00:00.000Z)"
RAW_ASSET_ID="$(jq -r '.id' <<<"${RAW}")"
curl -fsS "${BASE}/assets/${RAW_ASSET_ID}" \
  -H "Authorization: Bearer ${TOKEN}" | jq -e '.type == "IMAGE"' >/dev/null
LIVE_VIDEO="$(upload_asset smoke-live-video /tmp/domus-smoke-live.mov 2026-07-14T03:00:00.000Z)"
LIVE_VIDEO_ID="$(jq -r '.id' <<<"${LIVE_VIDEO}")"
LIVE_IMAGE="$(upload_asset smoke-live-image /tmp/domus-smoke-live.jpg 2026-07-14T03:00:00.000Z "${LIVE_VIDEO_ID}")"
LIVE_IMAGE_ID="$(jq -r '.id' <<<"${LIVE_IMAGE}")"
curl -fsS "${BASE}/assets/${LIVE_IMAGE_ID}" \
  -H "Authorization: Bearer ${TOKEN}" | jq -e --arg id "${LIVE_VIDEO_ID}" '.livePhotoVideoId == $id' >/dev/null
curl -fsS -X PUT "${BASE}/system-config" \
  -H "Authorization: Bearer ${TOKEN}" -H 'content-type: application/json' \
  -d '{"storageTemplate":{"enabled":true,"template":"camera/{{y}}/{{MM}}/{{filename}}"}}' >/dev/null
STORAGE_TEMPLATE_ASSET="$(upload_asset smoke-storage-template /tmp/domus-smoke-storage.png 2026-07-14T04:00:00.000Z)"
STORAGE_TEMPLATE_ASSET_ID="$(jq -r '.id' <<<"${STORAGE_TEMPLATE_ASSET}")"
curl -fsS "${BASE}/assets/${STORAGE_TEMPLATE_ASSET_ID}" \
  -H "Authorization: Bearer ${TOKEN}" | jq -e '.originalPath | contains("/library/") and contains("/camera/2026/07/")' >/dev/null
OAUTH_DISABLED_STATUS="$(curl -sS -o /tmp/domus-smoke-oauth-disabled.out -w '%{http_code}' -X POST "${BASE}/oauth/authorize" -H 'content-type: application/json' -d '{}')"
test "${OAUTH_DISABLED_STATUS}" = "400"
curl -fsS -X PUT "${BASE}/system-config" \
  -H "Authorization: Bearer ${TOKEN}" -H 'content-type: application/json' \
  -d "{\"storageTemplate\":{\"enabled\":true,\"template\":\"camera/{{y}}/{{MM}}/{{filename}}\"},\"oauth\":{\"enabled\":true,\"authorizeUrl\":\"http://127.0.0.1:${OAUTH_PORT}/authorize\",\"tokenEndpoint\":\"http://127.0.0.1:${OAUTH_PORT}/token\",\"userinfoEndpoint\":\"http://127.0.0.1:${OAUTH_PORT}/userinfo\",\"clientId\":\"domus\",\"clientSecret\":\"secret\",\"scope\":\"openid email profile\"}}" >/dev/null
curl -fsS -X POST "${BASE}/oauth/authorize" \
  -H 'content-type: application/json' \
  -d '{"redirectUri":"domus://oauth","state":"smoke"}' | jq -e '.url | contains("http://127.0.0.1:'"${OAUTH_PORT}"'/authorize") and contains("client_id=domus") and contains("redirect_uri=domus%3A%2F%2Foauth") and contains("scope=openid+email+profile")' >/dev/null
curl -fsS -X POST "${BASE}/oauth/callback" \
  -H 'content-type: application/json' \
  -d '{"code":"mock-code","redirectUri":"domus://oauth","state":"smoke"}' | jq -e '.accessToken and .userEmail == "oauth@example.com" and .isAdmin == false' >/dev/null

for _ in $(seq 1 30); do
  ASSET="$(curl -fsS "${BASE}/assets/${ASSET_ID}" -H "Authorization: Bearer ${TOKEN}")"
  [[ "$(jq -r '.thumbhash // empty' <<<"${ASSET}")" != "" ]] && break
  sleep 1
done
curl -fsS "${BASE}/assets/${ASSET_ID}/thumbnail" -H "Authorization: Bearer ${TOKEN}" -o /tmp/domus-smoke-thumb.out
test "$(wc -c </tmp/domus-smoke-thumb.out)" -gt 0
curl -fsS "${BASE}/assets/${ASSET_ID}/original" -H "Authorization: Bearer ${TOKEN}" -o /tmp/domus-smoke-original.out
test "$(wc -c </tmp/domus-smoke-original.out)" -eq "$(wc -c </tmp/domus-smoke-upload.png)"

UPDATED="$(curl -fsS -X PUT "${BASE}/assets/${ASSET_ID}" \
  -H "Authorization: Bearer ${TOKEN}" -H 'content-type: application/json' \
  -d '{"isFavorite":true,"isArchived":true}')"
jq -e '.isFavorite == true and .isArchived == true' <<<"${UPDATED}" >/dev/null

ALBUM="$(curl -fsS -X POST "${BASE}/albums" \
  -H "Authorization: Bearer ${TOKEN}" -H 'content-type: application/json' \
  -d "{\"albumName\":\"Smoke Album\",\"assetIds\":[\"${ASSET_ID}\"]}")"
ALBUM_ID="$(jq -r '.id' <<<"${ALBUM}")"
DETAIL="$(curl -fsS "${BASE}/albums/${ALBUM_ID}" -H "Authorization: Bearer ${TOKEN}")"
jq -e '.assetCount == 1 and (.assets | length) == 1' <<<"${DETAIL}" >/dev/null

USER_EMAIL="shared-$(date +%s)@example.com"
USER="$(curl -fsS -X POST "${BASE}/admin/users" \
  -H "Authorization: Bearer ${TOKEN}" -H 'content-type: application/json' \
  -d "{\"email\":\"${USER_EMAIL}\",\"password\":\"${PASS}\",\"name\":\"Shared User\"}")"
USER_ID="$(jq -r '.id' <<<"${USER}")"
curl -fsS -X PUT "${BASE}/albums/${ALBUM_ID}/users" \
  -H "Authorization: Bearer ${TOKEN}" -H 'content-type: application/json' \
  -d "[{\"userId\":\"${USER_ID}\",\"role\":\"editor\"}]" >/dev/null

API_KEY="$(curl -fsS -X POST "${BASE}/api-keys" \
  -H "Authorization: Bearer ${TOKEN}" -H 'content-type: application/json' \
  -d '{"name":"Smoke Key"}')"
SECRET="$(jq -r '.secret' <<<"${API_KEY}")"
curl -fsS "${BASE}/users/me" -H "x-api-key: ${SECRET}" | jq -e --arg email "${EMAIL}" '.email == $email' >/dev/null

TAG="$(curl -fsS -X POST "${BASE}/tags" \
  -H "Authorization: Bearer ${TOKEN}" -H 'content-type: application/json' \
  -d '{"name":"SmokeTag"}')"
TAG_ID="$(jq -r '.id' <<<"${TAG}")"
curl -fsS -X PUT "${BASE}/tags/${TAG_ID}/assets" \
  -H "Authorization: Bearer ${TOKEN}" -H 'content-type: application/json' \
  -d "{\"assetIds\":[\"${ASSET_ID}\"]}" | jq -e '.count == 1' >/dev/null

STACK="$(curl -fsS -X POST "${BASE}/stacks" \
  -H "Authorization: Bearer ${TOKEN}" -H 'content-type: application/json' \
  -d "{\"assetIds\":[\"${ASSET_ID}\",\"${OLD_ASSET_ID}\"]}")"
STACK_ID="$(jq -r '.id' <<<"${STACK}")"
curl -fsS "${BASE}/stacks/${STACK_ID}" \
  -H "Authorization: Bearer ${TOKEN}" | jq -e '.assets | length == 2' >/dev/null

curl -fsS -X POST "${BASE}/partners" \
  -H "Authorization: Bearer ${TOKEN}" -H 'content-type: application/json' \
  -d "{\"sharedWithId\":\"${USER_ID}\"}" | jq -e --arg id "${USER_ID}" '.id == $id and .inTimeline == true' >/dev/null
curl -fsS "${BASE}/partners?direction=shared-by" \
  -H "Authorization: Bearer ${TOKEN}" | jq -e 'length >= 1' >/dev/null

SHARE="$(curl -fsS -X POST "${BASE}/shared-links" \
  -H "Authorization: Bearer ${TOKEN}" -H 'content-type: application/json' \
  -d "{\"type\":\"INDIVIDUAL\",\"assetIds\":[\"${ASSET_ID}\"],\"allowDownload\":true}")"
SHARE_KEY="$(jq -r '.key' <<<"${SHARE}")"
curl -fsS --get "${BASE}/shared-links/me" \
  --data-urlencode "key=${SHARE_KEY}" | jq -e '.assets | length == 1' >/dev/null
curl -fsS --get "${BASE}/assets/${ASSET_ID}/original" \
  --data-urlencode "key=${SHARE_KEY}" -o /tmp/domus-smoke-share.out
test "$(wc -c </tmp/domus-smoke-share.out)" -eq "$(wc -c </tmp/domus-smoke-upload.png)"
READ_ONLY_SHARE="$(curl -fsS -X POST "${BASE}/shared-links" \
  -H "Authorization: Bearer ${TOKEN}" -H 'content-type: application/json' \
  -d "{\"type\":\"INDIVIDUAL\",\"assetIds\":[\"${OLD_ASSET_ID}\"],\"allowDownload\":false}")"
READ_ONLY_KEY="$(jq -r '.key' <<<"${READ_ONLY_SHARE}")"
curl -fsS --get "${BASE}/assets/${OLD_ASSET_ID}/thumbnail" \
  --data-urlencode "key=${READ_ONLY_KEY}" -o /tmp/domus-smoke-readonly-thumb.out
READ_ONLY_STATUS="$(curl -sS -o /tmp/domus-smoke-readonly-original.out -w '%{http_code}' --get "${BASE}/assets/${OLD_ASSET_ID}/original" --data-urlencode "key=${READ_ONLY_KEY}")"
test "${READ_ONLY_STATUS}" = "403"

curl -fsS "${BASE}/memories" \
  -H "Authorization: Bearer ${TOKEN}" | jq -e 'length >= 1 and (.[0].assets | length >= 1)' >/dev/null

su - postgres -c "psql '${DB_URL}' -c \"INSERT INTO asset_exif (\\\"assetId\\\", latitude, longitude, description, \\\"projectionType\\\") VALUES ('${ASSET_ID}', 37.7749, -122.4194, '', 'equirectangular') ON CONFLICT (\\\"assetId\\\") DO UPDATE SET latitude = EXCLUDED.latitude, longitude = EXCLUDED.longitude, \\\"projectionType\\\" = EXCLUDED.\\\"projectionType\\\"\"" >/dev/null
curl -fsS "${BASE}/map/markers" \
  -H "Authorization: Bearer ${TOKEN}" | jq -e 'length >= 1 and .[0].lat == 37.7749' >/dev/null
curl -fsS "${BASE}/assets/${ASSET_ID}" \
  -H "Authorization: Bearer ${TOKEN}" | jq -e '.exifInfo.projectionType == "equirectangular"' >/dev/null

curl -fsS -X POST "${BASE}/search/metadata" \
  -H "Authorization: Bearer ${TOKEN}" -H 'content-type: application/json' \
  -d '{"query":"upload"}' | jq -e '.assets.total >= 1' >/dev/null

FOLDER="$(curl -fsS "${BASE}/view/folder/unique-paths" -H "Authorization: Bearer ${TOKEN}" | jq -r '.[0]')"
curl -fsS --get "${BASE}/view/folder" \
  -H "Authorization: Bearer ${TOKEN}" \
  --data-urlencode "path=${FOLDER}" | jq -e 'length >= 1' >/dev/null

echo "Domus integration smoke passed"
