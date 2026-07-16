$ErrorActionPreference = "Stop"

$bundleDirectory = Resolve-Path "src-tauri/target/release/bundle"
$checksumPath = Join-Path $bundleDirectory "SHA256SUMS.txt"
$artifacts = Get-ChildItem -Path $bundleDirectory -Recurse -File |
  Where-Object { $_.Extension -in @(".exe", ".msi", ".sig") } |
  Sort-Object FullName

if ($artifacts.Count -eq 0) {
  throw "No Windows release artifacts were found for checksum generation."
}

$lines = foreach ($artifact in $artifacts) {
  $hash = (Get-FileHash -LiteralPath $artifact.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
  $relativePath = [System.IO.Path]::GetRelativePath($bundleDirectory, $artifact.FullName).Replace("\", "/")
  "$hash  $relativePath"
}
$lines | Set-Content -LiteralPath $checksumPath -Encoding utf8

$version = (Get-Content package.json | ConvertFrom-Json).version
gh release upload "v$version" $checksumPath --clobber
