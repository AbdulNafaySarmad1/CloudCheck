param(
  [Parameter(Mandatory = $true)]
  [string]$FilePath
)

$ErrorActionPreference = "Stop"

if ($env:NOCTURNE_WINDOWS_SIGNING_ENABLED -ne "true") {
  Write-Host "Unsigned validation build: skipping Authenticode for $FilePath"
  exit 0
}

$required = @("WINDOWS_CERTIFICATE_PFX_BASE64", "WINDOWS_CERTIFICATE_PASSWORD", "WINDOWS_TIMESTAMP_URL")
foreach ($name in $required) {
  if ([string]::IsNullOrWhiteSpace([Environment]::GetEnvironmentVariable($name))) {
    throw "Required Windows signing secret/environment value is missing: $name"
  }
}

$signtool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" |
  Sort-Object FullName -Descending |
  Select-Object -First 1
if (-not $signtool) { throw "signtool.exe was not found on this runner." }

$pfxPath = Join-Path $env:RUNNER_TEMP ("cloudcheck-" + [guid]::NewGuid() + ".pfx")
$certificate = $null
try {
  [IO.File]::WriteAllBytes($pfxPath, [Convert]::FromBase64String($env:WINDOWS_CERTIFICATE_PFX_BASE64))
  $password = ConvertTo-SecureString $env:WINDOWS_CERTIFICATE_PASSWORD -AsPlainText -Force
  $certificate = Import-PfxCertificate -FilePath $pfxPath -CertStoreLocation Cert:\CurrentUser\My -Password $password
  if (-not $certificate.HasPrivateKey) { throw "The imported Authenticode certificate has no private key." }

  & $signtool.FullName sign /sha1 $certificate.Thumbprint /fd SHA256 /tr $env:WINDOWS_TIMESTAMP_URL /td SHA256 $FilePath
  if ($LASTEXITCODE -ne 0) { throw "Authenticode signing failed for $FilePath" }
  & $signtool.FullName verify /pa /all /v $FilePath
  if ($LASTEXITCODE -ne 0) { throw "Authenticode verification failed for $FilePath" }
} finally {
  if ($certificate) { Remove-Item "Cert:\CurrentUser\My\$($certificate.Thumbprint)" -Force -ErrorAction SilentlyContinue }
  Remove-Item $pfxPath -Force -ErrorAction SilentlyContinue
}
