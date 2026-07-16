$ErrorActionPreference = "Stop"

$certificateDirectory = Join-Path $env:RUNNER_TEMP "interactivebackground-certificate"
New-Item -ItemType Directory -Force -Path $certificateDirectory | Out-Null
$encodedPath = Join-Path $certificateDirectory "certificate.txt"
$pfxPath = Join-Path $certificateDirectory "certificate.pfx"
Set-Content -LiteralPath $encodedPath -Value $env:WINDOWS_CERTIFICATE -NoNewline
certutil -decode $encodedPath $pfxPath | Out-Null

$password = ConvertTo-SecureString -String $env:WINDOWS_CERTIFICATE_PASSWORD -Force -AsPlainText
$certificate = Import-PfxCertificate `
  -FilePath $pfxPath `
  -CertStoreLocation Cert:\CurrentUser\My `
  -Password $password

if (-not $certificate.Thumbprint) {
  throw "The Windows signing certificate was imported without a thumbprint."
}
"WINDOWS_CERTIFICATE_THUMBPRINT=$($certificate.Thumbprint)" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
