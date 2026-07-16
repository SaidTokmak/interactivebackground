$ErrorActionPreference = "Stop"

$required = @(
  "TAURI_SIGNING_PRIVATE_KEY",
  "TAURI_SIGNING_PRIVATE_KEY_PASSWORD",
  "TAURI_UPDATER_PUBKEY"
)

$missing = $required | Where-Object { [string]::IsNullOrWhiteSpace((Get-Item "Env:$_").Value) }
if ($missing.Count -gt 0) {
  throw "Missing updater release settings: $($missing -join ', ')"
}

if (-not [string]::IsNullOrWhiteSpace($env:WINDOWS_CERTIFICATE)) {
  if ([string]::IsNullOrWhiteSpace($env:WINDOWS_CERTIFICATE_PASSWORD)) {
    throw "WINDOWS_CERTIFICATE_PASSWORD is required when WINDOWS_CERTIFICATE is set."
  }
  if ([string]::IsNullOrWhiteSpace($env:WINDOWS_TIMESTAMP_URL)) {
    throw "WINDOWS_TIMESTAMP_URL is required when Windows code signing is enabled."
  }
}
