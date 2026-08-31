$ErrorActionPreference = "Stop"
$Version = "0.4.0"
$Asset = "herdr-standup-windows-x86_64.exe"
$Base = "https://github.com/neospeed83/herdr-standup/releases/download/v$Version"
New-Item -ItemType Directory -Force bin | Out-Null
$Temp = "bin/.herdr-standup-$PID.exe"
$Checksum = "$Temp.sha256"
try {
  Invoke-WebRequest "$Base/$Asset" -OutFile $Temp
  Invoke-WebRequest "$Base/$Asset.sha256" -OutFile $Checksum
  $Expected = ((Get-Content $Checksum -Raw).Trim() -split '\s+')[0]
  $Actual = (Get-FileHash $Temp -Algorithm SHA256).Hash
  if ($Actual -ne $Expected) { throw "Checksum verification failed" }
  Move-Item -Force $Temp "bin/herdr-standup.exe"
} finally {
  Remove-Item -Force -ErrorAction SilentlyContinue $Temp, $Checksum
}
