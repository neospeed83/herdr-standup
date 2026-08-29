$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force bin | Out-Null
Invoke-WebRequest "https://github.com/neospeed83/herdr-standup/releases/latest/download/herdr-standup-windows-x86_64.exe" -OutFile "bin/herdr-standup.exe"
