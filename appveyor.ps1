$url = "https://win.rustup.rs"
$output = "$PSScriptRoot\rustup-init.exe"

echo "Running setup:"
echo ""
echo "Downloading..."

Invoke-WebRequest -Uri $url -OutFile $output

echo "Initializing rustup..."
rustup-init -yv
$env:Path += ";%USERPROFILE%\.cargo\bin"

cargo test
