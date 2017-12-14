$url = "https://win.rustup.rs"
$output = "$PSScriptRoot\rustup-init.exe"

Invoke-WebRequest -Uri $url -OutFile $output
rustup-init -yv --default-toolchain %channel% --default-host %target%
$env:Path += ";%USERPROFILE%\.cargo\bin"
