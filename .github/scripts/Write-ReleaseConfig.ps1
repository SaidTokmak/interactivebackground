$ErrorActionPreference = "Stop"

$windows = @{}
if (-not [string]::IsNullOrWhiteSpace($env:WINDOWS_CERTIFICATE_THUMBPRINT)) {
  $windows = @{
    certificateThumbprint = $env:WINDOWS_CERTIFICATE_THUMBPRINT
    digestAlgorithm = "sha256"
    timestampUrl = $env:WINDOWS_TIMESTAMP_URL
  }
}

$configuration = @{
  bundle = @{
    createUpdaterArtifacts = $true
    windows = $windows
  }
  plugins = @{
    updater = @{
      pubkey = $env:TAURI_UPDATER_PUBKEY
      endpoints = @(
        "https://github.com/SaidTokmak/interactivebackground/releases/latest/download/latest.json"
      )
      windows = @{
        installMode = "passive"
      }
    }
  }
}

$configuration |
  ConvertTo-Json -Depth 8 |
  Set-Content -LiteralPath "src-tauri/tauri.release.conf.json" -Encoding utf8
